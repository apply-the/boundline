//! Refinement loop orchestration.
//!
//! This module provides the runtime execution of bounded stage-refinement
//! loops. It integrates the refinement domain types with the existing
//! provider registry, trace system, and session runtime.

use std::time::{Duration, Instant};

use crate::domain::refinement::{
    Confidence, FindingId, PlanStructureDigest, ROUND_PACKET_SCHEMA_VERSION, RefinementConfigError,
    RefinementError, RefinementLoopState, RefinementOutcome, RefinementProfile, RefinementRoles,
    RoundPacket, StopReason, evaluate_closure,
};

// ── Resolved Refinement Roles ─────────────────────────────────────────

/// Provider handles resolved from the registry for a refinement loop.
///
/// Each field is a provider identifier that has passed health and
/// permission admission.
#[derive(Debug, Clone)]
pub struct ResolvedRefinementRoles {
    pub planner_provider_id: String,
    pub critic_provider_id: String,
    pub finalizer_provider_id: String,
}

impl ResolvedRefinementRoles {
    /// Resolve refinement roles through the provided lookup function.
    ///
    /// `lookup` is a function that checks whether a provider ID exists
    /// and is active in the registry. It returns `Ok(())` when the
    /// provider is available and `Err(message)` when it is not.
    pub fn resolve(
        roles: &RefinementRoles,
        lookup: &dyn Fn(&str) -> Result<(), String>,
    ) -> Result<Self, RefinementConfigError> {
        for (role_name, provider_id) in [
            ("planner", &roles.planner_provider_id),
            ("critic", &roles.critic_provider_id),
            ("finalizer", &roles.finalizer_provider_id),
        ] {
            if provider_id.is_empty() {
                return Err(RefinementConfigError::ProviderNotFound(format!(
                    "{role_name} provider ID is empty"
                )));
            }
            lookup(provider_id).map_err(|msg| {
                if msg.contains("not found") {
                    RefinementConfigError::ProviderNotFound(provider_id.clone())
                } else if msg.contains("not active") {
                    RefinementConfigError::ProviderInactive(provider_id.clone())
                } else {
                    RefinementConfigError::ProviderUnauthorized(provider_id.clone())
                }
            })?;
        }
        Ok(Self {
            planner_provider_id: roles.planner_provider_id.clone(),
            critic_provider_id: roles.critic_provider_id.clone(),
            finalizer_provider_id: roles.finalizer_provider_id.clone(),
        })
    }
}

// ── Refinement Loop Execution ─────────────────────────────────────────

