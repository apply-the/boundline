use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::governance::{
    AutopilotDecisionRecord, GovernedStagePacket, GovernedStageRecord, PacketReuseBinding,
};
use crate::domain::limits::RunLimits;
use crate::domain::step::{ErrorInfo, StepResultSummary};

pub const LATEST_GOVERNANCE_STAGE_KEY: &str = "latest_governance_stage";
pub const LATEST_GOVERNANCE_PACKET_KEY: &str = "latest_governance_packet";
pub const LATEST_GOVERNANCE_PACKET_REUSE_KEY: &str = "latest_governance_packet_reuse";
pub const LATEST_GOVERNANCE_DECISION_KEY: &str = "latest_governance_decision";

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

    pub fn apply_state_patch(&mut self, state_patch: &Map<String, Value>) {
        self.merge_into_state(state_patch);
    }

    pub fn set_latest_governance_stage(
        &mut self,
        record: &GovernedStageRecord,
    ) -> Result<(), TaskContextError> {
        self.store_serialized(LATEST_GOVERNANCE_STAGE_KEY, record)
    }

    pub fn latest_governance_stage(&self) -> Result<Option<GovernedStageRecord>, TaskContextError> {
        self.load_serialized(LATEST_GOVERNANCE_STAGE_KEY)
    }

    pub fn set_latest_governance_packet(
        &mut self,
        packet: &GovernedStagePacket,
    ) -> Result<(), TaskContextError> {
        self.store_serialized(LATEST_GOVERNANCE_PACKET_KEY, packet)
    }

    pub fn latest_governance_packet(
        &self,
    ) -> Result<Option<GovernedStagePacket>, TaskContextError> {
        self.load_serialized(LATEST_GOVERNANCE_PACKET_KEY)
    }

    pub fn set_latest_governance_packet_reuse(
        &mut self,
        binding: &PacketReuseBinding,
    ) -> Result<(), TaskContextError> {
        self.store_serialized(LATEST_GOVERNANCE_PACKET_REUSE_KEY, binding)
    }

    pub fn latest_governance_packet_reuse(
        &self,
    ) -> Result<Option<PacketReuseBinding>, TaskContextError> {
        self.load_serialized(LATEST_GOVERNANCE_PACKET_REUSE_KEY)
    }

    pub fn set_latest_governance_decision(
        &mut self,
        decision: &AutopilotDecisionRecord,
    ) -> Result<(), TaskContextError> {
        self.store_serialized(LATEST_GOVERNANCE_DECISION_KEY, decision)
    }

    pub fn latest_governance_decision(
        &self,
    ) -> Result<Option<AutopilotDecisionRecord>, TaskContextError> {
        self.load_serialized(LATEST_GOVERNANCE_DECISION_KEY)
    }

    fn merge_into_state(&mut self, patch: &Map<String, Value>) {
        for (key, value) in patch {
            self.state.insert(key.clone(), value.clone());
        }
    }

    fn store_serialized<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), TaskContextError> {
        let serialized = serde_json::to_value(value).map_err(|error| {
            TaskContextError::StateSerializationFailed {
                key: key.to_string(),
                message: error.to_string(),
            }
        })?;
        self.state.insert(key.to_string(), serialized);
        Ok(())
    }

    fn load_serialized<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, TaskContextError> {
        let Some(value) = self.state.get(key) else {
            return Ok(None);
        };
        if value.is_null() {
            return Ok(None);
        }
        serde_json::from_value(value.clone()).map(Some).map_err(|error| {
            TaskContextError::StateDeserializationFailed {
                key: key.to_string(),
                message: error.to_string(),
            }
        })
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
    #[error("task context state serialization failed for '{key}': {message}")]
    StateSerializationFailed { key: String, message: String },
    #[error("task context state deserialization failed for '{key}': {message}")]
    StateDeserializationFailed { key: String, message: String },
}
