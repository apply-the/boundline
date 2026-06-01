use std::fs;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::domain::configuration::{EffectiveRouting, RoutingOverrides, resolve_effective_routing};
use crate::domain::stage_council::{
    StageCouncilArtifact, StageCouncilOutcome, StageCouncilRequest, StageCouncilStatus,
    StageCouncilVoteResolution,
};

use super::{SessionRuntime, SessionRuntimeError, render_stage_council_blocked_note};

impl SessionRuntime {
    pub(super) fn planning_council_effective_routing(&self) -> EffectiveRouting {
        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
            .load()
            .ok()
            .flatten()
            .map(|config| config.routing);
        let global_routing = FileConfigStore::global_routing().ok().flatten();
        resolve_effective_routing(
            &RoutingOverrides::default(),
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        )
    }

    pub(super) fn stage_council_blocked_outcome(
        &self,
        request: &StageCouncilRequest,
        producer_output: &StageCouncilArtifact,
        reason: &str,
        next_action: &str,
    ) -> Result<StageCouncilOutcome, SessionRuntimeError> {
        let revised_ref = self.write_stage_council_artifact(
            request,
            "blocked",
            &render_stage_council_blocked_note(reason),
        )?;
        let outcome = StageCouncilOutcome {
            producer_output: producer_output.clone(),
            reviewer_findings: Vec::new(),
            vote_resolution: StageCouncilVoteResolution {
                strategy: "bounded_majority".to_string(),
                accepted_findings: Vec::new(),
                rejected_findings: Vec::new(),
                independent_review: false,
            },
            adjudication: None,
            revised_output: StageCouncilArtifact {
                route_slot: request.producer_slot.clone(),
                evidence_ref: revised_ref,
                summary: Some("stage council blocked planning discovery".to_string()),
            },
            status: StageCouncilStatus::Blocked,
            next_action: next_action.to_string(),
        };
        outcome.validate().map_err(SessionRuntimeError::ExecutionInvariant)?;
        Ok(outcome)
    }

    pub(super) fn write_stage_council_artifact(
        &self,
        request: &StageCouncilRequest,
        suffix: &str,
        contents: &str,
    ) -> Result<String, SessionRuntimeError> {
        let relative_ref =
            format!(".boundline/council/{}-{suffix}.md", request.stage_key.replace(':', "-"));
        let artifact_path = self.workspace_ref.join(&relative_ref);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                SessionRuntimeError::GoalPlan(format!(
                    "failed to create council artifact directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        fs::write(&artifact_path, contents).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write stage council artifact {}: {error}",
                artifact_path.display()
            ))
        })?;
        Ok(relative_ref)
    }
}
