//! Help-next domain types and diagnostic logic.
//!
//! This module defines the state model, diagnostic collector, link-map
//! resolver, and recommendation builder used by the `boundline help-next`
//! command. All diagnostics are deterministic inspections of typed
//! Boundline state; no mutation occurs.

use serde::{Deserialize, Serialize};

/// Detectable workspace/runtime states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpNextState {
    /// No `.boundline/` directory exists.
    Uninitialized,
    /// `.boundline/` exists but no active session.
    Initialized,
    /// Active session, current lifecycle phase, no blockers — healthy.
    Active,
    /// Active session with a blocking planning-analysis or execution gate.
    Blocked,
    /// Session in a terminal failure state (or corrupt/unreadable session file).
    Failed,
    /// Healthy active session — no blockers, next command available.
    Ready,
}

impl HelpNextState {
    /// Human-readable label for CLI output.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Uninitialized => "uninitialized",
            Self::Initialized => "initialized",
            Self::Active => "active",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
            Self::Ready => "ready",
        }
    }
}

/// Severity of a diagnostic finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Informational, no action needed.
    Info = 0,
    /// Does not block but indicates a gap.
    Warning = 1,
    /// Prevents the next action from proceeding.
    Blocking = 2,
}

/// Stable diagnostic keys for known issue categories.
pub const DIAGNOSTIC_KEY_UNINITIALIZED: &str = "workspace_not_initialized";
pub const DIAGNOSTIC_KEY_NO_SESSION: &str = "workspace_initialized_no_session";
pub const DIAGNOSTIC_KEY_BLOCKED_PLANNING: &str = "session_blocked_planning";
pub const DIAGNOSTIC_KEY_BLOCKED_EXECUTION: &str = "session_blocked_execution";
pub const DIAGNOSTIC_KEY_FAILED: &str = "session_failed";
pub const DIAGNOSTIC_KEY_CONFIG_MISSING: &str = "config_missing_key";
pub const DIAGNOSTIC_KEY_PROVIDER_NOT_ACTIVATED: &str = "provider_not_activated";
pub const DIAGNOSTIC_KEY_CONTEXT_PACK_MISSING: &str = "context_pack_missing";
pub const DIAGNOSTIC_KEY_GUARDIAN_FINDING: &str = "guardian_finding_active";
pub const DIAGNOSTIC_KEY_STOP_RULE: &str = "stop_rule_active";
pub const DIAGNOSTIC_KEY_HEALTHY: &str = "session_healthy";
pub const DIAGNOSTIC_KEY_FALLBACK: &str = "fallback";

/// A single actionable diagnostic finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpNextDiagnostic {
    /// Stable diagnostic key (e.g., `"workspace_not_initialized"`).
    pub key: String,
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Human-readable description.
    pub message: String,
    /// Source file or config path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Recommended CLI command to resolve the issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Key for the link map.
    pub docs_key: String,
}

/// The resolved next action returned to the operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpNextRecommendation {
    pub state: HelpNextState,
    pub blockers_found: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_issue: Option<HelpNextDiagnostic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_issues: Vec<HelpNextDiagnostic>,
    pub additional_count: u64,
    pub recommended_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_command: Option<String>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs_link: Option<String>,
}

impl HelpNextRecommendation {
    /// Build a recommendation from a collected set of diagnostics.
    #[must_use]
    pub fn from_diagnostics(
        state: HelpNextState,
        mut diagnostics: Vec<HelpNextDiagnostic>,
        docs_link: Option<String>,
    ) -> Self {
        diagnostics.sort_by_key(|d| std::cmp::Reverse(d.severity));
        let primary = diagnostics.first().cloned();
        let additional: Vec<_> = diagnostics.iter().skip(1).cloned().collect();
        let additional_count = additional.len() as u64;
        let blockers_found =
            primary.as_ref().is_some_and(|d| d.severity == DiagnosticSeverity::Blocking);

        let (recommended_action, recommended_command, reason) = match state {
            HelpNextState::Uninitialized => (
                "initialize workspace".into(),
                Some("boundline init".into()),
                "no .boundline/ directory found — initialization is required before any workflow"
                    .into(),
            ),
            HelpNextState::Initialized => (
                "create a goal".into(),
                Some("boundline goal".into()),
                "workspace initialized but no active session — create a goal to begin".into(),
            ),
            HelpNextState::Blocked if blockers_found => {
                let msg = primary.as_ref().map_or(String::new(), |d| d.message.clone());
                (
                    "repair and re-run plan".into(),
                    primary.as_ref().and_then(|d| d.command.clone()),
                    format!("blocked: {msg}"),
                )
            }
            HelpNextState::Failed => (
                "diagnose failure and retry or re-initialize".into(),
                Some("boundline run".into()),
                "session is in a failed state — review diagnostics and retry".into(),
            ),
            _ => (
                "continue execution".into(),
                Some("boundline run".into()),
                "healthy session — no blockers detected".into(),
            ),
        };

        Self {
            state,
            blockers_found,
            primary_issue: primary,
            additional_issues: additional,
            additional_count,
            recommended_action,
            recommended_command,
            reason,
            docs_link,
        }
    }

