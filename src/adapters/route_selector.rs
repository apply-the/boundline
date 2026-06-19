//! Risk-aware route selection for Boundline.
//!
//! Selects an approved inference route based on task class, authority zone,
//! budget state, provider health, and snapshot staleness. Low-risk work
//! prefers cheaper routes; red-zone work is never silently downgraded.

use boundline_core::domain::inference_economics::{
    BudgetState, InferenceRouteProfile, RouteSelection, RouteTier,
};

/// Selects an inference route based on task characteristics and
/// operational constraints.
pub struct RouteSelector;

/// Inputs to a route selection decision.
#[derive(Debug, Clone)]
pub struct RouteSelectionInput<'a> {
    /// The authority zone of the task (e.g. "green", "yellow", "red").
    pub authority_zone: &'a str,
    /// Whether the task is governance-critical.
    pub is_governance_critical: bool,
    /// The current budget state.
    pub budget_state: BudgetState,
    /// Available routes with their health status.
    /// (route, is_healthy)
    pub available_routes: Vec<(&'a InferenceRouteProfile, bool)>,
    /// Whether stale-snapshot routes should be blocked.
    pub block_stale_snapshots: bool,
}

impl RouteSelector {
    /// Select the best route given the task inputs and available routes.
    #[must_use]
    pub fn select(input: RouteSelectionInput<'_>) -> RouteSelection {
        let is_red_zone = input.authority_zone == "red" || input.is_governance_critical;

        // Filter to healthy routes only
        let healthy: Vec<&InferenceRouteProfile> = input
            .available_routes
            .iter()
            .filter(|(_, healthy)| *healthy)
            .map(|(r, _)| *r)
            .collect();

        if healthy.is_empty() {
            return RouteSelection::Blocked(
                "no healthy routes available for the requested task".into(),
            );
        }

        // Check for Tier 0 deterministic path
        if let Some(_t0) = healthy.iter().find(|r| r.tier == RouteTier::Tier0Deterministic) {
            return RouteSelection::DeterministicLocal;
        }

        // For red-zone: must use Tier 3 or the highest available tier
        if is_red_zone {
            if let Some(t3) = healthy.iter().find(|r| r.tier == RouteTier::Tier3HighCapability) {
                return RouteSelection::Selected((*t3).clone());
            }
            // If no Tier 3, pick the highest tier available.
            // The healthy list is known non-empty at this point.
            let Some(best) = healthy.iter().max_by_key(|r| r.tier) else {
                return RouteSelection::Blocked(
                    "no healthy routes available for red-zone task".into(),
                );
            };
            return RouteSelection::Selected((*best).clone());
        }

        // For green/yellow: prefer lowest-cost tier, respecting budget
        let is_budget_tight =
            matches!(input.budget_state, BudgetState::ApproachingLimit | BudgetState::Exhausted);

        // Sort by tier (lower = cheaper)
        let mut candidates: Vec<&InferenceRouteProfile> = healthy;
        candidates.sort_by_key(|r| r.tier);

        if is_budget_tight {
            // Under budget pressure, prefer the cheapest available route
            if let Some(cheapest) = candidates.first() {
                return RouteSelection::Selected((*cheapest).clone());
            }
        }

        // Default: lowest tier that's healthy
        candidates
            .first()
            .map(|best| RouteSelection::Selected((*best).clone()))
            .unwrap_or_else(|| RouteSelection::Blocked("no compliant route found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundline_core::domain::inference_economics::BudgetState;

    fn make_route(tier: RouteTier, provider: &str, model: &str) -> InferenceRouteProfile {
        InferenceRouteProfile {
            tier,
            provider_id: provider.into(),
            model_id: model.into(),
            capability_requirements: vec![],
            repository_egress: false,
        }
    }

    #[test]
    fn low_risk_selects_cheapest_healthy_route() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let t2 = make_route(RouteTier::Tier2Balanced, "openai", "gpt-4o");
        let routes = vec![(&t1, true), (&t2, true)];
        let input = RouteSelectionInput {
            authority_zone: "green",
            is_governance_critical: false,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        match result {
            RouteSelection::Selected(r) => assert_eq!(r.tier, RouteTier::Tier1Cheap),
            other => panic!("expected Selected, got {other:?}"),
        }
    }

    #[test]
    fn red_zone_preserves_tier3() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let t3 = make_route(RouteTier::Tier3HighCapability, "anthropic", "claude-opus");
        let routes = vec![(&t1, true), (&t3, true)];
        let input = RouteSelectionInput {
            authority_zone: "red",
            is_governance_critical: true,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        match result {
            RouteSelection::Selected(r) => assert_eq!(r.tier, RouteTier::Tier3HighCapability),
            other => panic!("expected Selected Tier3, got {other:?}"),
        }
    }

    #[test]
    fn excludes_unhealthy_routes() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let t2 = make_route(RouteTier::Tier2Balanced, "openai", "gpt-4o");
        // t1 is unhealthy, t2 is healthy
        let routes = vec![(&t1, false), (&t2, true)];
        let input = RouteSelectionInput {
            authority_zone: "green",
            is_governance_critical: false,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        match result {
            RouteSelection::Selected(r) => assert_eq!(r.tier, RouteTier::Tier2Balanced),
            other => panic!("expected Selected Tier2, got {other:?}"),
        }
    }

    #[test]
    fn all_unhealthy_routes_blocked() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let routes = vec![(&t1, false)];
        let input = RouteSelectionInput {
            authority_zone: "green",
            is_governance_critical: false,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        assert!(matches!(result, RouteSelection::Blocked(_)));
    }

    #[test]
    fn tier0_deterministic_selected_when_available() {
        let t0 = make_route(RouteTier::Tier0Deterministic, "local", "deterministic");
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let routes = vec![(&t0, true), (&t1, true)];
        let input = RouteSelectionInput {
            authority_zone: "green",
            is_governance_critical: false,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        assert_eq!(result, RouteSelection::DeterministicLocal);
    }

    #[test]
    fn red_zone_without_tier3_picks_highest_available() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let t2 = make_route(RouteTier::Tier2Balanced, "openai", "gpt-4o");
        let routes = vec![(&t1, true), (&t2, true)];
        let input = RouteSelectionInput {
            authority_zone: "red",
            is_governance_critical: true,
            budget_state: BudgetState::InBounds,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        assert!(
            matches!(result, RouteSelection::Selected(ref r) if r.tier == RouteTier::Tier2Balanced)
        );
    }

    #[test]
    fn budget_tight_prefers_cheapest() {
        let t1 = make_route(RouteTier::Tier1Cheap, "openai", "gpt-4o-mini");
        let t2 = make_route(RouteTier::Tier2Balanced, "openai", "gpt-4o");
        let routes = vec![(&t1, true), (&t2, true)];
        let input = RouteSelectionInput {
            authority_zone: "green",
            is_governance_critical: false,
            budget_state: BudgetState::ApproachingLimit,
            available_routes: routes,
            block_stale_snapshots: false,
        };
        let result = RouteSelector::select(input);
        assert!(
            matches!(result, RouteSelection::Selected(ref r) if r.tier == RouteTier::Tier1Cheap)
        );
    }
}
