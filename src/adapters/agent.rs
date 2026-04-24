use std::sync::Arc;

use crate::domain::step::{StepExecutionRequest, StepExecutionResult};

pub trait AgentAdapter: Send + Sync {
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult;
}

pub type SharedAgentAdapter = Arc<dyn AgentAdapter>;

pub struct FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    handler: F,
}

impl<F> FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> AgentAdapter for FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult {
        (self.handler)(request)
    }
}
