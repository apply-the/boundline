//! Session-scoped audit entries, actor attribution, and inspect-ready audit
//! projections.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::session::date_prefix_from_millis;
use crate::domain::trace::current_timestamp_millis;

const MILLIS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;
const DATE_PREFIX_YEAR_END: usize = 4;
const DATE_PREFIX_MONTH_END: usize = 6;
const DATE_PREFIX_DAY_END: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAuditEntryKind {
    SessionStart,
    SessionEnd,
    SessionStatusChanged,
    FollowThroughProjected,
    TraceEventProjected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAuditActorKind {
    System,
    Human,
    Agent,
    Model,
    Reviewer,
    ReasoningParticipant,
    GovernanceRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAuditPhase {
    Session,
    Goal,
    Plan,
    Run,
    Governance,
    Review,
    Reasoning,
    Recovery,
    Inspect,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAuditOutcomeStatus {
    Started,
    Recorded,
    Projected,
    Succeeded,
    Completed,
    Failed,
    Blocked,
    Awaiting,
    Retried,
    Replanned,
    Skipped,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAuditSourceKind {
    SessionLifecycle,
    TraceEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionAuditIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_user_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_user_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAuditActor {
    pub kind: SessionAuditActorKind,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_slot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participant_routes: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub mixed_routes: bool,
}

impl SessionAuditActor {
    pub fn system(id: impl Into<String>) -> Self {
        Self {
            kind: SessionAuditActorKind::System,
            id: id.into(),
            display_name: None,
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        }
    }

    pub fn display_name_or_id(&self) -> String {
        self.display_name.clone().unwrap_or_else(|| self.id.clone())
    }

    pub fn rollup_key(&self) -> String {
        format!("{}:{}", self.kind.as_str(), self.id)
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl SessionAuditActorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Human => "human",
            Self::Agent => "agent",
            Self::Model => "model",
            Self::Reviewer => "reviewer",
            Self::ReasoningParticipant => "reasoning_participant",
            Self::GovernanceRuntime => "governance_runtime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAuditAlgorithm {
    pub phase: SessionAuditPhase,
    pub family: String,
    pub name: String,
}

impl SessionAuditAlgorithm {
    pub fn new(
        phase: SessionAuditPhase,
        family: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self { phase, family: family.into(), name: name.into() }
    }

    pub fn rollup_key(&self) -> String {
        format!("{}::{}::{}", self.phase.as_str(), self.family, self.name)
    }
}

impl SessionAuditPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Goal => "goal",
            Self::Plan => "plan",
            Self::Run => "run",
            Self::Governance => "governance",
            Self::Review => "review",
            Self::Reasoning => "reasoning",
            Self::Recovery => "recovery",
            Self::Inspect => "inspect",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAuditOutcome {
    pub status: SessionAuditOutcomeStatus,
    pub summary: String,
    #[serde(default)]
    pub blocking: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
}

impl SessionAuditOutcome {
    pub fn new(status: SessionAuditOutcomeStatus, summary: impl Into<String>) -> Self {
        Self { status, summary: summary.into(), blocking: false, next_action: None }
    }

    pub fn rollup_key(&self) -> String {
        self.status.as_str().to_string()
    }
}

impl SessionAuditOutcomeStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Recorded => "recorded",
            Self::Projected => "projected",
            Self::Succeeded => "succeeded",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Awaiting => "awaiting",
            Self::Retried => "retried",
            Self::Replanned => "replanned",
            Self::Skipped => "skipped",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAuditSource {
    pub kind: SessionAuditSourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_revision: Option<usize>,
}

impl SessionAuditSource {
    pub fn session_lifecycle() -> Self {
        Self {
            kind: SessionAuditSourceKind::SessionLifecycle,
            trace_ref: None,
            trace_event_id: None,
            trace_event_type: None,
            step_id: None,
            plan_revision: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAuditEntry {
    pub entry_id: String,
    pub session_id: String,
    pub sequence: u64,
    pub timestamp: String,
    pub timestamp_ms: u64,
    pub entry_kind: SessionAuditEntryKind,
    pub message: String,
    pub session_identity: SessionAuditIdentity,
    pub actor: SessionAuditActor,
    pub algorithm: SessionAuditAlgorithm,
    pub outcome: SessionAuditOutcome,
    pub source: SessionAuditSource,
    pub details: Value,
}

impl SessionAuditEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_id: impl Into<String>,
        sequence: u64,
        entry_kind: SessionAuditEntryKind,
        message: impl Into<String>,
        session_identity: SessionAuditIdentity,
        actor: SessionAuditActor,
        algorithm: SessionAuditAlgorithm,
        outcome: SessionAuditOutcome,
        source: SessionAuditSource,
        details: Value,
    ) -> Self {
        Self::new_with_timestamp(
            session_id,
            sequence,
            current_timestamp_millis(),
            entry_kind,
            message,
            session_identity,
            actor,
            algorithm,
            outcome,
            source,
            details,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_timestamp(
        session_id: impl Into<String>,
        sequence: u64,
        timestamp_ms: u64,
        entry_kind: SessionAuditEntryKind,
        message: impl Into<String>,
        session_identity: SessionAuditIdentity,
        actor: SessionAuditActor,
        algorithm: SessionAuditAlgorithm,
        outcome: SessionAuditOutcome,
        source: SessionAuditSource,
        details: Value,
    ) -> Self {
        Self {
            entry_id: Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            sequence,
            timestamp: format_audit_timestamp(timestamp_ms),
            timestamp_ms,
            entry_kind,
            message: message.into(),
            session_identity,
            actor,
            algorithm,
            outcome,
            source,
            details,
        }
    }

    pub fn event_label(&self) -> String {
        self.source.trace_event_type.clone().unwrap_or_else(|| self.entry_kind.as_str().to_string())
    }
}

impl SessionAuditEntryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::SessionStatusChanged => "session_status_changed",
            Self::FollowThroughProjected => "follow_through_projected",
            Self::TraceEventProjected => "trace_event_projected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAuditRollup {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SessionAuditProjection {
    pub session_id: String,
    #[serde(default)]
    pub entry_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actor_rollups: Vec<SessionAuditRollup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub algorithm_rollups: Vec<SessionAuditRollup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome_rollups: Vec<SessionAuditRollup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<SessionAuditEntry>,
}

impl SessionAuditProjection {
    pub fn from_entries(session_id: impl Into<String>, entries: Vec<SessionAuditEntry>) -> Self {
        let session_id = session_id.into();
        let actor_rollups = build_rollups(entries.iter().map(|entry| entry.actor.rollup_key()));
        let algorithm_rollups =
            build_rollups(entries.iter().map(|entry| entry.algorithm.rollup_key()));
        let outcome_rollups = build_rollups(entries.iter().map(|entry| entry.outcome.rollup_key()));
        Self {
            session_id,
            entry_count: entries.len(),
            actor_rollups,
            algorithm_rollups,
            outcome_rollups,
            entries,
        }
    }
}

fn build_rollups<T>(keys: impl Iterator<Item = T>) -> Vec<SessionAuditRollup>
where
    T: Into<String>,
{
    let mut counts = BTreeMap::new();
    for key in keys {
        let key = key.into();
        let next = counts.get(&key).copied().unwrap_or_default() + 1;
        counts.insert(key, next);
    }
    counts.into_iter().map(|(key, count)| SessionAuditRollup { key, count }).collect()
}

pub fn format_audit_timestamp(timestamp_ms: u64) -> String {
    let date_prefix = date_prefix_from_millis(timestamp_ms);
    let seconds_since_midnight = (timestamp_ms / MILLIS_PER_SECOND) % SECONDS_PER_DAY;
    let hours = seconds_since_midnight / SECONDS_PER_HOUR;
    let minutes = (seconds_since_midnight % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = seconds_since_midnight % SECONDS_PER_MINUTE;

    format!(
        "{}-{}-{} {:02}:{:02}:{:02}",
        &date_prefix[..DATE_PREFIX_YEAR_END],
        &date_prefix[DATE_PREFIX_YEAR_END..DATE_PREFIX_MONTH_END],
        &date_prefix[DATE_PREFIX_MONTH_END..DATE_PREFIX_DAY_END],
        hours,
        minutes,
        seconds,
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        SessionAuditActor, SessionAuditAlgorithm, SessionAuditEntry, SessionAuditEntryKind,
        SessionAuditIdentity, SessionAuditOutcome, SessionAuditOutcomeStatus, SessionAuditPhase,
        SessionAuditProjection, SessionAuditSource, format_audit_timestamp,
    };

    #[test]
    fn format_audit_timestamp_uses_utc_wall_clock_shape() {
        let timestamp = format_audit_timestamp(0);

        assert_eq!(timestamp, "1970-01-01 00:00:00");
    }

    #[test]
    fn projection_rolls_up_actors_algorithms_and_outcomes() {
        let entries = vec![
            SessionAuditEntry::new(
                "session-1",
                1,
                SessionAuditEntryKind::SessionStart,
                "started session",
                SessionAuditIdentity::default(),
                SessionAuditActor::system("boundline"),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session opened"),
                SessionAuditSource::session_lifecycle(),
                json!({"status": "initialized"}),
            ),
            SessionAuditEntry::new(
                "session-1",
                2,
                SessionAuditEntryKind::TraceEventProjected,
                "projected decision event",
                SessionAuditIdentity::default(),
                SessionAuditActor::system("boundline"),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Run,
                    "decision_loop",
                    "run_with_options_and_context",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Projected, "decision mapped"),
                SessionAuditSource {
                    kind: super::SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some(".boundline/sessions/session-1/traces/task-1.json".to_string()),
                    trace_event_id: Some("event-1".to_string()),
                    trace_event_type: Some("decision_created".to_string()),
                    step_id: Some("step-1".to_string()),
                    plan_revision: Some(1),
                },
                json!({"decision_type": "analyze"}),
            ),
        ];

        let projection = SessionAuditProjection::from_entries("session-1", entries);

        assert_eq!(projection.entry_count, 2);
        assert_eq!(projection.actor_rollups.len(), 1);
        assert_eq!(projection.algorithm_rollups.len(), 2);
        assert_eq!(projection.outcome_rollups.len(), 2);
    }
}
