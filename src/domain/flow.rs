use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::domain::step::Step;

pub const FLOW_METADATA_KEY: &str = "delivery_flow";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowStageDefinition {
    pub id: &'static str,
    pub display_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowDefinition {
    pub name: &'static str,
    pub display_name: &'static str,
    pub stages: &'static [FlowStageDefinition],
}

impl FlowDefinition {
    pub fn stage(&self, stage_index: usize) -> Option<&'static FlowStageDefinition> {
        self.stages.get(stage_index)
    }

    pub fn initial_state(&self) -> SessionFlowState {
        SessionFlowState {
            flow_name: self.name.to_string(),
            current_stage_id: self.stages[0].id.to_string(),
            current_stage_index: 0,
            total_stages: self.stages.len(),
        }
    }
}

const BUG_FIX_STAGES: [FlowStageDefinition; 3] = [
    FlowStageDefinition { id: "investigate", display_name: "Investigate" },
    FlowStageDefinition { id: "implement", display_name: "Implement" },
    FlowStageDefinition { id: "verify", display_name: "Verify" },
];

const CHANGE_STAGES: [FlowStageDefinition; 3] = [
    FlowStageDefinition { id: "understand-change", display_name: "Understand Change" },
    FlowStageDefinition { id: "implement", display_name: "Implement" },
    FlowStageDefinition { id: "verify", display_name: "Verify" },
];

const DELIVERY_STAGES: [FlowStageDefinition; 5] = [
    FlowStageDefinition { id: "requirements", display_name: "Requirements" },
    FlowStageDefinition { id: "system-shaping", display_name: "System Shaping" },
    FlowStageDefinition { id: "architecture", display_name: "Architecture" },
    FlowStageDefinition { id: "backlog", display_name: "Backlog" },
    FlowStageDefinition { id: "implementation", display_name: "Implementation" },
];

const BUILTIN_FLOWS: [FlowDefinition; 3] = [
    FlowDefinition { name: "bug-fix", display_name: "Bug Fix", stages: &BUG_FIX_STAGES },
    FlowDefinition { name: "change", display_name: "Change", stages: &CHANGE_STAGES },
    FlowDefinition { name: "delivery", display_name: "Delivery", stages: &DELIVERY_STAGES },
];

const SUPPORTED_FLOW_NAMES: [&str; 3] = ["bug-fix", "change", "delivery"];

pub fn built_in_flow(name: &str) -> Option<&'static FlowDefinition> {
    BUILTIN_FLOWS.iter().find(|flow| flow.name.eq_ignore_ascii_case(name))
}

pub fn supported_flow_names() -> &'static [&'static str] {
    &SUPPORTED_FLOW_NAMES
}

