use boundline::domain::review::{
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
                ReviewerFinding::new(
                    "safety".to_string(),
                    ReviewerDisposition::Approve,
                    "No blocking issues".to_string(),
                ),
                ReviewerFinding::new(
                    "maintainability".to_string(),
                    ReviewerDisposition::Concern,
                    "Minor cleanup follow-up".to_string(),
                ),
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
    profile.reviewers.clear();
    assert_eq!(profile.validate().unwrap_err(), ReviewProfileError::InsufficientReviewers(0));

    let mut profile = sample_profile();
    profile.reviewers[1].reviewer_id = "safety".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateReviewerId("safety".to_string())
    );

    let mut profile = sample_profile();
    profile.reviewers[0].reviewer_id = " ".to_string();
    assert_eq!(profile.validate().unwrap_err(), ReviewProfileError::MissingReviewerId);

    let mut profile = sample_profile();
    profile.reviewers[0].role = " ".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::MissingReviewerRole("safety".to_string())
    );

    let mut profile = sample_profile();
    profile.reviewers[0].weight = 0;
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::InvalidReviewerWeight("safety".to_string())
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
    profile.adjudication = AdjudicationDefinition { enabled: true, reviewer_id: None };
    assert_eq!(profile.validate().unwrap_err(), ReviewProfileError::MissingAdjudicatorReviewerId);

    let mut profile = sample_profile();
    profile.adjudication =
        AdjudicationDefinition { enabled: true, reviewer_id: Some(" ".to_string()) };
    assert_eq!(profile.validate().unwrap_err(), ReviewProfileError::MissingAdjudicatorReviewerId);

    let mut profile = sample_profile();
    profile.scenarios.push(profile.scenarios[0].clone());
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateScenarioTrigger(ReviewTrigger::PrReady)
    );

    let mut profile = sample_profile();
    profile.scenarios[0].trigger = ReviewTrigger::HighRiskChange;
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::ScenarioTriggerNotConfigured(ReviewTrigger::HighRiskChange)
    );

    let mut profile = sample_profile();
    profile.scenarios[0].findings.clear();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::MissingScenarioFindings(ReviewTrigger::PrReady)
    );

    let mut profile = sample_profile();
    profile.scenarios[0].findings[0].reviewer_id = " ".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::MissingFindingReviewerId("review")
    );

    let mut profile = sample_profile();
    profile.scenarios[0].findings[0].reviewer_id = "unknown".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::UnknownFindingReviewer("unknown".to_string())
    );

    let mut profile = sample_profile();
    profile.scenarios[0].findings[0].summary = " ".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::MissingFindingSummary("safety".to_string())
    );

    let mut profile = sample_profile();
    profile.scenarios[0].findings[1].reviewer_id = "safety".to_string();
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::DuplicateFindingReviewer("safety".to_string())
    );

    let mut profile = sample_profile();
    profile.scenarios[0].adjudication_finding = Some(ReviewerFinding::new(
        "safety".to_string(),
        ReviewerDisposition::Approve,
        "ok".to_string(),
    ));
    assert_eq!(
        profile.validate().unwrap_err(),
        ReviewProfileError::UnexpectedAdjudicationFinding(ReviewTrigger::PrReady)
    );
}

#[test]
fn review_profile_resolver_methods_fetch_correct_data() {
    let profile = sample_profile();
    assert_eq!(profile.reviewer_by_id("safety").unwrap().reviewer_id, "safety");
    assert!(profile.reviewer_by_id("unknown").is_none());

    assert_eq!(
        profile.scenario_for(ReviewTrigger::PrReady).unwrap().trigger,
        ReviewTrigger::PrReady
    );
    assert!(profile.scenario_for(ReviewTrigger::ValidationFailed).is_none());
}
