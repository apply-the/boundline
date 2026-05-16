use boundline::domain::review::{
    ReviewProfileError, ReviewerDefinition, ReviewerDisposition, ReviewerFinding, VoteDecision,
    VoteRuleDefinition, VoteStrategy,
};

fn reviewers() -> Vec<ReviewerDefinition> {
    vec![
        ReviewerDefinition {
            reviewer_id: "safety".to_string(),
            role: "Safety".to_string(),
            source: Some("gpt".to_string()),
            weight: 2,
        },
        ReviewerDefinition {
            reviewer_id: "maintainability".to_string(),
            role: "Maintainability".to_string(),
            source: Some("claude".to_string()),
            weight: 1,
        },
        ReviewerDefinition {
            reviewer_id: "release".to_string(),
            role: "Release".to_string(),
            source: Some("gemini".to_string()),
            weight: 1,
        },
    ]
}

fn finding(reviewer_id: &str, disposition: ReviewerDisposition) -> ReviewerFinding {
    ReviewerFinding::new(reviewer_id.to_string(), disposition, format!("{reviewer_id} summary"))
}

#[test]
fn majority_voting_accepts_and_weighted_voting_rejects_on_blocking() {
    let majority =
        VoteRuleDefinition { strategy: VoteStrategy::Majority, reject_on_blocking: false }
            .resolve(
                &reviewers(),
                &[
                    finding("safety", ReviewerDisposition::Approve),
                    finding("maintainability", ReviewerDisposition::Approve),
                    finding("release", ReviewerDisposition::Concern),
                ],
                None,
            )
            .unwrap();
    assert_eq!(majority.decision, VoteDecision::Accepted);
    assert_eq!(majority.approvals, 2);
    assert_eq!(majority.concerns, 1);
    assert_eq!(majority.blocks, 0);
    assert_eq!(majority.participants.len(), 3);

    let weighted =
        VoteRuleDefinition { strategy: VoteStrategy::Weighted, reject_on_blocking: true }
            .resolve(
                &reviewers(),
                &[
                    finding("safety", ReviewerDisposition::Block),
                    finding("maintainability", ReviewerDisposition::Approve),
                ],
                None,
            )
            .unwrap();
    assert_eq!(weighted.decision, VoteDecision::Rejected);
    assert_eq!(weighted.blocks, 2);
    assert_eq!(weighted.approvals, 1);
}

#[test]
fn voting_requests_adjudication_when_no_threshold_is_met() {
    let resolution =
        VoteRuleDefinition { strategy: VoteStrategy::Majority, reject_on_blocking: false }
            .resolve(
                &reviewers(),
                &[
                    finding("safety", ReviewerDisposition::Approve),
                    finding("maintainability", ReviewerDisposition::Concern),
                    finding("release", ReviewerDisposition::Block),
                ],
                None,
            )
            .unwrap();

    assert_eq!(resolution.decision, VoteDecision::NeedsAdjudication);
}

#[test]
fn vote_resolution_rejects_unknown_or_duplicate_findings() {
    let duplicate = VoteRuleDefinition::default()
        .resolve(
            &reviewers(),
            &[
                finding("safety", ReviewerDisposition::Approve),
                finding("safety", ReviewerDisposition::Concern),
            ],
            None,
        )
        .unwrap_err();
    assert_eq!(duplicate, ReviewProfileError::DuplicateFindingReviewer("safety".to_string()));

    let unknown = VoteRuleDefinition::default()
        .resolve(&reviewers(), &[finding("unknown", ReviewerDisposition::Approve)], None)
        .unwrap_err();
    assert_eq!(unknown, ReviewProfileError::UnknownFindingReviewer("unknown".to_string()));
}