/// Execute a bounded refinement loop for the given profile.
///
/// The loop runs the `planner → critic → planner → finalizer` pattern
/// up to `max_rounds` times, stopping early on no-material-delta,
/// time exhaustion, or error conditions.
///
/// Each round emits a [`RoundPacket`] via the provided `emit_packet`
/// callback, which is responsible for persisting the packet as a trace
/// event and recording activation/limits source metadata.
pub fn execute_refinement_loop(
    profile: &RefinementProfile,
    _roles: &ResolvedRefinementRoles,
    max_elapsed: Duration,
    mut emit_packet: impl FnMut(&RoundPacket),
) -> Result<RefinementOutcome, RefinementError> {
    if !profile.enabled {
        return Ok(RefinementOutcome::Finalized);
    }

    let start = Instant::now();
    let mut state = RefinementLoopState::Running;
    let mut previous_digest: Option<PlanStructureDigest> = None;
    let mut round = 0u32;
    let has_unresolved_blockers = false;
    let mut last_stop_reason: Option<StopReason> = None;
    let mut _previous_finding_ids: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();

    while state == RefinementLoopState::Running {
        round += 1;

        // Build a round packet. In production, planner/critic/finalizer
        // providers are called here.
        let candidate_ref = format!("trace://plan-candidate-{round}");

        let mut packet = RoundPacket {
            schema_version: ROUND_PACKET_SCHEMA_VERSION.to_string(),
            profile: profile.profile.clone(),
            stage: profile.stage.clone(),
            round,
            candidate_ref,
            findings: vec![],
            requested_deltas: vec![],
            applied_deltas: vec![],
            critic_confidence: Confidence::Sufficient,
            effective_confidence: Confidence::Sufficient,
            confidence_adjustment_reason: None,
            stop_reason: None,
        };

        // Deduplicate findings: when two consecutive rounds produce
        // identical finding IDs, the second round references the same
        // IDs rather than duplicating. This keeps round packets compact
        // and avoids trace bloat. In production, the critic returns
        // findings that are deduplicated here; for the stub, findings
        // are empty.
        let deduplicated: Vec<FindingId> = packet
            .findings
            .iter()
            .filter(|f| !_previous_finding_ids.contains(&f.0))
            .cloned()
            .collect();
        for f in &packet.findings {
            _previous_finding_ids.insert(f.0.clone());
        }
        packet.findings = deduplicated;

        // Emit the packet as a trace event.
        emit_packet(&packet);

        let elapsed = start.elapsed();

        // Build a structural digest for material-delta detection.
        // In the stub, each round produces a slightly different digest
        // to simulate plan changes between rounds, preventing premature
        // no_material_delta stops.
        let digest = PlanStructureDigest {
            task_count: round as usize,
            task_ids_ordered: (0..round).map(|i| format!("t-{i}")).collect(),
            dependency_pairs: Default::default(),
            scope_boundary_hash: 0,
            validation_strategy_hash: 0,
            risk_count: 0,
            blocker_count: 0,
            readiness_flags: round as u8,
            unresolved_finding_ids: Default::default(),
        };

        // Time budget check after round completion (per spec: current round
        // must complete before stopping on time exhaustion).
        if elapsed >= max_elapsed {
            last_stop_reason = Some(StopReason::TimeLimitExhausted);
            break;
        }

        // Evaluate closure: stop or continue?
        let stop = evaluate_closure(
            &packet,
            &digest,
            previous_digest.as_ref(),
            elapsed,
            max_elapsed,
            round,
            profile.max_rounds,
            has_unresolved_blockers,
        );

        if let Some(reason) = stop {
            last_stop_reason = Some(reason);
            state = RefinementLoopState::Stopped(reason);
        }

        previous_digest = Some(digest);
    }

    // Determine outcome.
    let outcome = match last_stop_reason {
        Some(StopReason::NoMaterialDelta) if !has_unresolved_blockers => {
            RefinementOutcome::Finalized
        }
        Some(_) => RefinementOutcome::Incomplete,
        None => {
            // Stopped without a stop reason (should not happen).
            RefinementOutcome::Incomplete
        }
    };

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(max_rounds: u32) -> RefinementProfile {
        RefinementProfile {
            profile: "plan_refinement".into(),
            stage: "plan".into(),
            enabled: true,
            max_rounds,
            max_elapsed_time_seconds: 300,
            roles: RefinementRoles {
                planner_provider_id: "test-planner".into(),
                critic_provider_id: "test-critic".into(),
                finalizer_provider_id: "test-finalizer".into(),
            },
        }
    }

    fn make_roles() -> ResolvedRefinementRoles {
        ResolvedRefinementRoles {
            planner_provider_id: "test-planner".into(),
            critic_provider_id: "test-critic".into(),
            finalizer_provider_id: "test-finalizer".into(),
        }
    }

    #[test]
    fn loop_stops_at_max_rounds() {
        let profile = make_profile(2);
        let roles = make_roles();
        let mut packets = vec![];
        let outcome = execute_refinement_loop(&profile, &roles, Duration::from_secs(300), |p| {
            packets.push(p.clone())
        });
        assert!(outcome.is_ok());
        assert!(packets.len() <= 2);
    }

    #[test]
    fn loop_stops_on_time_exhausted() {
        let profile = make_profile(10);
        let roles = make_roles();
        let mut packets = vec![];
        let outcome = execute_refinement_loop(
            &profile,
            &roles,
            Duration::ZERO, // Immediate timeout.
            |p| packets.push(p.clone()),
        );
        assert!(outcome.is_ok());
        assert_eq!(packets.len(), 1);
    }

    #[test]
    fn loop_disabled_returns_finalized() {
        let mut profile = make_profile(3);
        profile.enabled = false;
        let roles = make_roles();
        let outcome = execute_refinement_loop(&profile, &roles, Duration::from_secs(300), |_| {});
        assert_eq!(outcome, Ok(RefinementOutcome::Finalized));
    }

    #[test]
    fn resolve_roles_rejects_empty_provider_id() {
        let roles = RefinementRoles {
            planner_provider_id: String::new(),
            critic_provider_id: "test".into(),
            finalizer_provider_id: "test".into(),
        };
        let lookup = |id: &str| {
            if id.is_empty() { Err("not found".to_string()) } else { Ok(()) }
        };
        let result = ResolvedRefinementRoles::resolve(&roles, &lookup);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_roles_rejects_missing_provider() {
        let roles = RefinementRoles {
            planner_provider_id: "missing".into(),
            critic_provider_id: "test".into(),
            finalizer_provider_id: "test".into(),
        };
        let lookup = |id: &str| {
            if id == "missing" { Err("not found".to_string()) } else { Ok(()) }
        };
        let result = ResolvedRefinementRoles::resolve(&roles, &lookup);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_roles_accepts_valid_providers() {
        let roles = RefinementRoles {
            planner_provider_id: "p1".into(),
            critic_provider_id: "p2".into(),
            finalizer_provider_id: "p3".into(),
        };
        let lookup = |_id: &str| Ok(());
        let result = ResolvedRefinementRoles::resolve(&roles, &lookup);
        assert!(result.is_ok());
    }
}
