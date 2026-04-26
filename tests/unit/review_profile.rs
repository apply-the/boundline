use synod::domain::review::{
    AdjudicationDefinition, ReviewProfile, ReviewProfileError, ReviewScenario, ReviewTrigger,
    ReviewerDefinition, ReviewerDisposition, ReviewerFinding, VoteRuleDefinition,
};

fn sample_profile() -> ReviewProfile {
    ReviewProfile {
        triggers: vec![ReviewTrigger::PrReady],
        reviewers: vec![
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
        ],
        vote_rule: VoteRuleDefinition::default(),
        adjudication: AdjudicationDefinition::default(),
        scenarios: vec![ReviewScenario {
            trigger: ReviewTrigger::PrReady,
            findings: vec![
                ReviewerFinding {
                    reviewer_id: "safety".to_string(),
                    disposition: ReviewerDisposition::Approve,
                    summary: "No blocking issues".to_string(),
                    details: None,
                },
                ReviewerFinding {
                    reviewer_id: "maintainability".to_string(),
                    disposition: ReviewerDisposition::Concern,
                    summary: "Minor cleanup follow-up".to_string(),
                    details: None,
                },
            ],
            adjudication_finding: None,
        }],
    }
}

#[test]
fn review_profile_validation_accepts_bounded_review_configuration() {
    sample_profile().validate().unwrap();
}

#[test]
fn review_profile_validation_rejects_duplicate_reviewers_and_missing_triggers() {
    let mut profile = sample_profile();
    profile.triggers.clear();
    assert_eq!(profile.validate().unwrap_err(), ReviewProfileError::MissingReviewTriggers);

    let mut profile = sample_profile();
    profile.reviewers[1].reviewer_id = "safety".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateReviewerId("safety".to_string())
    );
}

#[test]
fn review_profile_validation_rejects_duplicate_adjudicator_and_duplicate_scenarios() {
    let mut profile = sample_profile();
    profile.adjudication =
        AdjudicationDefinition { enabled: true, reviewer_id: Some("safety".to_string()) };
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateAdjudicatorReviewerId("safety".to_string())
    );

    let mut profile = sample_profile();
    profile.scenarios.push(profile.scenarios[0].clone());
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateScenarioTrigger(ReviewTrigger::PrReady)
    );
}
