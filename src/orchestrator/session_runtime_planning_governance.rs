use super::{
    EffectiveRouting, ModelRoute, ProviderReviewDisposition, ReviewerDefinition,
    ReviewerDisposition, RouteSlot, StageCouncilFindingDisposition, StageCouncilOutcome,
    StageCouncilRequest, StageCouncilReviewerRoute, StageCouncilStatus, VotingSessionState,
};

pub(super) fn discovery_stage_council_request(
    stage_key: &str,
    goal: &str,
    stage_brief_ref: &str,
) -> StageCouncilRequest {
    StageCouncilRequest {
        stage_key: stage_key.to_string(),
        goal: goal.to_string(),
        producer_slot: RouteSlot::Planning.as_str().to_string(),
        phase: "planning-discovery".to_string(),
        target_refs: vec![stage_brief_ref.to_string()],
        current_artifact_ref: Some(stage_brief_ref.to_string()),
        constraints: vec![
            "use independent reviewer routes when available".to_string(),
            "do not promote discovery planning when council independence collapses".to_string(),
        ],
    }
}

pub(super) fn discovery_stage_council_reviewers(
    routing: &EffectiveRouting,
) -> Vec<StageCouncilReviewerRoute> {
    let configured = routing
        .reviewer_roles
        .iter()
        .take(2)
        .map(|(reviewer_id, reviewer_route)| StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: reviewer_id.clone(),
                role: reviewer_id.replace(['_', '-'], " "),
                source: Some(model_route_label(&reviewer_route.route)),
                weight: 1,
            },
            route: reviewer_route.route.clone(),
        })
        .collect::<Vec<_>>();
    if configured.len() == 2 {
        return configured;
    }

    let fallback_route = routing.review.route.clone();
    vec![
        StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: configured
                    .first()
                    .map(|route| route.reviewer.reviewer_id.clone())
                    .unwrap_or_else(|| "reviewer-a".to_string()),
                role: configured
                    .first()
                    .map(|route| route.reviewer.role.clone())
                    .unwrap_or_else(|| "discovery challenger a".to_string()),
                source: configured
                    .first()
                    .and_then(|route| route.reviewer.source.clone())
                    .or_else(|| Some(model_route_label(&fallback_route))),
                weight: 1,
            },
            route: configured
                .first()
                .map(|route| route.route.clone())
                .unwrap_or_else(|| fallback_route.clone()),
        },
        StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: configured
                    .get(1)
                    .map(|route| route.reviewer.reviewer_id.clone())
                    .unwrap_or_else(|| "reviewer-b".to_string()),
                role: configured
                    .get(1)
                    .map(|route| route.reviewer.role.clone())
                    .unwrap_or_else(|| "discovery challenger b".to_string()),
                source: configured
                    .get(1)
                    .and_then(|route| route.reviewer.source.clone())
                    .or_else(|| Some(model_route_label(&fallback_route))),
                weight: 1,
            },
            route: configured.get(1).map(|route| route.route.clone()).unwrap_or(fallback_route),
        },
    ]
}

pub(super) fn reviewer_disposition_from_provider(
    disposition: ProviderReviewDisposition,
) -> ReviewerDisposition {
    match disposition {
        ProviderReviewDisposition::Approve => ReviewerDisposition::Approve,
        ProviderReviewDisposition::Concern => ReviewerDisposition::Concern,
        ProviderReviewDisposition::Block => ReviewerDisposition::Block,
    }
}

pub(super) fn stage_council_disposition_from_provider(
    disposition: ProviderReviewDisposition,
) -> StageCouncilFindingDisposition {
    match disposition {
        ProviderReviewDisposition::Approve => StageCouncilFindingDisposition::Approve,
        ProviderReviewDisposition::Concern => StageCouncilFindingDisposition::Concern,
        ProviderReviewDisposition::Block => StageCouncilFindingDisposition::Block,
    }
}

pub(super) fn provider_review_disposition_text(
    disposition: ProviderReviewDisposition,
) -> &'static str {
    match disposition {
        ProviderReviewDisposition::Approve => "approve",
        ProviderReviewDisposition::Concern => "concern",
        ProviderReviewDisposition::Block => "block",
    }
}

pub(super) fn planning_stage_council_block_reason(
    stage_key: &str,
    outcome: &StageCouncilOutcome,
) -> String {
    let summary = outcome
        .reviewer_findings
        .iter()
        .find(|finding| finding.disposition == StageCouncilFindingDisposition::Block)
        .map(|finding| finding.summary.as_str())
        .unwrap_or(outcome.next_action.as_str());
    format!("{stage_key} stage council blocked planning: {summary}")
}

