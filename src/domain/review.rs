use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTrigger {
    ValidationFailed,
    HighRiskChange,
    PrReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VotingBoundaryTrigger {
    Architecture,
    Change,
    Implementation,
    Verification,
    PrReview,
    Refactor,
    SecurityAssessment,
    SupplyChainAnalysis,
    Migration,
    Incident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VotingStageRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingBoundaryInput {
    pub stage: VotingBoundaryTrigger,
    pub risk: VotingStageRisk,
    pub structural_impact: bool,
    pub public_contract_change: bool,
    pub validation_exhausted: bool,
    pub pr_ready: bool,
    pub material_security_finding: bool,
    pub critical_supply_chain_finding: bool,
    pub migration_cutover: bool,
    pub incident_high_blast_radius: bool,
    pub preserved_behavior_evidence: bool,
    pub explicitly_requested: bool,
}

impl VotingBoundaryInput {
    pub const fn low_risk(stage: VotingBoundaryTrigger) -> Self {
        Self {
            stage,
            risk: VotingStageRisk::Low,
            structural_impact: false,
            public_contract_change: false,
            validation_exhausted: false,
            pr_ready: false,
            material_security_finding: false,
            critical_supply_chain_finding: false,
            migration_cutover: false,
            incident_high_blast_radius: false,
            preserved_behavior_evidence: false,
            explicitly_requested: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VotingBoundaryDecision {
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    pub blocks_continuation_until_resolved: bool,
}

const HIGH_RISK_BOUNDARY_STAGES: &[VotingBoundaryTrigger] = &[
    VotingBoundaryTrigger::Change,
    VotingBoundaryTrigger::Implementation,
    VotingBoundaryTrigger::Verification,
    VotingBoundaryTrigger::PrReview,
    VotingBoundaryTrigger::SecurityAssessment,
    VotingBoundaryTrigger::SupplyChainAnalysis,
    VotingBoundaryTrigger::Migration,
    VotingBoundaryTrigger::Incident,
];

pub fn voting_boundary_decision(input: VotingBoundaryInput) -> VotingBoundaryDecision {
    if input.explicitly_requested {
        return voting_required("operator_requested");
    }
    if input.validation_exhausted {
        return voting_required("validation_exhausted");
    }
    if input.pr_ready {
        return voting_required("pr_ready");
    }
    if input.material_security_finding {
        return voting_required("material_security_finding");
    }
    if input.critical_supply_chain_finding {
        return voting_required("critical_supply_chain_finding");
    }
    if input.migration_cutover {
        return voting_required("migration_cutover");
    }
    if input.incident_high_blast_radius {
        return voting_required("incident_high_blast_radius");
    }
    if input.public_contract_change {
        return voting_required("public_contract_change");
    }
    let elevated_risk = matches!(input.risk, VotingStageRisk::High | VotingStageRisk::Critical);
    let high_impact_architecture = input.stage == VotingBoundaryTrigger::Architecture
        && (input.structural_impact || elevated_risk);
    if high_impact_architecture {
        return voting_required("high_impact_architecture");
    }
    let high_risk_boundary = elevated_risk && HIGH_RISK_BOUNDARY_STAGES.contains(&input.stage);
    if high_risk_boundary {
        return voting_required("high_risk_boundary");
    }
    if input.stage == VotingBoundaryTrigger::Refactor
        && input.risk == VotingStageRisk::Low
        && input.preserved_behavior_evidence
    {
        return voting_skipped("low_risk_preserved_behavior");
    }

    voting_skipped("risk_policy_not_triggered")
}

fn voting_required(trigger: &str) -> VotingBoundaryDecision {
    VotingBoundaryDecision {
        required: true,
        trigger: Some(trigger.to_string()),
        skip_reason: None,
        blocks_continuation_until_resolved: true,
    }
}

fn voting_skipped(reason: &str) -> VotingBoundaryDecision {
    VotingBoundaryDecision {
        required: false,
        trigger: None,
        skip_reason: Some(reason.to_string()),
        blocks_continuation_until_resolved: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerDisposition {
    Approve,
    Concern,
    Block,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteStrategy {
    #[default]
    Majority,
    Weighted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteDecision {
    Accepted,
    Rejected,
    NeedsAdjudication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOutcome {
    Accepted,
    Rejected,
    Escalated,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerParticipationStatus {
    Completed,
    Failed,
    Omitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewerDefinition {
    pub reviewer_id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default = "default_reviewer_weight")]
    pub weight: usize,
}

impl ReviewerDefinition {
    pub fn validate(&self) -> Result<(), ReviewProfileError> {
        if self.reviewer_id.trim().is_empty() {
            return Err(ReviewProfileError::MissingReviewerId);
        }

        if self.role.trim().is_empty() {
            return Err(ReviewProfileError::MissingReviewerRole(self.reviewer_id.clone()));
        }

        if self.weight == 0 {
            return Err(ReviewProfileError::InvalidReviewerWeight(self.reviewer_id.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewerFinding {
    pub reviewer_id: String,
    pub disposition: ReviewerDisposition,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ReviewerFinding {
    pub fn validate(
        &self,
        reviewer_ids: &BTreeSet<String>,
        label: &'static str,
    ) -> Result<(), ReviewProfileError> {
        if self.reviewer_id.trim().is_empty() {
            return Err(ReviewProfileError::MissingFindingReviewerId(label));
        }

        if !reviewer_ids.contains(&self.reviewer_id) {
            return Err(ReviewProfileError::UnknownFindingReviewer(self.reviewer_id.clone()));
        }

        if self.summary.trim().is_empty() {
            return Err(ReviewProfileError::MissingFindingSummary(self.reviewer_id.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewScenario {
    pub trigger: ReviewTrigger,
    #[serde(default)]
    pub findings: Vec<ReviewerFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjudication_finding: Option<ReviewerFinding>,
}

impl ReviewScenario {
    pub fn validate(
        &self,
        triggers: &BTreeSet<ReviewTrigger>,
        reviewer_ids: &BTreeSet<String>,
        adjudication: &AdjudicationDefinition,
    ) -> Result<(), ReviewProfileError> {
        if !triggers.contains(&self.trigger) {
            return Err(ReviewProfileError::ScenarioTriggerNotConfigured(self.trigger));
        }

        if self.findings.is_empty() {
            return Err(ReviewProfileError::MissingScenarioFindings(self.trigger));
        }

        let mut seen_reviewer_ids = BTreeSet::new();
        for finding in &self.findings {
            finding.validate(reviewer_ids, "review")?;
            if !seen_reviewer_ids.insert(finding.reviewer_id.clone()) {
                return Err(ReviewProfileError::DuplicateFindingReviewer(
                    finding.reviewer_id.clone(),
                ));
            }
        }

        match (&self.adjudication_finding, adjudication.enabled) {
            (Some(finding), true) => {
                let adjudicator_id = adjudication
                    .reviewer_id
                    .as_ref()
                    .expect("validated adjudication must define reviewer_id")
                    .clone();
                let mut adjudicator_ids = BTreeSet::new();
                adjudicator_ids.insert(adjudicator_id.clone());
                finding.validate(&adjudicator_ids, "adjudication")?;
            }
            (Some(_), false) => {
                return Err(ReviewProfileError::UnexpectedAdjudicationFinding(self.trigger));
            }
            (None, true) | (None, false) => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewerParticipation {
    pub reviewer_id: String,
    pub status: ReviewerParticipationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_route: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VoteRuleDefinition {
    #[serde(default)]
    pub strategy: VoteStrategy,
    #[serde(default)]
    pub reject_on_blocking: bool,
}

impl VoteRuleDefinition {
    pub fn resolve(
        &self,
        reviewers: &[ReviewerDefinition],
        findings: &[ReviewerFinding],
        effective_routes: Option<&BTreeMap<String, String>>,
    ) -> Result<VoteResolution, ReviewProfileError> {
        if findings.is_empty() {
            return Err(ReviewProfileError::MissingVoteFindings);
        }

        let reviewer_weights = reviewers
            .iter()
            .map(|reviewer| (reviewer.reviewer_id.clone(), reviewer.weight))
            .collect::<BTreeMap<_, _>>();
        let reviewer_ids = reviewer_weights.keys().cloned().collect::<BTreeSet<_>>();

        let mut seen_finding_reviewers = BTreeSet::new();
        let mut dispositions = BTreeMap::new();
        let mut approvals = 0;
        let mut concerns = 0;
        let mut blocks = 0;
        let mut total = 0;

        for finding in findings {
            finding.validate(&reviewer_ids, "review")?;
            if !seen_finding_reviewers.insert(finding.reviewer_id.clone()) {
                return Err(ReviewProfileError::DuplicateFindingReviewer(
                    finding.reviewer_id.clone(),
                ));
            }

            let weight = match self.strategy {
                VoteStrategy::Majority => 1,
                VoteStrategy::Weighted => *reviewer_weights
                    .get(&finding.reviewer_id)
                    .expect("validated finding reviewer must exist"),
            };
            total += weight;
            dispositions.insert(finding.reviewer_id.clone(), finding.disposition);
            match finding.disposition {
                ReviewerDisposition::Approve => approvals += weight,
                ReviewerDisposition::Concern => concerns += weight,
                ReviewerDisposition::Block => blocks += weight,
            }
        }

        let participants = reviewers
            .iter()
            .map(|reviewer| ReviewerParticipation {
                reviewer_id: reviewer.reviewer_id.clone(),
                status: if dispositions.contains_key(&reviewer.reviewer_id) {
                    ReviewerParticipationStatus::Completed
                } else {
                    ReviewerParticipationStatus::Omitted
                },
                reason: None,
                effective_route: effective_route_for_reviewer(reviewer, effective_routes),
            })
            .collect::<Vec<_>>();

        ensure_distinct_effective_routes(&participants)?;

        let decision = if self.reject_on_blocking && blocks > 0 {
            VoteDecision::Rejected
        } else if approvals * 2 > total {
            VoteDecision::Accepted
        } else if blocks * 2 > total {
            VoteDecision::Rejected
        } else {
            VoteDecision::NeedsAdjudication
        };

        Ok(VoteResolution {
            strategy: self.strategy,
            participants,
            approvals,
            concerns,
            blocks,
            decision,
        })
    }
}

fn effective_route_for_reviewer(
    reviewer: &ReviewerDefinition,
    effective_routes: Option<&BTreeMap<String, String>>,
) -> Option<String> {
    effective_routes.and_then(|routes| routes.get(&reviewer.reviewer_id).cloned()).or_else(|| {
        reviewer
            .source
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn ensure_distinct_effective_routes(
    participants: &[ReviewerParticipation],
) -> Result<(), ReviewProfileError> {
    let mut seen_routes = BTreeMap::<String, String>::new();

    for participant in participants
        .iter()
        .filter(|participant| participant.status == ReviewerParticipationStatus::Completed)
    {
        let Some(route) =
            participant.effective_route.as_deref().map(str::trim).filter(|value| !value.is_empty())
        else {
            return Err(ReviewProfileError::MissingEffectiveReviewerRoute(
                participant.reviewer_id.clone(),
            ));
        };

        if let Some(first_reviewer) =
            seen_routes.insert(route.to_string(), participant.reviewer_id.clone())
        {
            return Err(ReviewProfileError::DuplicateEffectiveReviewerRoute {
                first_reviewer,
                second_reviewer: participant.reviewer_id.clone(),
                route: route.to_string(),
            });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AdjudicationDefinition {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<String>,
}

impl AdjudicationDefinition {
    pub fn validate(&self, reviewer_ids: &BTreeSet<String>) -> Result<(), ReviewProfileError> {
        if !self.enabled {
            return Ok(());
        }

        let Some(reviewer_id) = self.reviewer_id.as_ref() else {
            return Err(ReviewProfileError::MissingAdjudicatorReviewerId);
        };

        if reviewer_id.trim().is_empty() {
            return Err(ReviewProfileError::MissingAdjudicatorReviewerId);
        }

        if reviewer_ids.contains(reviewer_id) {
            return Err(ReviewProfileError::DuplicateAdjudicatorReviewerId(reviewer_id.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteResolution {
    pub strategy: VoteStrategy,
    #[serde(default)]
    pub participants: Vec<ReviewerParticipation>,
    pub approvals: usize,
    pub concerns: usize,
    pub blocks: usize,
    pub decision: VoteDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewProfile {
    #[serde(default)]
    pub triggers: Vec<ReviewTrigger>,
    #[serde(default)]
    pub reviewers: Vec<ReviewerDefinition>,
    #[serde(default)]
    pub vote_rule: VoteRuleDefinition,
    #[serde(default)]
    pub adjudication: AdjudicationDefinition,
    #[serde(default)]
    pub scenarios: Vec<ReviewScenario>,
}

impl ReviewProfile {
    pub fn reviewer_by_id(&self, reviewer_id: &str) -> Option<&ReviewerDefinition> {
        self.reviewers.iter().find(|reviewer| reviewer.reviewer_id == reviewer_id)
    }

    pub fn scenario_for(&self, trigger: ReviewTrigger) -> Option<&ReviewScenario> {
        self.scenarios.iter().find(|scenario| scenario.trigger == trigger)
    }

    pub fn validate(&self) -> Result<(), ReviewProfileError> {
        if self.triggers.is_empty() {
            return Err(ReviewProfileError::MissingReviewTriggers);
        }

        if self.reviewers.len() < 2 {
            return Err(ReviewProfileError::InsufficientReviewers(self.reviewers.len()));
        }

        let mut trigger_set = BTreeSet::new();
        for trigger in &self.triggers {
            trigger_set.insert(*trigger);
        }

        let mut reviewer_ids = BTreeSet::new();
        for reviewer in &self.reviewers {
            reviewer.validate()?;
            if !reviewer_ids.insert(reviewer.reviewer_id.clone()) {
                return Err(ReviewProfileError::DuplicateReviewerId(reviewer.reviewer_id.clone()));
            }
        }

        self.adjudication.validate(&reviewer_ids)?;

        let mut seen_triggers = BTreeSet::new();
        for scenario in &self.scenarios {
            if !seen_triggers.insert(scenario.trigger) {
                return Err(ReviewProfileError::DuplicateScenarioTrigger(scenario.trigger));
            }
            scenario.validate(&trigger_set, &reviewer_ids, &self.adjudication)?;
        }

        Ok(())
    }
}

const fn default_reviewer_weight() -> usize {
    1
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ReviewProfileError {
    #[error("review profile must define at least one trigger")]
    MissingReviewTriggers,
    #[error("review profile requires at least two reviewers, found {0}")]
    InsufficientReviewers(usize),
    #[error("reviewer id must not be empty")]
    MissingReviewerId,
    #[error("reviewer '{0}' requires a non-empty role")]
    MissingReviewerRole(String),
    #[error("reviewer '{0}' must have a positive weight")]
    InvalidReviewerWeight(String),
    #[error("reviewer '{0}' is duplicated")]
    DuplicateReviewerId(String),
    #[error("scenario trigger '{0:?}' is duplicated")]
    DuplicateScenarioTrigger(ReviewTrigger),
    #[error("scenario trigger '{0:?}' is not configured in review.triggers")]
    ScenarioTriggerNotConfigured(ReviewTrigger),
    #[error("scenario for trigger '{0:?}' must define at least one finding")]
    MissingScenarioFindings(ReviewTrigger),
    #[error("adjudication requires a reviewer_id when enabled")]
    MissingAdjudicatorReviewerId,
    #[error("adjudicator '{0}' must be distinct from configured council reviewers")]
    DuplicateAdjudicatorReviewerId(String),
    #[error("{0} finding requires a reviewer_id")]
    MissingFindingReviewerId(&'static str),
    #[error("finding references unknown reviewer '{0}'")]
    UnknownFindingReviewer(String),
    #[error("finding for reviewer '{0}' requires a non-empty summary")]
    MissingFindingSummary(String),
    #[error("finding reviewer '{0}' is duplicated in one scenario or vote")]
    DuplicateFindingReviewer(String),
    #[error("reviewer '{0}' did not resolve to an effective review route")]
    MissingEffectiveReviewerRoute(String),
    #[error(
        "reviewers '{first_reviewer}' and '{second_reviewer}' resolve to the same effective route '{route}'"
    )]
    DuplicateEffectiveReviewerRoute {
        first_reviewer: String,
        second_reviewer: String,
        route: String,
    },
    #[error(
        "scenario for trigger '{0:?}' cannot define an adjudication finding when adjudication is disabled"
    )]
    UnexpectedAdjudicationFinding(ReviewTrigger),
    #[error("vote resolution requires at least one finding")]
    MissingVoteFindings,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;

    fn sample_reviewers() -> Vec<ReviewerDefinition> {
        vec![
            ReviewerDefinition {
                reviewer_id: "safety".to_string(),
                role: "Safety".to_string(),
                source: Some("copilot/gpt-5.5".to_string()),
                weight: 1,
            },
            ReviewerDefinition {
                reviewer_id: "maintainability".to_string(),
                role: "Maintainability".to_string(),
                source: Some("claude/sonnet-4.6".to_string()),
                weight: 1,
            },
        ]
    }

    fn sample_findings() -> Vec<ReviewerFinding> {
        vec![
            ReviewerFinding {
                reviewer_id: "safety".to_string(),
                disposition: ReviewerDisposition::Approve,
                summary: "No blocking issues".to_string(),
                details: None,
            },
            ReviewerFinding {
                reviewer_id: "maintainability".to_string(),
                disposition: ReviewerDisposition::Approve,
                summary: "Looks maintainable".to_string(),
                details: None,
            },
        ]
    }

    #[test]
    fn review_scenario_validates_enabled_adjudication_finding() {
        let triggers = BTreeSet::from([ReviewTrigger::PrReady]);
        let reviewer_ids = BTreeSet::from(["safety".to_string(), "maintainability".to_string()]);
        let adjudication =
            AdjudicationDefinition { enabled: true, reviewer_id: Some("arbiter".to_string()) };
        let scenario = ReviewScenario {
            trigger: ReviewTrigger::PrReady,
            findings: sample_findings(),
            adjudication_finding: Some(ReviewerFinding {
                reviewer_id: "arbiter".to_string(),
                disposition: ReviewerDisposition::Approve,
                summary: "Break the tie".to_string(),
                details: None,
            }),
        };

        assert!(scenario.validate(&triggers, &reviewer_ids, &adjudication).is_ok());
    }

    #[test]
    fn resolve_rejects_missing_vote_findings() {
        let error =
            VoteRuleDefinition::default().resolve(&sample_reviewers(), &[], None).unwrap_err();

        assert_eq!(error, ReviewProfileError::MissingVoteFindings);
    }

    #[test]
    fn resolve_uses_reviewer_source_routes_and_rejects_block_majority() {
        let reviewers = vec![
            ReviewerDefinition {
                reviewer_id: "safety".to_string(),
                role: "Safety".to_string(),
                source: Some("copilot/gpt-5.5".to_string()),
                weight: 1,
            },
            ReviewerDefinition {
                reviewer_id: "maintainability".to_string(),
                role: "Maintainability".to_string(),
                source: Some("claude/sonnet-4.6".to_string()),
                weight: 1,
            },
            ReviewerDefinition {
                reviewer_id: "ux".to_string(),
                role: "UX".to_string(),
                source: Some("gemini/gemini-2.5-pro".to_string()),
                weight: 1,
            },
        ];
        let findings = vec![
            ReviewerFinding {
                reviewer_id: "safety".to_string(),
                disposition: ReviewerDisposition::Block,
                summary: "Unsafe".to_string(),
                details: None,
            },
            ReviewerFinding {
                reviewer_id: "maintainability".to_string(),
                disposition: ReviewerDisposition::Block,
                summary: "Hard to maintain".to_string(),
                details: None,
            },
            ReviewerFinding {
                reviewer_id: "ux".to_string(),
                disposition: ReviewerDisposition::Approve,
                summary: "Looks fine from UX".to_string(),
                details: None,
            },
        ];

        let resolution =
            VoteRuleDefinition::default().resolve(&reviewers, &findings, None).unwrap();

        assert_eq!(resolution.decision, VoteDecision::Rejected);
        assert_eq!(resolution.participants[0].effective_route.as_deref(), Some("copilot/gpt-5.5"));
        assert_eq!(
            resolution.participants[1].effective_route.as_deref(),
            Some("claude/sonnet-4.6")
        );
        assert_eq!(
            resolution.participants[2].effective_route.as_deref(),
            Some("gemini/gemini-2.5-pro")
        );
    }

    #[test]
    fn resolve_requires_effective_route_for_completed_reviewers() {
        let reviewers = vec![
            ReviewerDefinition {
                reviewer_id: "safety".to_string(),
                role: "Safety".to_string(),
                source: None,
                weight: 1,
            },
            ReviewerDefinition {
                reviewer_id: "maintainability".to_string(),
                role: "Maintainability".to_string(),
                source: None,
                weight: 1,
            },
        ];
        let findings = vec![ReviewerFinding {
            reviewer_id: "safety".to_string(),
            disposition: ReviewerDisposition::Approve,
            summary: "No blockers".to_string(),
            details: None,
        }];

        let error = VoteRuleDefinition::default().resolve(&reviewers, &findings, None).unwrap_err();

        assert_eq!(error, ReviewProfileError::MissingEffectiveReviewerRoute("safety".to_string()));
    }

    #[test]
    fn adjudication_definition_accepts_distinct_reviewer_id() {
        let reviewer_ids = BTreeSet::from(["safety".to_string(), "maintainability".to_string()]);
        let adjudication =
            AdjudicationDefinition { enabled: true, reviewer_id: Some("arbiter".to_string()) };

        assert!(adjudication.validate(&reviewer_ids).is_ok());
    }

    #[test]
    fn reviewer_definition_deserializes_default_weight() {
        let reviewer: ReviewerDefinition = serde_json::from_value(json!({
            "reviewer_id": "safety",
            "role": "Safety"
        }))
        .unwrap();

        assert_eq!(reviewer.weight, 1);
    }

    #[test]
    fn resolve_includes_effective_routes_for_completed_reviewers() {
        let routes = BTreeMap::from([
            ("safety".to_string(), "copilot/gpt-5.5".to_string()),
            ("maintainability".to_string(), "claude/sonnet-4.6".to_string()),
        ]);

        let resolution = VoteRuleDefinition::default()
            .resolve(&sample_reviewers(), &sample_findings(), Some(&routes))
            .unwrap();

        assert_eq!(resolution.decision, VoteDecision::Accepted);
        assert_eq!(resolution.participants.len(), 2);
        assert_eq!(resolution.participants[0].effective_route.as_deref(), Some("copilot/gpt-5.5"));
        assert_eq!(
            resolution.participants[1].effective_route.as_deref(),
            Some("claude/sonnet-4.6")
        );
    }

    #[test]
    fn resolve_rejects_duplicate_effective_routes() {
        let routes = BTreeMap::from([
            ("safety".to_string(), "copilot/gpt-5.5".to_string()),
            ("maintainability".to_string(), "copilot/gpt-5.5".to_string()),
        ]);

        let error = VoteRuleDefinition::default()
            .resolve(&sample_reviewers(), &sample_findings(), Some(&routes))
            .unwrap_err();

        assert!(matches!(
            error,
            ReviewProfileError::DuplicateEffectiveReviewerRoute {
                first_reviewer,
                second_reviewer,
                route,
            } if first_reviewer == "safety"
                && second_reviewer == "maintainability"
                && route == "copilot/gpt-5.5"
        ));
    }

    #[test]
    fn voting_boundary_input_low_risk_and_skip_paths_are_stable() {
        let input = VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Refactor);

        assert_eq!(input.stage, VotingBoundaryTrigger::Refactor);
        assert_eq!(input.risk, VotingStageRisk::Low);
        assert!(!input.structural_impact);
        assert!(!input.public_contract_change);
        assert!(!input.validation_exhausted);
        assert!(!input.pr_ready);
        assert!(!input.material_security_finding);
        assert!(!input.critical_supply_chain_finding);
        assert!(!input.migration_cutover);
        assert!(!input.incident_high_blast_radius);
        assert!(!input.preserved_behavior_evidence);
        assert!(!input.explicitly_requested);

        let skipped = voting_boundary_decision(VotingBoundaryInput {
            preserved_behavior_evidence: true,
            ..input
        });
        assert!(!skipped.required);
        assert_eq!(skipped.skip_reason.as_deref(), Some("low_risk_preserved_behavior"));
        assert!(!skipped.blocks_continuation_until_resolved);

        let default_skip = voting_boundary_decision(VotingBoundaryInput::low_risk(
            VotingBoundaryTrigger::Verification,
        ));
        assert!(!default_skip.required);
        assert_eq!(default_skip.skip_reason.as_deref(), Some("risk_policy_not_triggered"));
    }

    #[test]
    fn voting_boundary_decision_requires_expected_escalation_triggers() {
        for (input, expected_trigger) in [
            (
                VotingBoundaryInput {
                    explicitly_requested: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Change)
                },
                "operator_requested",
            ),
            (
                VotingBoundaryInput {
                    validation_exhausted: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Change)
                },
                "validation_exhausted",
            ),
            (
                VotingBoundaryInput {
                    pr_ready: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::PrReview)
                },
                "pr_ready",
            ),
            (
                VotingBoundaryInput {
                    public_contract_change: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Implementation)
                },
                "public_contract_change",
            ),
            (
                VotingBoundaryInput {
                    structural_impact: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Architecture)
                },
                "high_impact_architecture",
            ),
            (
                VotingBoundaryInput {
                    risk: VotingStageRisk::Critical,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Change)
                },
                "high_risk_boundary",
            ),
            (
                VotingBoundaryInput {
                    critical_supply_chain_finding: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::SupplyChainAnalysis)
                },
                "critical_supply_chain_finding",
            ),
            (
                VotingBoundaryInput {
                    incident_high_blast_radius: true,
                    ..VotingBoundaryInput::low_risk(VotingBoundaryTrigger::Incident)
                },
                "incident_high_blast_radius",
            ),
        ] {
            let decision = voting_boundary_decision(input);
            assert!(decision.required, "expected escalation for {expected_trigger}");
            assert_eq!(decision.trigger.as_deref(), Some(expected_trigger));
            assert!(decision.blocks_continuation_until_resolved);
        }
    }
}
