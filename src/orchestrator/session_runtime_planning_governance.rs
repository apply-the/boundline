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