    /// Build a ready-state recommendation for a healthy session.
    #[must_use]
    pub fn ready(docs_link: Option<String>) -> Self {
        Self {
            state: HelpNextState::Ready,
            blockers_found: false,
            primary_issue: None,
            additional_issues: Vec::new(),
            additional_count: 0,
            recommended_action: "continue execution".into(),
            recommended_command: Some("boundline run".into()),
            reason: "healthy session — no blockers detected".into(),
            docs_link,
        }
    }

    /// Build a failed-state recommendation for a corrupt/unreadable session.
    #[must_use]
    pub fn corrupt_session(docs_link: Option<String>) -> Self {
        Self {
            state: HelpNextState::Failed,
            blockers_found: true,
            primary_issue: Some(HelpNextDiagnostic {
                key: DIAGNOSTIC_KEY_FAILED.into(),
                severity: DiagnosticSeverity::Blocking,
                message: "session file is corrupt or unreadable".into(),
                source: Some(".boundline/session.json".into()),
                command: Some("restore from backup or re-initialize".into()),
                docs_key: DIAGNOSTIC_KEY_FAILED.into(),
            }),
            additional_issues: Vec::new(),
            additional_count: 0,
            recommended_action: "restore session or re-initialize".into(),
            recommended_command: None,
            reason: "session file is corrupt — restore from backup or re-initialize the workspace"
                .into(),
            docs_link,
        }
    }
}