pub(super) fn stage_council_voting_session_state(
    stage_key: &str,
    outcome: &StageCouncilOutcome,
) -> VotingSessionState {
    VotingSessionState {
        trigger: format!("stage_council:{stage_key}"),
        reviewed_evidence_ref: Some(outcome.producer_output.evidence_ref.clone()),
        result: stage_council_status_text(outcome.status).to_string(),
        reviewer_findings: outcome
            .reviewer_findings
            .iter()
            .map(|finding| {
                format!(
                    "{} [{}]: {}",
                    finding.reviewer_id, finding.effective_route, finding.summary
                )
            })
            .collect(),
        adjudication_result: outcome
            .adjudication
            .as_ref()
            .map(|adjudication| format!("{}: {}", adjudication.decision, adjudication.rationale)),
        blocking: outcome.status == StageCouncilStatus::Blocked,
        next_action: outcome.next_action.clone(),
    }
}

pub(super) fn model_route_label(route: &ModelRoute) -> String {
    format!("{}/{}", route.runtime.as_str(), route.model)
}

fn stage_council_status_text(status: StageCouncilStatus) -> &'static str {
    match status {
        StageCouncilStatus::Proceed => "proceed",
        StageCouncilStatus::Blocked => "blocked",
        StageCouncilStatus::Degraded => "degraded",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::domain::configuration::{
        EffectiveRouting, ModelRoute, RuntimeKind, SourcedRoute, ValueSource,
    };
    use crate::domain::review::ReviewerDisposition;
    use crate::domain::stage_council::{
        StageCouncilAdjudication, StageCouncilArtifact, StageCouncilFinding,
        StageCouncilFindingDisposition, StageCouncilOutcome, StageCouncilStatus,
        StageCouncilVoteResolution,
    };
    use crate::orchestrator::session_runtime::ProviderReviewDisposition;

    use super::{
        discovery_stage_council_request, discovery_stage_council_reviewers, model_route_label,
        planning_stage_council_block_reason, provider_review_disposition_text,
        reviewer_disposition_from_provider, stage_council_disposition_from_provider,
        stage_council_voting_session_state,
    };

    const BRIEF_REF: &str = "brief.md";
    const DISCOVERY_STAGE_KEY: &str = "plan:discovery";
    const GOAL: &str = "Clarify the bounded context";

    #[test]
    fn planning_governance_helpers_cover_request_reviewer_and_disposition_paths() {
        let request = discovery_stage_council_request(DISCOVERY_STAGE_KEY, GOAL, BRIEF_REF);
        assert_eq!(request.stage_key, DISCOVERY_STAGE_KEY);
        assert_eq!(request.goal, GOAL);
        assert_eq!(request.producer_slot, "planning");
        assert_eq!(request.phase, "planning-discovery");
        assert_eq!(request.target_refs, vec![BRIEF_REF.to_string()]);
        assert_eq!(request.current_artifact_ref.as_deref(), Some(BRIEF_REF));
        assert_eq!(request.constraints.len(), 2);

        let review_route =
            ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
        let first_reviewer_route =
            ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() };
        let second_reviewer_route =
            ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() };

        let configured_routing = EffectiveRouting {
            planning: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            implementation: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            verification: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            review: SourcedRoute { route: review_route.clone(), source: ValueSource::Workspace },
            chat: None,
            adjudication: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            reviewer_roles: BTreeMap::from([
                (
                    "security_reviewer".to_string(),
                    SourcedRoute {
                        route: first_reviewer_route.clone(),
                        source: ValueSource::Workspace,
                    },
                ),
                (
                    "ux-reviewer".to_string(),
                    SourcedRoute {
                        route: second_reviewer_route.clone(),
                        source: ValueSource::Workspace,
                    },
                ),
            ]),
        };
        let configured_reviewers = discovery_stage_council_reviewers(&configured_routing);
        assert_eq!(configured_reviewers.len(), 2);
        assert_eq!(configured_reviewers[0].reviewer.reviewer_id, "security_reviewer");
        assert_eq!(configured_reviewers[0].reviewer.role, "security reviewer");
        assert_eq!(
            configured_reviewers[0].reviewer.source.as_deref(),
            Some("gemini/gemini-2.5-pro")
        );
        assert_eq!(configured_reviewers[1].reviewer.reviewer_id, "ux-reviewer");
        assert_eq!(configured_reviewers[1].reviewer.role, "ux reviewer");
        assert_eq!(configured_reviewers[1].route, second_reviewer_route);

        let fallback_routing = EffectiveRouting {
            planning: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            implementation: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            verification: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            review: SourcedRoute { route: review_route.clone(), source: ValueSource::Workspace },
            chat: None,
            adjudication: sourced_route(RuntimeKind::Copilot, "gpt-5.4"),
            reviewer_roles: BTreeMap::new(),
        };
        let fallback_reviewers = discovery_stage_council_reviewers(&fallback_routing);
        assert_eq!(fallback_reviewers.len(), 2);
        assert_eq!(fallback_reviewers[0].reviewer.reviewer_id, "reviewer-a");
        assert_eq!(fallback_reviewers[0].route, review_route);
        assert_eq!(fallback_reviewers[1].reviewer.source.as_deref(), Some("copilot/gpt-5.4"));

        assert_eq!(
            reviewer_disposition_from_provider(ProviderReviewDisposition::Approve),
            ReviewerDisposition::Approve
        );
        assert_eq!(
            reviewer_disposition_from_provider(ProviderReviewDisposition::Concern),
            ReviewerDisposition::Concern
        );
        assert_eq!(
            reviewer_disposition_from_provider(ProviderReviewDisposition::Block),
            ReviewerDisposition::Block
        );
        assert_eq!(
            stage_council_disposition_from_provider(ProviderReviewDisposition::Approve),
            StageCouncilFindingDisposition::Approve
        );
        assert_eq!(
            stage_council_disposition_from_provider(ProviderReviewDisposition::Concern),
            StageCouncilFindingDisposition::Concern
        );
        assert_eq!(
            stage_council_disposition_from_provider(ProviderReviewDisposition::Block),
            StageCouncilFindingDisposition::Block
        );
        assert_eq!(provider_review_disposition_text(ProviderReviewDisposition::Approve), "approve");
        assert_eq!(provider_review_disposition_text(ProviderReviewDisposition::Concern), "concern");
        assert_eq!(provider_review_disposition_text(ProviderReviewDisposition::Block), "block");
        assert_eq!(model_route_label(&first_reviewer_route), "gemini/gemini-2.5-pro");
    }

    #[test]
    fn planning_governance_helpers_cover_block_reason_and_voting_projection() {
        let outcome = StageCouncilOutcome {
            producer_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: "evidence/discovery.md".to_string(),
                summary: Some("draft discovery packet".to_string()),
            },
            reviewer_findings: vec![
                StageCouncilFinding {
                    reviewer_id: "reviewer-a".to_string(),
                    effective_route: "copilot/gpt-5.4".to_string(),
                    disposition: StageCouncilFindingDisposition::Concern,
                    summary: "needs tighter scope".to_string(),
                    accepted: false,
                },
                StageCouncilFinding {
                    reviewer_id: "reviewer-b".to_string(),
                    effective_route: "gemini/gemini-2.5-pro".to_string(),
                    disposition: StageCouncilFindingDisposition::Block,
                    summary: "independent review collapsed".to_string(),
                    accepted: false,
                },
            ],
            vote_resolution: StageCouncilVoteResolution {
                strategy: "bounded_majority".to_string(),
                accepted_findings: vec!["reviewer-b".to_string()],
                rejected_findings: Vec::new(),
                independent_review: false,
            },
            adjudication: Some(StageCouncilAdjudication {
                adjudicator_route: "claude/sonnet-4".to_string(),
                decision: "block".to_string(),
                rationale: "independence requirement not met".to_string(),
            }),
            revised_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: "evidence/discovery-revised.md".to_string(),
                summary: Some("revised discovery packet".to_string()),
            },
            status: StageCouncilStatus::Blocked,
            next_action: "restore distinct reviewer routes".to_string(),
        };

        assert_eq!(
            planning_stage_council_block_reason(DISCOVERY_STAGE_KEY, &outcome),
            "plan:discovery stage council blocked planning: independent review collapsed"
        );

        let voting_state = stage_council_voting_session_state(DISCOVERY_STAGE_KEY, &outcome);
        assert_eq!(voting_state.trigger, "stage_council:plan:discovery");
        assert_eq!(voting_state.reviewed_evidence_ref.as_deref(), Some("evidence/discovery.md"));
        assert_eq!(voting_state.result, "blocked");
        assert!(voting_state.blocking);
        assert_eq!(voting_state.next_action, "restore distinct reviewer routes");
        assert_eq!(voting_state.reviewer_findings.len(), 2);
        assert_eq!(
            voting_state.adjudication_result.as_deref(),
            Some("block: independence requirement not met")
        );
    }

    fn sourced_route(runtime: RuntimeKind, model: &str) -> SourcedRoute {
        SourcedRoute {
            route: ModelRoute { runtime, model: model.to_string() },
            source: ValueSource::BuiltIn,
        }
    }
}
