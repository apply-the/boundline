use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::step::Step;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Active,
    Completed,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub revision: usize,
    pub steps: Vec<Step>,
    pub current_step_index: usize,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRevision {
    pub from_revision: usize,
    pub to_revision: usize,
    pub replaced_step_ids: Vec<String>,
    pub added_step_ids: Vec<String>,
}

impl Plan {
    pub fn new(steps: Vec<Step>) -> Result<Self, PlanError> {
        let plan = Self { revision: 0, steps, current_step_index: 0, status: PlanStatus::Active };
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), PlanError> {
        if self.steps.is_empty() {
            return Err(PlanError::NoExecutableSteps);
        }

        if self.current_step_index > self.steps.len() {
            return Err(PlanError::InvalidCurrentStepIndex {
                index: self.current_step_index,
                len: self.steps.len(),
            });
        }

        Ok(())
    }

    pub fn current_step(&self) -> Option<&Step> {
        self.steps.get(self.current_step_index)
    }

    pub fn current_step_mut(&mut self) -> Option<&mut Step> {
        self.steps.get_mut(self.current_step_index)
    }

    pub fn advance(&mut self) {
        if self.current_step_index < self.steps.len() {
            self.current_step_index += 1;
        }

        if self.current_step_index >= self.steps.len() {
            self.status = PlanStatus::Completed;
        }
    }

    pub fn replace_remaining_steps(
        &mut self,
        new_steps: Vec<Step>,
    ) -> Result<PlanRevision, PlanError> {
        let from_revision = self.revision;
        let keep_until = self.current_step_index.saturating_add(1).min(self.steps.len());
        let replaced_step_ids =
            self.steps[keep_until..].iter().map(|step| step.id.clone()).collect::<Vec<_>>();
        let added_step_ids = new_steps.iter().map(|step| step.id.clone()).collect::<Vec<_>>();

        self.steps.truncate(keep_until);
        self.steps.extend(new_steps);
        self.revision += 1;
        self.current_step_index = keep_until;
        self.status = PlanStatus::Active;

        if self.current_step_index >= self.steps.len() {
            return Err(PlanError::NoExecutableSteps);
        }

        Ok(PlanRevision {
            from_revision,
            to_revision: self.revision,
            replaced_step_ids,
            added_step_ids,
        })
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanError {
    #[error("a plan must contain at least one executable step")]
    NoExecutableSteps,
    #[error("current_step_index {index} is out of bounds for plan length {len}")]
    InvalidCurrentStepIndex { index: usize, len: usize },
}
