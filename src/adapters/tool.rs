use std::sync::Arc;

use crate::domain::step::{StepExecutionRequest, StepExecutionResult};

pub trait ToolAdapter: Send + Sync {
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult;
}

pub type SharedToolAdapter = Arc<dyn ToolAdapter>;

pub struct FnToolAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    handler: F,
}

impl<F> FnToolAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> ToolAdapter for FnToolAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult {
        (self.handler)(request)
    }
}
