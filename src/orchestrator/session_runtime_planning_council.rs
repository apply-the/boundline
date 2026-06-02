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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::configuration::{
        ConfigFile, ModelRoute, RoutingConfig, RuntimeKind, ValueSource,
    };
    use crate::domain::stage_council::{
        StageCouncilArtifact, StageCouncilRequest, StageCouncilStatus,
    };

    use super::SessionRuntime;

    const BLOCKED_NEXT_ACTION: &str = "restore an independent reviewer route";
    const BLOCKED_REASON: &str = "reviewer independence collapsed";
    const COUNCIL_ARTIFACT_REF: &str = "evidence/discovery.md";
    const CUSTOM_PLANNING_MODEL: &str = "planning-test-model";
    const STAGE_KEY: &str = "plan:discovery";

    #[test]
    fn planning_council_helpers_cover_workspace_routing_and_blocked_outcome_persistence()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-planning-council")?;
        FileConfigStore::for_workspace(workspace.as_path()).save_local(&ConfigFile {
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: CUSTOM_PLANNING_MODEL.to_string(),
                }),
                ..RoutingConfig::default()
            },
            ..ConfigFile::default()
        })?;

        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let routing = runtime.planning_council_effective_routing();
        assert_eq!(routing.planning.route.model, CUSTOM_PLANNING_MODEL);
        assert_eq!(routing.planning.source, ValueSource::Workspace);

        let request = StageCouncilRequest {
            stage_key: STAGE_KEY.to_string(),
            phase: "planning-discovery".to_string(),
            producer_slot: "planning".to_string(),
            target_refs: vec!["brief.md".to_string()],
            current_artifact_ref: Some("brief.md".to_string()),
            goal: "Clarify scope before planning".to_string(),
            constraints: vec!["use independent review".to_string()],
        };
        let producer_output = StageCouncilArtifact {
            route_slot: "planning".to_string(),
            evidence_ref: COUNCIL_ARTIFACT_REF.to_string(),
            summary: Some("discovery proposal".to_string()),
        };

        let outcome = runtime.stage_council_blocked_outcome(
            &request,
            &producer_output,
            BLOCKED_REASON,
            BLOCKED_NEXT_ACTION,
        )?;

        assert_eq!(outcome.status, StageCouncilStatus::Blocked);
        assert_eq!(outcome.producer_output, producer_output);
        assert_eq!(outcome.next_action, BLOCKED_NEXT_ACTION);
        assert_eq!(
            outcome.revised_output.evidence_ref,
            ".boundline/council/plan-discovery-blocked.md"
        );

        let blocked_artifact = workspace.as_path().join(&outcome.revised_output.evidence_ref);
        let contents = fs::read_to_string(blocked_artifact)?;
        assert_eq!(contents, super::render_stage_council_blocked_note(BLOCKED_REASON));

        Ok(())
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
    }

    struct TestWorkspace {
        path: PathBuf,
    }

    impl TestWorkspace {
        fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
            let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn as_path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
