//! Structured output from tool adapter invocations (feature 013).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured output from a tool adapter invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_id: String,
    pub invocation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    pub duration_ms: u64,
    pub success: bool,
}

impl ToolResult {
    pub fn new(
        tool_id: impl Into<String>,
        invocation: impl Into<String>,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            invocation: invocation.into(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            diff: None,
            duration_ms,
            success,
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = stdout.into();
        self
    }

    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = stderr.into();
        self
    }

    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = Some(diff.into());
        self
    }

    pub fn validate(&self) -> Result<(), ToolResultError> {
        if self.tool_id.trim().is_empty() {
            return Err(ToolResultError::MissingToolId);
        }
        if self.invocation.trim().is_empty() {
            return Err(ToolResultError::MissingInvocation);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ToolResultError {
    #[error("tool_id must not be empty")]
    MissingToolId,
    #[error("invocation must not be empty")]
    MissingInvocation,
}
