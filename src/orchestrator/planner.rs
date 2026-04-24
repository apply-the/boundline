use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::domain::plan::Plan;
use crate::domain::step::{Step, StepExecutionResult};
use crate::domain::task::{Task, TaskRunRequest};
use crate::domain::task_context::TaskContext;

pub trait Planner: Send + Sync {
    fn create_initial_plan(
        &self,
        request: &TaskRunRequest,
        context: &TaskContext,
    ) -> Result<Plan, PlanningError>;

    fn replan(
        &self,
        task: &Task,
        failed_step: &Step,
        failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError>;
}

#[derive(Debug, Clone)]
pub struct StaticPlanner {
    initial_plan: Plan,
    replans: Arc<Mutex<Vec<Vec<Step>>>>,
}

impl StaticPlanner {
    pub fn new(initial_plan: Plan) -> Self {
        Self { initial_plan, replans: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn with_replans(initial_plan: Plan, replans: Vec<Vec<Step>>) -> Self {
        Self { initial_plan, replans: Arc::new(Mutex::new(replans)) }
    }
}

impl Planner for StaticPlanner {
    fn create_initial_plan(
        &self,
        _request: &TaskRunRequest,
        _context: &TaskContext,
    ) -> Result<Plan, PlanningError> {
        Ok(self.initial_plan.clone())
    }

    fn replan(
        &self,
        _task: &Task,
        _failed_step: &Step,
        _failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError> {
        let mut replans = self.replans.lock().map_err(|_| {
            PlanningError::Internal("failed to acquire replan queue lock".to_string())
        })?;

        if replans.is_empty() {
            return Err(PlanningError::ReplanUnavailable(
                "no replacement plan is available".to_string(),
            ));
        }

        Ok(replans.remove(0))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanningError {
    #[error("planner returned an invalid plan: {0}")]
    InvalidPlan(String),
    #[error("planner cannot provide a replan: {0}")]
    ReplanUnavailable(String),
    #[error("planner internal error: {0}")]
    Internal(String),
}
