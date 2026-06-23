//! Browser capability provider domain model for Boundline.
//!
//! This module owns the domain types for browser validation steps, evidence
//! packets, findings, artifact references, interaction scripts, readiness
//! locators, and retryability hints. The browser provider communicates over
//! the existing external capability provider protocol (S10) via JSON over
//! stdio and produces session-scoped evidence packets with normalized
//! findings.
//!
//! Browser automation MUST NOT be embedded in the Boundline core runtime —
//! the provider is an external binary registered and activated through
//! configuration.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// StepStatus (T004)
// ---------------------------------------------------------------------------

/// Primary outcome status of a browser validation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Completed,
    Failed,
    TimedOut,
    ProviderError,
    Cancelled,
    QueueTimeout,
    QueueFull,
}

// ---------------------------------------------------------------------------
// FindingSeverity + FindingKind (T005)
// ---------------------------------------------------------------------------

/// Severity level of a browser validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Blocking,
    Warning,
    Info,
}

/// Category of a browser validation finding (12 variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    ConsoleError,
    AccessibilityViolation,
    VisualDiffDetected,
    NetworkAccessViolation,
    PageLoadTimeout,
    BrowserReadinessTimeout,
    BaselineCreated,
    ScriptStepFailed,
    AccessibilityScanFailed,
    BrowserConcurrencyTimeout,
    BrowserQueueFull,
    CancelledBeforeStart,
}

// ---------------------------------------------------------------------------
// ArtifactKind + RetentionClass (T006)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Screenshot,
    ConsoleLog,
    NetworkLog,
    DomSnapshot,
    AccessibilityOutput,
    EvidencePacket,
    DiffImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionClass {
    RequiredEvidence,
    Diagnostic,
    Verbose,
    Ephemeral,
}

// ---------------------------------------------------------------------------
// ArtifactReference (T007)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub kind: ArtifactKind,
    pub relative_path: String,
    pub content_hash: String,
    pub media_type: String,
    pub byte_size: u64,
    pub created_at: String,
    pub retention_class: RetentionClass,
    pub validation_run_id: String,
}

// ---------------------------------------------------------------------------
// BrowserFinding (T008)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserFinding {
    pub kind: FindingKind,
    pub severity: FindingSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryability: Option<RetryabilityHint>,
    #[serde(default)]
    pub confirmed_intermittent: bool,
}

// ---------------------------------------------------------------------------
// RetryabilityHint + enums (T009)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryabilityLevel {
    NotIndicated,
    Possible,
    Likely,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryabilityCategory {
    NetworkTransient,
    ResourceContention,
    BrowserProcessFailure,
    ProviderUnavailable,
    QueueTimeout,
    EnvironmentStartupDelay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryabilityHint {
    pub level: RetryabilityLevel,
    pub category: RetryabilityCategory,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing_context: Option<StepTiming>,
}

// ---------------------------------------------------------------------------
// StepTiming (T010)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepTiming {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_wait_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness_wait_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_execution_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility_ms: Option<u64>,
    pub total_ms: u64,
}

// ---------------------------------------------------------------------------
// LocatorType + LocatorState (T011)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocatorType {
    CssSelector,
    TestId,
    AccessibleRole,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocatorState {
    Attached,
    Visible,
    Hidden,
    Detached,
}

// ---------------------------------------------------------------------------
// ReadinessLocator (T012)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessLocator {
    #[serde(rename = "type")]
    pub locator_type: LocatorType,
    pub value: String,
    pub state: LocatorState,
    pub timeout_seconds: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stabilization_delay_ms: Option<u32>,
}

// ---------------------------------------------------------------------------
// BrowserAction (T013)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BrowserAction {
    Navigate {
        url: String,
        #[serde(default = "default_script_step_timeout")]
        timeout_seconds: u32,
    },
    Click {
        selector: String,
        #[serde(default = "default_script_step_timeout")]
        timeout_seconds: u32,
    },
    Type {
        selector: String,
        text: String,
        #[serde(default = "default_script_step_timeout")]
        timeout_seconds: u32,
    },
    Wait {
        selector_or_ms: String,
    },
    Screenshot {
        label: String,
    },
}

const fn default_script_step_timeout() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// BrowserEvidencePacket (T014)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserEvidencePacket {
    pub validation_run_id: String,
    pub provider_id: String,
    pub status: StepStatus,
    pub started_at: String,
    pub completed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<BrowserFinding>,
    pub timing: StepTiming,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities_active: Vec<String>,
    pub schema_version: u32,
}

// ---------------------------------------------------------------------------
// DomInspectionConfig + BrowserValidationStep (T015)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomInspectionConfig {
    #[serde(default = "default_dom_root_selector")]
    pub root_selector: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
}

