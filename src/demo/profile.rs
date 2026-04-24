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
}
