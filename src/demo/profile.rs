use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::domain::limits::RunLimits;
use crate::domain::plan::Plan;
use crate::domain::step::{Step, StepError, StepKind};
use crate::domain::task::TaskRunRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DemoStepOutline {
    pub step_id: String,
    pub step_kind: StepKind,
    pub target_name: Option<String>,
    pub input: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DemoRunProfile {
    pub name: String,
    pub goal: String,
    pub initial_input: Value,
    pub step_outline: Vec<DemoStepOutline>,
    pub recovery_trigger_step: String,
    pub limits: RunLimits,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replan_steps: Vec<Vec<DemoStepOutline>>,
}

impl DemoRunProfile {
    pub fn guided_demo() -> Self {
        let mut profile = Self::build(
            "guided_demo",
            "Walk through a deterministic bounded delivery flow",
            json!({
                "mode": "demo",
                "ticket": "DEMO-001",
            }),
        );
        profile.set_step_flag("code", "force_retry", true);
        profile
    }

    pub fn default_run(goal: impl Into<String>) -> Self {
        let goal = goal.into();
        let lowered_goal = goal.to_ascii_lowercase();
        let mut profile = Self::build(
            "default_developer_flow",
            goal.clone(),
            json!({
                "mode": "custom",
                "goal": goal,
            }),
        );
        profile.set_step_flag(
            "code",
            "force_retry",
            lowered_goal.contains("retry") || lowered_goal.contains("recover"),
        );
        profile.set_step_flag("code", "force_replan", lowered_goal.contains("replan"));
        profile.set_step_flag(
            "verify",
            "force_terminal_failure",
            lowered_goal.contains("fail") || lowered_goal.contains("non-success"),
        );
        profile
    }

    /// Build the profile used by `synod run-demo`: forces one retry on the
    /// `code` step and one replan triggered by the `verify` step, then
    /// converges to a real fix written to disk.
    pub fn test_fix_loop(workspace: &crate::demo::workspace::DemoWorkspace) -> Self {
        let goal = "Fix the seeded failing test in the demo workspace".to_string();
        let target_file = workspace.target_file.to_string_lossy().into_owned();
        let bug_marker = workspace.bug_marker;
        let partial_fix = crate::demo::workspace::PARTIAL_FIX_SOURCE;
        let fixed_content = workspace.fixed_content;

        let mut profile = Self::build(
            "test_fix_loop",
            goal.clone(),
            json!({
                "mode": "test_fix_loop",
                "workspace_root": workspace.root.to_string_lossy(),
            }),
        );
        // Bump the step budget to fit one retry on `code` plus the inserted
        // analyze/code/verify after the replan from `verify`.
        profile.limits.max_steps = 8;
        // Code step: first attempt fails recoverable (retry), second attempt
        // writes the partial fix so the verify step can request a replan.
        profile.set_step_flag("code", "force_retry", true);
        profile.set_step_input_field("code", "target_file", json!(target_file));
        profile.set_step_input_field("code", "fixed_content", json!(partial_fix));
        // Verify step: first attempt forces a replan; second attempt reads the
        // file and reports success once the bug marker is gone.
        profile.set_step_flag("verify", "force_replan", true);
        profile.set_step_input_field("verify", "target_file", json!(target_file));
        profile.set_step_input_field("verify", "bug_marker", json!(bug_marker));
        profile.replan_steps =
            vec![Self::test_fix_loop_replan_steps(&target_file, bug_marker, fixed_content)];
        profile
    }

    fn test_fix_loop_replan_steps(
        target_file: &str,
        bug_marker: &str,
        fixed_content: &str,
    ) -> Vec<DemoStepOutline> {
        vec![
            DemoStepOutline {
                step_id: "analyze#replan-1".to_string(),
                step_kind: StepKind::Agent,
                target_name: Some("analyzer".to_string()),
                input: json!({"phase": "analyze", "goal": "Re-analyze after partial fix"}),
            },
            DemoStepOutline {
                step_id: "code#replan-1".to_string(),
                step_kind: StepKind::Agent,
                target_name: Some("coder".to_string()),
                input: json!({
                    "phase": "code",
                    "goal": "Apply the full fix",
                    "target_file": target_file,
                    "fixed_content": fixed_content,
                }),
            },
            DemoStepOutline {
                step_id: "verify#replan-1".to_string(),
                step_kind: StepKind::Tool,
                target_name: Some("tester".to_string()),
                input: json!({
                    "phase": "verify",
                    "goal": "Verify the full fix",
                    "target_file": target_file,
                    "bug_marker": bug_marker,
                }),
            },
        ]
    }

    fn build(name: impl Into<String>, goal: impl Into<String>, initial_input: Value) -> Self {
        let goal = goal.into();

        Self {
            name: name.into(),
            goal: goal.clone(),
            initial_input,
            step_outline: vec![
                DemoStepOutline {
                    step_id: "analyze".to_string(),
                    step_kind: StepKind::Agent,
                    target_name: Some("analyzer".to_string()),
                    input: json!({"phase": "analyze", "goal": goal}),
                },
                DemoStepOutline {
                    step_id: "code".to_string(),
                    step_kind: StepKind::Agent,
                    target_name: Some("coder".to_string()),
                    input: json!({"phase": "code", "goal": goal}),
                },
                DemoStepOutline {
                    step_id: "verify".to_string(),
                    step_kind: StepKind::Tool,
                    target_name: Some("tester".to_string()),
                    input: json!({"phase": "verify", "goal": goal}),
                },
            ],
            recovery_trigger_step: "code".to_string(),
            limits: RunLimits {
                max_steps: 6,
                max_retries: 1,
                max_replans: 1,
                ..RunLimits::default()
            },
            replan_steps: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), DemoProfileError> {
        if self.name.trim().is_empty() {
            return Err(DemoProfileError::MissingName);
        }
        if self.goal.trim().is_empty() {
            return Err(DemoProfileError::MissingGoal);
        }
        if self.step_outline.is_empty() {
            return Err(DemoProfileError::MissingStepOutline);
        }
        if !self.step_outline.iter().any(|step| step.step_id == self.recovery_trigger_step) {
            return Err(DemoProfileError::MissingRecoveryTriggerStep(
                self.recovery_trigger_step.clone(),
            ));
        }

        self.limits
            .validate()
            .map_err(|error| DemoProfileError::InvalidRunLimits(error.to_string()))?;

        for step in &self.step_outline {
            let _ = self.build_step(step)?;
        }

        Ok(())
    }

    pub fn to_plan(&self) -> Result<Plan, DemoProfileError> {
        self.validate()?;
        let steps = self
            .step_outline
            .iter()
            .map(|step| self.build_step(step))
            .collect::<Result<Vec<_>, _>>()?;
        Plan::new(steps).map_err(DemoProfileError::InvalidPlan)
    }

    pub fn to_task_request(
        &self,
        workspace_ref: impl Into<String>,
        session_id: impl Into<String>,
    ) -> TaskRunRequest {
        TaskRunRequest {
            goal: self.goal.clone(),
            input: self.initial_input.clone(),
            session_id: session_id.into(),
            workspace_ref: workspace_ref.into(),
            limits: self.limits.clone(),
            initial_context: None,
        }
    }

    fn build_step(&self, step: &DemoStepOutline) -> Result<Step, DemoProfileError> {
        match step.step_kind {
            StepKind::Agent => Step::agent(
                step.step_id.clone(),
                step.target_name.clone().unwrap_or_default(),
                step.input.clone(),
            )
            .map_err(DemoProfileError::InvalidStep),
            StepKind::Tool => Step::tool(
                step.step_id.clone(),
                step.target_name.clone().unwrap_or_default(),
                step.input.clone(),
            )
            .map_err(DemoProfileError::InvalidStep),
            StepKind::Decision => Step::decision(step.step_id.clone(), step.input.clone())
                .map_err(DemoProfileError::InvalidStep),
        }
    }

    fn set_step_flag(&mut self, step_id: &str, key: &str, value: bool) {
        if let Some(step) = self.step_outline.iter_mut().find(|step| step.step_id == step_id)
            && let Some(input) = step.input.as_object_mut()
        {
            input.insert(key.to_string(), json!(value));
        }
    }

    fn set_step_input_field(&mut self, step_id: &str, key: &str, value: Value) {
        if let Some(step) = self.step_outline.iter_mut().find(|step| step.step_id == step_id)
            && let Some(input) = step.input.as_object_mut()
        {
            input.insert(key.to_string(), value);
        }
    }

    /// Build the queued replacement step lists used by `StaticPlanner::with_replans`.
    pub fn to_replan_steps(&self) -> Result<Vec<Vec<Step>>, DemoProfileError> {
        self.replan_steps
            .iter()
            .map(|outlines| {
                outlines
                    .iter()
                    .map(|outline| self.build_step(outline))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum DemoProfileError {
    #[error("demo profile requires a stable name")]
    MissingName,
    #[error("demo profile requires a non-empty goal")]
    MissingGoal,
    #[error("demo profile requires at least one executable step")]
    MissingStepOutline,
    #[error("demo profile recovery trigger step '{0}' is not present in the step outline")]
    MissingRecoveryTriggerStep(String),
    #[error("demo profile run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("demo profile step is invalid: {0}")]
    InvalidStep(StepError),
    #[error("demo profile cannot build a plan: {0}")]
    InvalidPlan(crate::domain::plan::PlanError),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DemoProfileError, DemoRunProfile};

    #[test]
    fn demo_profile_builds_a_deterministic_guided_profile() {
        let profile = DemoRunProfile::guided_demo();

        assert_eq!(profile.name, "guided_demo");
        assert_eq!(profile.recovery_trigger_step, "code");
        assert_eq!(profile.step_outline.len(), 3);
        assert!(profile.validate().is_ok());
        assert!(profile.to_plan().is_ok());
    }

    #[test]
    fn demo_profile_rejects_missing_recovery_trigger_steps() {
        let mut profile = DemoRunProfile::guided_demo();
        profile.recovery_trigger_step = "missing".to_string();

        assert!(matches!(
            profile.validate(),
            Err(DemoProfileError::MissingRecoveryTriggerStep(step_id)) if step_id == "missing"
        ));
    }

    #[test]
    fn default_run_profile_uses_goal_keywords_to_configure_failure_modes() {
        let profile = DemoRunProfile::default_run("Force a non-success failure and replan");
        let code = profile.step_outline.iter().find(|step| step.step_id == "code").unwrap();
        let verify = profile.step_outline.iter().find(|step| step.step_id == "verify").unwrap();

        assert_eq!(code.input["force_replan"], json!(true));
        assert_eq!(verify.input["force_terminal_failure"], json!(true));
    }

    #[test]
    fn test_fix_loop_profile_seeds_retry_and_replan_inputs() {
        use crate::demo::workspace::{BUG_MARKER, DemoWorkspace};
        use std::path::PathBuf;

        let ws = DemoWorkspace {
            root: PathBuf::from("/tmp/.synod/demo-workspace"),
            target_file: PathBuf::from("/tmp/.synod/demo-workspace/src/buggy.rs"),
            test_file: PathBuf::from("/tmp/.synod/demo-workspace/tests/buggy_test.rs"),
            bug_marker: BUG_MARKER,
            fixed_content: crate::demo::workspace::FIXED_SOURCE,
        };
        let profile = DemoRunProfile::test_fix_loop(&ws);
        assert_eq!(profile.name, "test_fix_loop");
        assert!(profile.validate().is_ok());
        let code = profile.step_outline.iter().find(|s| s.step_id == "code").unwrap();
        let verify = profile.step_outline.iter().find(|s| s.step_id == "verify").unwrap();
        assert_eq!(code.input["force_retry"], json!(true));
        assert!(code.input.get("target_file").is_some());
        assert!(code.input.get("fixed_content").is_some());
        assert_eq!(verify.input["force_replan"], json!(true));
        assert_eq!(verify.input["bug_marker"], json!(BUG_MARKER));
        let replans = profile.to_replan_steps().unwrap();
        assert_eq!(replans.len(), 1);
        assert_eq!(replans[0].len(), 3);
    }
}
