use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::task::{TaskStatus, TerminalReason};

pub fn current_timestamp_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    TaskStarted,
    StepStarted,
    StepCompleted,
    RetryScheduled,
    Replanned,
    TerminalRecorded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub event_id: String,
    pub event_type: TraceEventType,
    pub step_id: Option<String>,
    pub plan_revision: usize,
    pub payload: Value,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub task_id: String,
    pub session_id: String,
    pub goal: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub terminal_status: Option<TaskStatus>,
    pub terminal_reason: Option<TerminalReason>,
    pub events: Vec<TraceEvent>,
    pub trace_location: Option<String>,
}

impl ExecutionTrace {
    pub fn new(
        task_id: impl Into<String>,
        session_id: impl Into<String>,
        goal: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            session_id: session_id.into(),
            goal: goal.into(),
            started_at: current_timestamp_millis(),
            ended_at: None,
            terminal_status: None,
            terminal_reason: None,
            events: Vec::new(),
            trace_location: None,
        }
    }

    pub fn record_event(
        &mut self,
        event_type: TraceEventType,
        step_id: Option<String>,
        plan_revision: usize,
        payload: Value,
    ) {
        self.events.push(TraceEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            step_id,
            plan_revision,
            payload,
            recorded_at: current_timestamp_millis(),
        });
    }

    pub fn finalize(&mut self, terminal_status: TaskStatus, terminal_reason: TerminalReason) {
        self.ended_at = Some(current_timestamp_millis());
        self.terminal_status = Some(terminal_status);
        self.terminal_reason = Some(terminal_reason);
    }

    pub fn set_trace_location(&mut self, trace_location: impl Into<String>) {
        self.trace_location = Some(trace_location.into());
    }
}
