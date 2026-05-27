//! Session-native stage council orchestration records.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilRequest {
    pub stage_key: String,
    pub phase: String,
    pub producer_slot: String,
    #[serde(default)]
    pub target_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_artifact_ref: Option<String>,
    pub goal: String,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilOutcome {
    pub producer_output: StageCouncilArtifact,
    #[serde(default)]
    pub reviewer_findings: Vec<StageCouncilFinding>,
    pub vote_resolution: StageCouncilVoteResolution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjudication: Option<StageCouncilAdjudication>,
    pub revised_output: StageCouncilArtifact,
    pub status: StageCouncilStatus,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilArtifact {
    pub route_slot: String,
    pub evidence_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilFinding {
    pub reviewer_id: String,
    pub effective_route: String,
    pub disposition: StageCouncilFindingDisposition,
    pub summary: String,
    #[serde(default)]
    pub accepted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageCouncilFindingDisposition {
    Approve,
    Concern,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilVoteResolution {
    pub strategy: String,
    #[serde(default)]
    pub accepted_findings: Vec<String>,
    #[serde(default)]
    pub rejected_findings: Vec<String>,
    #[serde(default)]
    pub independent_review: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageCouncilAdjudication {
    pub adjudicator_route: String,
    pub decision: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageCouncilStatus {
    Proceed,
    Blocked,
    Degraded,
}

impl StageCouncilOutcome {
    pub fn validate(&self) -> Result<(), String> {
        if self.next_action.trim().is_empty() {
            return Err("stage council outcome requires next_action".to_string());
        }
        if self.producer_output.evidence_ref.trim().is_empty() {
            return Err("stage council producer output requires evidence_ref".to_string());
        }
        if self.revised_output.evidence_ref.trim().is_empty() {
            return Err("stage council revised output requires evidence_ref".to_string());
        }
        if matches!(self.status, StageCouncilStatus::Proceed)
            && !self.vote_resolution.independent_review
        {
            return Err("proceeding stage council requires independent review".to_string());
        }
        Ok(())
    }

    pub fn context_projection(&self) -> Value {
        serde_json::json!({
            "latest_stage_council_status": self.status,
            "latest_stage_council_producer_ref": self.producer_output.evidence_ref,
            "latest_stage_council_reviser_ref": self.revised_output.evidence_ref,
            "latest_stage_council_independent_review": self.vote_resolution.independent_review,
            "latest_stage_council_next_action": self.next_action,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StageCouncilArtifact, StageCouncilOutcome, StageCouncilStatus, StageCouncilVoteResolution,
    };

    #[test]
    fn stage_council_outcome_requires_independent_review_to_proceed() {
        let outcome = StageCouncilOutcome {
            producer_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: ".boundline/council/producer.md".to_string(),
                summary: None,
            },
            reviewer_findings: Vec::new(),
            vote_resolution: StageCouncilVoteResolution {
                strategy: "majority".to_string(),
                accepted_findings: Vec::new(),
                rejected_findings: Vec::new(),
                independent_review: false,
            },
            adjudication: None,
            revised_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: ".boundline/council/revised.md".to_string(),
                summary: None,
            },
            status: StageCouncilStatus::Proceed,
            next_action: "continue".to_string(),
        };

        assert_eq!(
            outcome.validate().unwrap_err(),
            "proceeding stage council requires independent review"
        );
    }

    fn valid_outcome() -> StageCouncilOutcome {
        StageCouncilOutcome {
            producer_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: ".boundline/council/producer.md".to_string(),
                summary: None,
            },
            reviewer_findings: Vec::new(),
            vote_resolution: StageCouncilVoteResolution {
                strategy: "majority".to_string(),
                accepted_findings: Vec::new(),
                rejected_findings: Vec::new(),
                independent_review: true,
            },
            adjudication: None,
            revised_output: StageCouncilArtifact {
                route_slot: "planning".to_string(),
                evidence_ref: ".boundline/council/revised.md".to_string(),
                summary: None,
            },
            status: StageCouncilStatus::Proceed,
            next_action: "continue".to_string(),
        }
    }

    #[test]
    fn context_projection_emits_all_status_fields() {
        let outcome = valid_outcome();
        assert!(outcome.validate().is_ok());

        let projection = outcome.context_projection();
        assert_eq!(projection["latest_stage_council_next_action"], "continue");
        assert_eq!(
            projection["latest_stage_council_producer_ref"],
            ".boundline/council/producer.md"
        );
        assert_eq!(projection["latest_stage_council_reviser_ref"], ".boundline/council/revised.md");
        assert!(projection["latest_stage_council_independent_review"].as_bool().unwrap());
    }

    #[test]
    fn validate_rejects_missing_next_action_and_evidence_refs() {
        let mut outcome = valid_outcome();
        outcome.next_action = "   ".to_string();
        assert_eq!(outcome.validate().unwrap_err(), "stage council outcome requires next_action");

        let mut outcome = valid_outcome();
        outcome.producer_output.evidence_ref = String::new();
        assert_eq!(
            outcome.validate().unwrap_err(),
            "stage council producer output requires evidence_ref"
        );

        let mut outcome = valid_outcome();
        outcome.revised_output.evidence_ref = String::new();
        assert_eq!(
            outcome.validate().unwrap_err(),
            "stage council revised output requires evidence_ref"
        );
    }
}