/// Payload for the `boundline.help_next.requested` structured event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpNextEvent {
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_category: Option<String>,
    pub diagnostics_count: u64,
    pub recommended_action_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs_link: Option<String>,
    pub output_format: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_recommendation_suggests_init() {
        let rec = HelpNextRecommendation::from_diagnostics(
            HelpNextState::Uninitialized,
            vec![],
            Some("wiki/Getting-Started".into()),
        );
        assert_eq!(rec.state, HelpNextState::Uninitialized);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline init"));
        assert!(!rec.blockers_found);
    }

    #[test]
    fn blocked_state_has_blocker_flag() {
        let diag = HelpNextDiagnostic {
            key: DIAGNOSTIC_KEY_BLOCKED_PLANNING.into(),
            severity: DiagnosticSeverity::Blocking,
            message: "planning analysis blocked".into(),
            source: Some(".boundline/session.json".into()),
            command: Some("boundline plan".into()),
            docs_key: DIAGNOSTIC_KEY_BLOCKED_PLANNING.into(),
        };
        let rec =
            HelpNextRecommendation::from_diagnostics(HelpNextState::Blocked, vec![diag], None);
        assert!(rec.blockers_found);
        assert_eq!(rec.primary_issue.as_ref().unwrap().severity, DiagnosticSeverity::Blocking);
    }

    #[test]
    fn diagnostics_sorted_by_severity_descending() {
        let d1 = HelpNextDiagnostic {
            key: "info".into(),
            severity: DiagnosticSeverity::Info,
            message: "i".into(),
            source: None,
            command: None,
            docs_key: "fallback".into(),
        };
        let d2 = HelpNextDiagnostic {
            key: "block".into(),
            severity: DiagnosticSeverity::Blocking,
            message: "b".into(),
            source: None,
            command: None,
            docs_key: "fallback".into(),
        };
        let rec =
            HelpNextRecommendation::from_diagnostics(HelpNextState::Blocked, vec![d1, d2], None);
        assert_eq!(rec.primary_issue.unwrap().severity, DiagnosticSeverity::Blocking);
        assert_eq!(rec.additional_count, 1);
    }

    #[test]
    fn ready_state_is_blocker_free() {
        let rec = HelpNextRecommendation::ready(Some("wiki/Daily".into()));
        assert_eq!(rec.state, HelpNextState::Ready);
        assert!(!rec.blockers_found);
        assert!(rec.primary_issue.is_none());
    }

    #[test]
    fn corrupt_session_has_failed_state() {
        let rec = HelpNextRecommendation::corrupt_session(None);
        assert_eq!(rec.state, HelpNextState::Failed);
        assert!(rec.blockers_found);
        assert!(rec.primary_issue.is_some());
    }

    #[test]
    fn help_next_state_labels_are_non_empty() {
        let states = [
            HelpNextState::Uninitialized,
            HelpNextState::Initialized,
            HelpNextState::Active,
            HelpNextState::Blocked,
            HelpNextState::Failed,
            HelpNextState::Ready,
        ];
        for s in states {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn initialized_state_suggests_goal() {
        let rec =
            HelpNextRecommendation::from_diagnostics(HelpNextState::Initialized, vec![], None);
        assert_eq!(rec.state, HelpNextState::Initialized);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline goal"));
        assert!(!rec.blockers_found);
    }

    #[test]
    fn failed_state_from_diagnostics_suggests_retry() {
        let rec = HelpNextRecommendation::from_diagnostics(HelpNextState::Failed, vec![], None);
        assert_eq!(rec.state, HelpNextState::Failed);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline run"));
    }

    #[test]
    fn blocked_state_with_no_blocking_diagnostics_is_not_blocked() {
        let diag = HelpNextDiagnostic {
            key: "warn".into(),
            severity: DiagnosticSeverity::Warning,
            message: "just a warning".into(),
            source: None,
            command: None,
            docs_key: "fallback".into(),
        };
        let rec =
            HelpNextRecommendation::from_diagnostics(HelpNextState::Blocked, vec![diag], None);
        assert!(!rec.blockers_found);
    }

    #[test]
    fn ready_state_via_from_diagnostics() {
        let rec = HelpNextRecommendation::from_diagnostics(HelpNextState::Ready, vec![], None);
        assert_eq!(rec.state, HelpNextState::Ready);
        assert!(!rec.blockers_found);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline run"));
    }

    #[test]
    fn active_state_via_from_diagnostics() {
        let rec = HelpNextRecommendation::from_diagnostics(HelpNextState::Active, vec![], None);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline run"));
    }

    #[test]
    fn diagnostic_serialization_roundtrip() {
        let d = HelpNextDiagnostic {
            key: "test".into(),
            severity: DiagnosticSeverity::Blocking,
            message: "msg".into(),
            source: Some("file".into()),
            command: Some("cmd".into()),
            docs_key: "fallback".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let parsed: HelpNextDiagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key, "test");
        assert_eq!(parsed.severity, DiagnosticSeverity::Blocking);
    }

    #[test]
    fn recommendation_serialization_roundtrip() {
        let rec = HelpNextRecommendation::ready(Some("docs".into()));
        let json = serde_json::to_string(&rec).unwrap();
        let parsed: HelpNextRecommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.state, HelpNextState::Ready);
    }

    #[test]
    fn help_next_event_serialization_roundtrip() {
        let event = HelpNextEvent {
            state: "blocked".into(),
            lifecycle_phase: Some("plan".into()),
            blocked_category: Some("planning".into()),
            diagnostics_count: 2,
            recommended_action_id: "repair".into(),
            recommended_command: Some("boundline plan".into()),
            docs_link: Some("wiki/troubleshoot".into()),
            output_format: "human".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HelpNextEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.state, "blocked");
        assert_eq!(parsed.diagnostics_count, 2);
    }

    #[test]
    fn diagnostic_key_constants_are_non_empty() {
        let keys = [
            DIAGNOSTIC_KEY_UNINITIALIZED,
            DIAGNOSTIC_KEY_NO_SESSION,
            DIAGNOSTIC_KEY_BLOCKED_PLANNING,
            DIAGNOSTIC_KEY_BLOCKED_EXECUTION,
            DIAGNOSTIC_KEY_FAILED,
            DIAGNOSTIC_KEY_CONFIG_MISSING,
            DIAGNOSTIC_KEY_PROVIDER_NOT_ACTIVATED,
            DIAGNOSTIC_KEY_CONTEXT_PACK_MISSING,
            DIAGNOSTIC_KEY_GUARDIAN_FINDING,
            DIAGNOSTIC_KEY_STOP_RULE,
            DIAGNOSTIC_KEY_HEALTHY,
            DIAGNOSTIC_KEY_FALLBACK,
        ];
        for k in keys {
            assert!(!k.is_empty());
        }
    }

    #[test]
    fn severity_ordering_blocking_gt_warning_gt_info() {
        assert!(DiagnosticSeverity::Blocking > DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning > DiagnosticSeverity::Info);
    }
}
