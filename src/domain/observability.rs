//! Structured runtime event vocabulary and metrics collection.
//!
//! Every event emitted during a session is represented by a typed variant
//! in this module. Events carry a per-event-type `schema_version` for
//! consumer compatibility and are serialized to JSONL for export.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION_PLANNING_ANALYSIS: &str = "1.0";
pub const SCHEMA_VERSION_GUARDIAN_FINDING: &str = "1.0";
pub const SCHEMA_VERSION_PROVIDER_CALL: &str = "1.0";
pub const SCHEMA_VERSION_TRACE_COMPACTED: &str = "1.0";
pub const SCHEMA_VERSION_HELP_NEXT_REQUESTED: &str = "1.0";
pub const SCHEMA_VERSION_PHASE_REQUESTED: &str = "1.0";
pub const SCHEMA_VERSION_ROUTE_DECISION: &str = "1.0";
pub const SCHEMA_VERSION_CONTEXT_SELECTION: &str = "1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    PlanningAnalysisCompleted,
    GuardianFindingEmitted,
    ProviderCallCompleted,
    TraceCompacted,
    PhaseRequested,
    RouteDecisionMade,
    ContextSelectionRecorded,
    HelpNextRequested,
}

impl EventType {
    pub const fn all() -> [Self; 8] {
        [
            Self::PlanningAnalysisCompleted,
            Self::GuardianFindingEmitted,
            Self::ProviderCallCompleted,
            Self::TraceCompacted,
            Self::PhaseRequested,
            Self::RouteDecisionMade,
            Self::ContextSelectionRecorded,
            Self::HelpNextRequested,
        ]
    }

    pub const fn type_name(self) -> &'static str {
        match self {
            Self::PlanningAnalysisCompleted => "planning.analysis.completed",
            Self::GuardianFindingEmitted => "guardian.finding.emitted",
            Self::ProviderCallCompleted => "provider.call.completed",
            Self::TraceCompacted => "trace.compacted",
            Self::PhaseRequested => "phase.requested",
            Self::RouteDecisionMade => "route.decision.made",
            Self::ContextSelectionRecorded => "context.selection.recorded",
            Self::HelpNextRequested => "boundline.help_next.requested",
        }
    }

    pub const fn schema_version(self) -> &'static str {
        match self {
            Self::PlanningAnalysisCompleted => SCHEMA_VERSION_PLANNING_ANALYSIS,
            Self::GuardianFindingEmitted => SCHEMA_VERSION_GUARDIAN_FINDING,
            Self::ProviderCallCompleted => SCHEMA_VERSION_PROVIDER_CALL,
            Self::TraceCompacted => SCHEMA_VERSION_TRACE_COMPACTED,
            Self::PhaseRequested => SCHEMA_VERSION_PHASE_REQUESTED,
            Self::RouteDecisionMade => SCHEMA_VERSION_ROUTE_DECISION,
            Self::ContextSelectionRecorded => SCHEMA_VERSION_CONTEXT_SELECTION,
            Self::HelpNextRequested => SCHEMA_VERSION_HELP_NEXT_REQUESTED,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningAnalysisPayload {
    pub state: String,
    pub finding_count: u64,
    pub blocked_finding_count: u64,
    pub coverage_summary: PlanningCoverageSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningCoverageSummary {
    pub outcomes_covered: u64,
    pub outcomes_total: u64,
    pub backlog_slices_covered: u64,
    pub backlog_slices_total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianFindingPayload {
    pub guardian_id: String,
    pub finding_id: Uuid,
    pub severity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCallPayload {
    pub provider_id: String,
    pub call_id: Uuid,
    pub capability: String,
    pub status: String,
    pub latency_ms: u64,
    pub finding_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceCompactedPayload {
    pub policy: String,
    pub source_trace: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRequestedPayload {
    pub phase: String,
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_phase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecisionPayload {
    pub route_id: Uuid,
    pub model_family: String,
    pub reason: String,
    pub fallback_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextSelectionPayload {
    pub total_items: u64,
    pub total_bytes: u64,
    pub omitted_items: u64,
    pub omission_reasons: HashMap<String, u64>,
    pub fidelity_tier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredRuntimeEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub schema_version: String,
    pub timestamp: String,
    pub session_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_ref: Option<String>,
    pub payload: serde_json::Value,
}

impl StructuredRuntimeEvent {
    #[must_use]
    pub fn new(
        event_type: EventType,
        session_id: Uuid,
        trace_ref: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        let timestamp = timestamp_iso8601();
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            schema_version: event_type.schema_version().to_string(),
            timestamp,
            session_id,
            trace_ref,
            payload,
        }
    }
}

fn timestamp_iso8601() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let tod = secs % 86400;
    let h = tod / 3600;
    let m = (tod % 3600) / 60;
    let s = tod % 60;
    let (y, mo, d) = civil_from_days((days + 719_468) as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

const fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z - 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub compaction_count: u64,
    pub compaction_class_distribution: HashMap<String, u64>,
    pub trace_size_before_bytes: u64,
    pub trace_size_after_bytes: u64,
    pub lossy_compaction_count: u64,
    pub preserved_decision_count: u64,
    pub preserved_rejection_count: u64,
    pub context_size_bytes: u64,
    pub context_item_count: u64,
    pub provider_latency_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    pub finding_count: u64,
}

pub const SENSITIVE_FIELD_NAMES: &[&str] =
    &["token", "secret", "password", "key", "credential", "authorization"];

#[must_use]
pub fn is_sensitive_field_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    SENSITIVE_FIELD_NAMES.iter().any(|s| lower.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_type_name_non_empty_and_dotted() {
        for ty in EventType::all() {
            let name = ty.type_name();
            assert!(!name.is_empty());
            assert!(name.contains('.'));
        }
    }

    #[test]
    fn schema_versions_non_empty_and_major_one() {
        for ty in EventType::all() {
            let v = ty.schema_version();
            assert!(!v.is_empty());
            assert!(v.starts_with('1'));
        }
    }

    #[test]
    fn sensitive_field_detection() {
        assert!(is_sensitive_field_name("token"));
        assert!(is_sensitive_field_name("api_secret"));
        assert!(!is_sensitive_field_name("file_name"));
    }

    #[test]
    fn runtime_metrics_default_zeroed() {
        let m = RuntimeMetrics::default();
        assert_eq!(m.compaction_count, 0);
        assert_eq!(m.finding_count, 0);
    }

    #[test]
    fn structured_event_new_populates_schema_version() {
        let event = StructuredRuntimeEvent::new(
            EventType::TraceCompacted,
            Uuid::nil(),
            Some("trace://abc".into()),
            serde_json::json!({"k": "v"}),
        );
        assert_eq!(event.schema_version, SCHEMA_VERSION_TRACE_COMPACTED);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn structured_event_json_roundtrip() {
        let event = StructuredRuntimeEvent::new(
            EventType::GuardianFindingEmitted,
            Uuid::nil(),
            None,
            serde_json::json!({"guardian_id": "rust"}),
        );
        let json = serde_json::to_string(&event).unwrap();
        let parsed: StructuredRuntimeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, EventType::GuardianFindingEmitted);
    }
}