fn default_dom_root_selector() -> String {
    "body".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserValidationStep {
    pub validation_run_id: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness: Option<ReadinessLocator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction_script: Option<Vec<BrowserAction>>,
    #[serde(default)]
    pub accessibility_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dom_inspection: Option<DomInspectionConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_ref: Option<String>,
    #[serde(default)]
    pub timeouts: ValidationTimeouts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_allowlist: Option<Vec<String>>,
    pub artifact_dir: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// ValidationTimeouts (T016)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationTimeouts {
    #[serde(default = "default_page_load_timeout")]
    pub page_load_seconds: u32,
    #[serde(default = "default_readiness_timeout")]
    pub readiness_seconds: u32,
    #[serde(default = "default_script_step_timeout")]
    pub script_step_seconds: u32,
    #[serde(default = "default_execution_timeout")]
    pub execution_seconds: u32,
}

const fn default_page_load_timeout() -> u32 {
    30
}
const fn default_readiness_timeout() -> u32 {
    20
}
const fn default_execution_timeout() -> u32 {
    120
}

impl Default for ValidationTimeouts {
    fn default() -> Self {
        Self {
            page_load_seconds: default_page_load_timeout(),
            readiness_seconds: default_readiness_timeout(),
            script_step_seconds: default_script_step_timeout(),
            execution_seconds: default_execution_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_status_all_variants_serialize() {
        let variants = [
            StepStatus::Completed,
            StepStatus::Failed,
            StepStatus::TimedOut,
            StepStatus::ProviderError,
            StepStatus::Cancelled,
            StepStatus::QueueTimeout,
            StepStatus::QueueFull,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).expect("serialize");
            let round: StepStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*v, round, "StepStatus round-trip failed for {v:?}");
        }
    }

    #[test]
    fn finding_kind_all_variants_serialize() {
        let variants = [
            FindingKind::ConsoleError,
            FindingKind::AccessibilityViolation,
            FindingKind::VisualDiffDetected,
            FindingKind::NetworkAccessViolation,
            FindingKind::PageLoadTimeout,
            FindingKind::BrowserReadinessTimeout,
            FindingKind::BaselineCreated,
            FindingKind::ScriptStepFailed,
            FindingKind::AccessibilityScanFailed,
            FindingKind::BrowserConcurrencyTimeout,
            FindingKind::BrowserQueueFull,
            FindingKind::CancelledBeforeStart,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).expect("serialize");
            let round: FindingKind = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*v, round, "FindingKind round-trip failed for {v:?}");
        }
    }

    #[test]
    fn evidence_packet_round_trip() {
        let packet = BrowserEvidencePacket {
            validation_run_id: "run-1".into(),
            provider_id: "browser-playwright".into(),
            status: StepStatus::Completed,
            started_at: "2026-06-20T10:00:00Z".into(),
            completed_at: "2026-06-20T10:00:05Z".into(),
            page_title: Some("Test Page".into()),
            http_status: Some(200),
            artifacts: vec![ArtifactReference {
                kind: ArtifactKind::Screenshot,
                relative_path: "screenshots/final.png".into(),
                content_hash: "sha256:abc123".into(),
                media_type: "image/png".into(),
                byte_size: 12345,
                created_at: "2026-06-20T10:00:05Z".into(),
                retention_class: RetentionClass::RequiredEvidence,
                validation_run_id: "run-1".into(),
            }],
            findings: vec![],
            timing: StepTiming {
                queue_wait_ms: None,
                navigation_ms: Some(1840),
                readiness_wait_ms: Some(350),
                script_execution_ms: None,
                accessibility_ms: None,
                total_ms: 2190,
            },
            capabilities_active: vec!["screenshot".into(), "console".into()],
            schema_version: 1,
        };
        let json = serde_json::to_string(&packet).expect("serialize");
        let round: BrowserEvidencePacket = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(packet.validation_run_id, round.validation_run_id);
        assert_eq!(packet.status, round.status);
        assert_eq!(packet.schema_version, round.schema_version);
        assert_eq!(packet.artifacts.len(), 1);
        assert_eq!(packet.artifacts[0].kind, ArtifactKind::Screenshot);
    }

    #[test]
    fn validation_step_serialization_matches_contract() {
        let step = BrowserValidationStep {
            validation_run_id: "run-2".into(),
            url: "http://localhost:3000".into(),
            readiness: Some(ReadinessLocator {
                locator_type: LocatorType::TestId,
                value: "dashboard-ready".into(),
                state: LocatorState::Visible,
                timeout_seconds: 20,
                stabilization_delay_ms: Some(250),
            }),
            interaction_script: None,
            accessibility_enabled: false,
            dom_inspection: None,
            baseline_ref: None,
            timeouts: ValidationTimeouts::default(),
            network_allowlist: None,
            artifact_dir: ".boundline/sessions/sess-1/browser/run-2".into(),
            session_id: "sess-1".into(),
        };
        let json = serde_json::to_string_pretty(&step).expect("serialize");
        // Verify key fields present in contract format
        assert!(json.contains("\"validation_run_id\": \"run-2\""));
        assert!(json.contains("\"url\": \"http://localhost:3000\""));
        assert!(json.contains("\"type\": \"test_id\""));
        // readiness should have "type" not "locator_type" per contract
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
        let readiness = value.get("readiness").expect("readiness field");
        assert_eq!(readiness.get("type").and_then(|v| v.as_str()), Some("test_id"));
        assert_eq!(readiness.get("state").and_then(|v| v.as_str()), Some("visible"));
    }
}