pub fn supported_flow_names_csv() -> String {
    SUPPORTED_FLOW_NAMES.join(", ")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFlowState {
    pub flow_name: String,
    pub current_stage_id: String,
    pub current_stage_index: usize,
    pub total_stages: usize,
}

impl SessionFlowState {
    pub fn validate(&self) -> Result<(), FlowValidationError> {
        let flow = built_in_flow(&self.flow_name)
            .ok_or_else(|| FlowValidationError::UnknownFlow(self.flow_name.clone()))?;

        if self.total_stages != flow.stages.len() {
            return Err(FlowValidationError::StageCountMismatch {
                flow_name: self.flow_name.clone(),
                expected: flow.stages.len(),
                actual: self.total_stages,
            });
        }

        let expected_stage = flow.stage(self.current_stage_index).ok_or_else(|| {
            FlowValidationError::InvalidStageIndex {
                flow_name: self.flow_name.clone(),
                stage_index: self.current_stage_index,
                total_stages: self.total_stages,
            }
        })?;

        if self.current_stage_id != expected_stage.id {
            return Err(FlowValidationError::StageIdMismatch {
                flow_name: self.flow_name.clone(),
                expected: expected_stage.id.to_string(),
                actual: self.current_stage_id.clone(),
            });
        }

        Ok(())
    }

    pub fn advance(&mut self) -> Result<bool, FlowValidationError> {
        self.validate()?;
        let flow = built_in_flow(&self.flow_name)
            .ok_or_else(|| FlowValidationError::UnknownFlow(self.flow_name.clone()))?;

        let next_stage_index = self.current_stage_index + 1;
        let Some(next_stage) = flow.stage(next_stage_index) else {
            return Ok(false);
        };

        self.current_stage_index = next_stage_index;
        self.current_stage_id = next_stage.id.to_string();
        Ok(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowStepMetadata {
    pub flow_name: String,
    pub stage_id: String,
    pub stage_index: usize,
    pub total_stages: usize,
}

impl FlowStepMetadata {
    pub fn from_step(step: &Step) -> Result<Option<Self>, FlowValidationError> {
        let Some(raw_metadata) = step.input.get(FLOW_METADATA_KEY) else {
            return Ok(None);
        };

        Self::from_value(raw_metadata)
    }

    pub fn from_value(value: &Value) -> Result<Option<Self>, FlowValidationError> {
        if value.is_null() {
            return Ok(None);
        }

        let flow_name = value
            .get("flow_name")
            .and_then(Value::as_str)
            .ok_or(FlowValidationError::MissingMetadataField("flow_name"))?
            .to_string();
        let stage_id = value
            .get("stage_id")
            .and_then(Value::as_str)
            .ok_or(FlowValidationError::MissingMetadataField("stage_id"))?
            .to_string();
        let stage_index = value
            .get("stage_index")
            .and_then(Value::as_u64)
            .ok_or(FlowValidationError::MissingMetadataField("stage_index"))?
            as usize;
        let total_stages = value
            .get("total_stages")
            .and_then(Value::as_u64)
            .ok_or(FlowValidationError::MissingMetadataField("total_stages"))?
            as usize;

        let metadata = Self { flow_name, stage_id, stage_index, total_stages };
        metadata.validate()?;
        Ok(Some(metadata))
    }

    pub fn validate(&self) -> Result<(), FlowValidationError> {
        SessionFlowState {
            flow_name: self.flow_name.clone(),
            current_stage_id: self.stage_id.clone(),
            current_stage_index: self.stage_index,
            total_stages: self.total_stages,
        }
        .validate()
    }
}

pub fn attach_stage_metadata(
    input: Value,
    flow: &FlowDefinition,
    stage_index: usize,
) -> Result<Value, FlowValidationError> {
    let stage = flow.stage(stage_index).ok_or_else(|| FlowValidationError::InvalidStageIndex {
        flow_name: flow.name.to_string(),
        stage_index,
        total_stages: flow.stages.len(),
    })?;

    let mut input_object = input.as_object().cloned().ok_or_else(|| {
        FlowValidationError::NonObjectStepInput { flow_name: flow.name.to_string() }
    })?;
    input_object.insert(
        FLOW_METADATA_KEY.to_string(),
        json!({
            "flow_name": flow.name,
            "stage_id": stage.id,
            "stage_index": stage_index,
            "total_stages": flow.stages.len(),
        }),
    );
    Ok(Value::Object(input_object))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FlowValidationError {
    #[error("unknown flow `{0}`")]
    UnknownFlow(String),
    #[error("flow `{flow_name}` expected stage count {expected}, got {actual}")]
    StageCountMismatch { flow_name: String, expected: usize, actual: usize },
    #[error("flow `{flow_name}` has invalid stage index {stage_index} for total {total_stages}")]
    InvalidStageIndex { flow_name: String, stage_index: usize, total_stages: usize },
    #[error("flow `{flow_name}` expected current stage `{expected}`, got `{actual}`")]
    StageIdMismatch { flow_name: String, expected: String, actual: String },
    #[error("flow step metadata is missing `{0}`")]
    MissingMetadataField(&'static str),
    #[error("flow `{flow_name}` requires object-shaped step input for metadata attachment")]
    NonObjectStepInput { flow_name: String },
}
