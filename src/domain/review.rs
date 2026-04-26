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
            })
            .collect::<Vec<_>>();

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
    #[error(
        "scenario for trigger '{0:?}' cannot define an adjudication finding when adjudication is disabled"
    )]
    UnexpectedAdjudicationFinding(ReviewTrigger),
    #[error("vote resolution requires at least one finding")]
    MissingVoteFindings,
}
