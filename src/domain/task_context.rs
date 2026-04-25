use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::limits::RunLimits;
use crate::domain::step::{ErrorInfo, StepResultSummary};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskContext {
    pub session_id: String,
    pub workspace_ref: String,
    pub constraints: RunLimits,
    pub state: Map<String, Value>,
    pub history_refs: Vec<String>,
    pub last_result: Option<StepResultSummary>,
}

impl TaskContext {
    pub fn new(
        session_id: impl Into<String>,
        workspace_ref: impl Into<String>,
        constraints: RunLimits,
        initial_state: Map<String, Value>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            workspace_ref: workspace_ref.into(),
            constraints,
            state: initial_state,
            history_refs: Vec::new(),
            last_result: None,
        }
    }

    pub fn validate(&self) -> Result<(), TaskContextError> {
        if self.session_id.trim().is_empty() {
            return Err(TaskContextError::MissingSessionId);
        }
        if self.workspace_ref.trim().is_empty() {
            return Err(TaskContextError::MissingWorkspaceRef);
        }

        self.constraints
            .validate()
            .map_err(|error| TaskContextError::InvalidRunLimits(error.to_string()))
    }

    pub fn belongs_to_workspace(&self, workspace_ref: &str) -> bool {
        self.workspace_ref == workspace_ref
    }

    pub fn push_history_ref(&mut self, history_ref: impl Into<String>) {
        self.history_refs.push(history_ref.into());
    }

    pub fn set_last_result(&mut self, last_result: StepResultSummary) {
        self.last_result = Some(last_result);
    }

    pub fn apply_success_output(
        &mut self,
        step_id: &str,
        output: &Value,
        state_patch: Option<&Map<String, Value>>,
    ) {
        self.state.insert("last_step_id".to_string(), Value::String(step_id.to_string()));
        self.state.insert("last_output".to_string(), output.clone());

        if let Some(object) = output.as_object() {
            self.merge_into_state(object);
        }

        if let Some(patch) = state_patch {
            self.merge_into_state(patch);
        }

        self.store_nested("step_outputs", step_id, output.clone());
    }

    pub fn apply_failure_error(&mut self, step_id: &str, error: &ErrorInfo) {
        self.state.insert("last_step_id".to_string(), Value::String(step_id.to_string()));
        self.state
            .insert("last_error".to_string(), serde_json::to_value(error).unwrap_or(Value::Null));
        self.store_nested(
            "step_errors",
            step_id,
            serde_json::to_value(error).unwrap_or(Value::Null),
        );
    }

    fn merge_into_state(&mut self, patch: &Map<String, Value>) {
        for (key, value) in patch {
            self.state.insert(key.clone(), value.clone());
        }
    }

    fn store_nested(&mut self, bucket_key: &str, entry_key: &str, value: Value) {
        if !self.state.contains_key(bucket_key) {
            self.state.insert(bucket_key.to_string(), Value::Object(Map::new()));
        }

        let bucket = self.state.get_mut(bucket_key).expect("bucket key was inserted before access");

        if !bucket.is_object() {
            *bucket = Value::Object(Map::new());
        }

        bucket
            .as_object_mut()
            .expect("bucket value is guaranteed to be an object")
            .insert(entry_key.to_string(), value);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TaskContextError {
    #[error("task context session_id must not be empty")]
    MissingSessionId,
    #[error("task context workspace_ref must not be empty")]
    MissingWorkspaceRef,
    #[error("task context constraints are invalid: {0}")]
    InvalidRunLimits(String),
}
