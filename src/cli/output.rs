//! Operator-facing text and JSON renderers for CLI commands.

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;

use crate::cli::diagnostics::{DiagnosticsReport, DiagnosticsStatus};
use crate::cli::{CliValidationError, DeveloperCommand};
use crate::domain::configuration::{
    ModelRoute, RoutingConfig, RoutingOverrides, resolve_effective_routing,
    resolve_effective_runtime_capabilities, resolve_effective_slot_effort_policies,
};
use crate::domain::context_intelligence::{
    AdvancedContextProjection, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
    RelationshipCredibilityState, RelationshipKind, RetrievalSourceKind,
    RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState, SemanticTraceRecord,
};
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::reasoning::ProfileActivationRecord;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::RoutingOutcome;
use crate::domain::session::{
    RoutingMode, RoutingSource, SessionStatus, SessionStatusView, governance_packet_provenance_text,
};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskRunResponse, TaskStatus};
use crate::domain::trace::{ExecutionTrace, TraceEventType, TraceSummaryView};

const KEY_FAILURE_REASON: &str = "failure_reason";
const KEY_FINDING: &str = "finding";
const KEY_REASON: &str = "reason";
const KEY_REVIEW_OUTCOME: &str = "review_outcome";
const KEY_REVIEW_TRIGGER: &str = "review_trigger";
const KEY_REVIEWER_ID: &str = "reviewer_id";
const KEY_REVIEWER_ROLE: &str = "reviewer_role";
const KEY_STAGE_ID: &str = "stage_id";
const KEY_SUMMARY: &str = "summary";
const KEY_VOTE_RESOLUTION: &str = "vote_resolution";
const UNKNOWN_STAGE_ID: &str = "unknown-stage";
const EXPLANATION_CONFIDENCE_HIGH: &str = "high";
const EXPLANATION_CONFIDENCE_LOW: &str = "low";
const EXPLANATION_CONFIDENCE_MEDIUM: &str = "medium";
const EXPLANATION_FALLBACK_READY: &str =
    "Using authoritative Boundline runtime and available Canon signals.";
const EXPLANATION_FALLBACK_CANON_MISSING: &str =
    "Canon input not yet available; using Boundline runtime evidence only";
const EXPLANATION_FALLBACK_CLARIFICATION_PREFIX: &str = "Clarification is still required for: ";
const EXPLANATION_FALLBACK_CONTEXT_STALE_PREFIX: &str =
    "Context is stale; refresh before treating this answer as fully current: ";
const EXPLANATION_MISSING_CANON_SOURCE: &str = "canon_input";
const EXPLANATION_MISSING_CLARIFICATION_SOURCE: &str = "clarification_fields";
const EXPLANATION_MISSING_CONTEXT_SOURCE: &str = "fresh_context";
const EXPLANATION_NONE: &str = "none";
const EXPLANATION_RUNTIME_SOURCE_AUTHORED_INPUT: &str = "authored_input";
const EXPLANATION_RUNTIME_SOURCE_CONTEXT: &str = "context";
const EXPLANATION_RUNTIME_SOURCE_DECISION_TIMELINE: &str = "decision_timeline";
const EXPLANATION_RUNTIME_SOURCE_REASONING_PROFILE: &str = "reasoning_profile";
const EXPLANATION_RUNTIME_SOURCE_REVIEW_TIMELINE: &str = "review_timeline";
const EXPLANATION_RUNTIME_SOURCE_SESSION_STATE: &str = "session_state";
const EXPLANATION_RUNTIME_SOURCE_TRACE_EVIDENCE: &str = "trace_evidence";
const EXPLANATION_RUNTIME_SOURCE_TRACE_STEPS: &str = "trace_steps";
const EXPLANATION_CANON_SOURCE_APPROVAL_PROVENANCE: &str = "approval_provenance";
const EXPLANATION_CANON_SOURCE_GOVERNANCE_ACTION: &str = "governance_next_action";
const EXPLANATION_CANON_SOURCE_GOVERNANCE_DECISION: &str = "governance_decision";
const EXPLANATION_CANON_SOURCE_GOVERNANCE_PACKET: &str = "governance_packet";
const EXPLANATION_CANON_SOURCE_GOVERNANCE_TIMELINE: &str = "governance_timeline";
const EXPLANATION_LABEL_CONFIDENCE_LEVEL: &str = "confidence_level";
const EXPLANATION_LABEL_EVIDENCE_SUMMARY: &str = "evidence_summary";
const EXPLANATION_LABEL_FALLBACK_DISCLOSURE: &str = "fallback_disclosure";
const EXPLANATION_LABEL_NEXT_BEST_ACTION: &str = "next_best_action";
const EXPLANATION_LABEL_RISK_SUMMARY: &str = "risk_summary";
const EXPLANATION_LABEL_SOURCE_ATTRIBUTION: &str = "source_attribution";
const EXPLANATION_LABEL_WHY_SUMMARY: &str = "why_summary";
const EXPLANATION_LABEL_ASSUMPTIONS_SUMMARY: &str = "assumptions_summary";
const EXPLANATION_LABEL_ASSUMPTION_GROUP: &str = "assumption_group";
const EXPLANATION_LABEL_HIDDEN_IMPACT_SUMMARY: &str = "hidden_impact_summary";
const EXPLANATION_LABEL_HIDDEN_IMPACT_FALLBACK_DISCLOSURE: &str =
    "hidden_impact_fallback_disclosure";
const EXPLANATION_LABEL_CHALLENGE_COUNCIL_REQUIRED: &str = "challenge_council_required";
const EXPLANATION_LABEL_CHALLENGE_FAILURE_MODE: &str = "challenge_failure_mode";
const EXPLANATION_LABEL_CHALLENGE_MISSING_EVIDENCE: &str = "challenge_missing_evidence";
const EXPLANATION_LABEL_CHALLENGE_REQUIRED_REVIEW: &str = "challenge_required_review";
const EXPLANATION_LABEL_CHALLENGE_STRONGEST_OBJECTION: &str = "challenge_strongest_objection";
const EXPLANATION_LABEL_CHALLENGE_WEAKEST_ASSUMPTION: &str = "challenge_weakest_assumption";
const EXPLANATION_LABEL_EXPLAIN_PLAN_GOVERNANCE: &str = "explain_plan_governance";
const EXPLANATION_LABEL_EXPLAIN_PLAN_RECOVERY: &str = "explain_plan_recovery";
const EXPLANATION_LABEL_EXPLAIN_PLAN_SUMMARY: &str = "explain_plan_summary";
const EXPLANATION_LABEL_EXPLAIN_PLAN_VALIDATION: &str = "explain_plan_validation";
const EXPLANATION_LABEL_REASONING_CONTRIBUTION: &str = "reasoning_contribution";
const EXPLANATION_LABEL_REASONING_FALLBACK_DISCLOSURE: &str = "reasoning_fallback_disclosure";
const EXPLANATION_LABEL_REASONING_SELECTION_REASON: &str = "reasoning_selection_reason";
const EXPLANATION_ASSUMPTION_CATEGORY_ARCHITECTURE: &str = "architecture";
const EXPLANATION_ASSUMPTION_CATEGORY_DOMAIN: &str = "domain";
const EXPLANATION_ASSUMPTION_CATEGORY_GOVERNANCE: &str = "governance";
const EXPLANATION_ASSUMPTION_CATEGORY_IMPLEMENTATION: &str = "implementation";
const EXPLANATION_ASSUMPTION_CATEGORY_VALIDATION: &str = "validation";
const EXPLANATION_ASSUMPTION_RISK_HIGH: &str = "high";
const EXPLANATION_ASSUMPTION_RISK_LOW: &str = "low";
const EXPLANATION_ASSUMPTION_RISK_MEDIUM: &str = "medium";
const EXPLANATION_ASSUMPTION_SOURCE_CANON: &str = "Canon";
const EXPLANATION_ASSUMPTION_SOURCE_TRACE: &str = "trace";
const EXPLANATION_ASSUMPTION_SOURCE_WORKSPACE: &str = "workspace";
const EXPLANATION_ASSUMPTION_STATUS_EXPLICIT: &str = "explicit";
const EXPLANATION_ASSUMPTION_STATUS_INFERRED: &str = "inferred";
const EXPLANATION_ASSUMPTION_STATUS_MISSING: &str = "missing";
const EXPLANATION_COUNCIL_REQUIRED_NO: &str = "no";
const EXPLANATION_COUNCIL_REQUIRED_YES: &str = "yes";
const EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS: &str = "affected_domains";
const EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS: &str = "affected_systems";
const EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES: &str = "contract_exposures";
const EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE: &str = "missing_evidence";
const EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS: &str = "missing_tests";
const EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS: &str = "required_reviewers";
const EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_DOMAINS: &str = "hidden_impact_affected_domains";
const EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_SYSTEMS: &str = "hidden_impact_affected_systems";
const EXPLANATION_HIDDEN_IMPACT_LABEL_CONTRACT_EXPOSURES: &str = "hidden_impact_contract_exposures";
const EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_EVIDENCE: &str = "hidden_impact_missing_evidence";
const EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_TESTS: &str = "hidden_impact_missing_tests";
const EXPLANATION_HIDDEN_IMPACT_LABEL_REQUIRED_REVIEWERS: &str = "hidden_impact_required_reviewers";
const EXPLANATION_REVIEW_RUNTIME_ONLY: &str = "bounded runtime review only";
const EXPLANATION_WEAK_ASSUMPTION_NONE: &str = "none";
const EXPLANATION_RISK_CANON_GAP: &str =
    "Canon confirmation is missing, so risk remains bounded by runtime-only evidence.";
const EXPLANATION_RISK_NO_EXPLICIT_FAILURE: &str =
    "No explicit runtime failure evidence is currently reported.";
const EXPLANATION_WHY_FALLBACK: &str =
    "Boundline has current runtime state but no richer explanation summary yet.";

#[path = "output_run_trace.rs"]
mod run_trace;

#[path = "output_session_status.rs"]
mod session_status;

#[path = "output_trace_summary.rs"]
mod trace_summary;

#[path = "output_explanation.rs"]
mod explanation;

#[path = "output_context.rs"]
mod context;

#[path = "output_cluster.rs"]
mod cluster;

#[path = "output_delight.rs"]
mod delight;

#[path = "output_compatibility.rs"]
mod compatibility;

#[path = "output_host.rs"]
mod host;

#[path = "output_runtime.rs"]
mod runtime;

#[path = "output_support.rs"]
mod support;

#[path = "output_routing.rs"]
mod routing;

#[path = "output_events.rs"]
mod events;

#[path = "output_orchestrate.rs"]
mod orchestrate;

pub use cluster::{render_cluster_init, render_cluster_inspect, render_cluster_status};
pub use compatibility::render_compatibility_follow_up_status;
pub use events::render_diagnostics;
pub use host::{
    CommandExitCode, HostCommandEnvelope, command_name, render_host_command_json,
    render_orchestrate_event_json, render_orchestrate_stream_json,
};
pub use orchestrate::render_human_orchestrate_report;
pub use routing::{
    render_goal_plan_flow_state, render_route_outcome, trace_execution_condition_text,
};
pub use run_trace::render_run_trace;
pub use session_status::{render_session_status, render_session_status_brief};
pub use support::{
    next_command_after_inspect, next_command_after_run, render_guidance_projection_brief_lines,
    render_guidance_projection_lines, render_inspect_failure, render_session_error,
};
pub use trace_summary::{
    render_trace_audit_summary, render_trace_summary, render_trace_summary_brief,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExplanationProjection {
    why_summary: String,
    risk_summary: String,
    evidence_summary: String,
    source_attribution: String,
    fallback_disclosure: String,
    confidence_level: &'static str,
    next_best_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExplanationAssumptionEntry {
    category: &'static str,
    subject_ref: String,
    status: &'static str,
    source: &'static str,
    risk: &'static str,
    explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExplanationHiddenImpactEntry {
    group: &'static str,
    label: &'static str,
    subject_ref: String,
    status: &'static str,
    severity: &'static str,
    follow_up: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExplanationCognitiveProjection {
    assumptions_summary: String,
    assumption_groups: Vec<String>,
    hidden_impact_summary: String,
    hidden_impact_lines: Vec<String>,
    hidden_impact_fallback_disclosure: Option<String>,
    challenge_strongest_objection: String,
    challenge_weakest_assumption: String,
    challenge_missing_evidence: String,
    challenge_failure_mode: String,
    challenge_required_review: String,
    challenge_council_required: &'static str,
    explain_plan_summary: String,
    explain_plan_validation: String,
    explain_plan_governance: String,
    explain_plan_recovery: String,
}

/// Returns the default not-implemented message for a developer command.
pub fn unimplemented_message(command: &DeveloperCommand) -> String {
    format!("`{}` is not implemented yet", command_name(command))
}

/// Returns the user-facing validation error message for CLI argument failures.
pub fn validation_error_message(error: &CliValidationError) -> String {
    error.to_string()
}

const UNKNOWN_VALIDATION_EXIT_CODE: i64 = -1;

fn task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

fn step_kind_text(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Agent => "agent",
        StepKind::Tool => "tool",
        StepKind::Decision => "decision",
    }
}

fn step_status_text(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
    }
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::context::push_advanced_context_lines;
    use super::events::{
        governance_event_line, render_diagnostics, review_event_line, reviewer_event_line,
    };
    use super::routing::{
        render_run_execution_condition, session_execution_condition_parts,
        trace_execution_condition_parts,
    };
    use super::{
        command_name, render_guidance_projection_brief_lines, render_host_command_json,
        render_run_trace, render_session_status, render_session_status_brief,
        render_trace_audit_summary, render_trace_summary, render_trace_summary_brief,
    };
    use crate::cli::CommandExitStatus;
    use crate::cli::assistant_assets::{AssistantHost, AssistantInstallScope};
    use crate::cli::diagnostics::{
        DiagnosticsCheck, DiagnosticsReport, DiagnosticsStatus, DiagnosticsSubject,
    };
    use crate::cli::{
        AssistantSubcommand, CheckpointSubcommand, ClusterSubcommand, ConfigSubcommand,
        DeveloperCommand,
    };
    use crate::domain::audit::{
        SessionAuditActor, SessionAuditActorKind, SessionAuditAlgorithm, SessionAuditEntry,
        SessionAuditEntryKind, SessionAuditIdentity, SessionAuditOutcome,
        SessionAuditOutcomeStatus, SessionAuditPhase, SessionAuditProjection, SessionAuditSource,
        SessionAuditSourceKind,
    };
    use crate::domain::configuration::InitConfigScope;
    use crate::domain::context_intelligence::{
        AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
        RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
        RetrievalMatchOrigin, RetrievalMode, RetrievalScore, RetrievalSourceKind,
        RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate,
        SemanticCapabilityState, SemanticPolicyState, SemanticTraceEventKind, SemanticTraceRecord,
    };
    use crate::domain::governance::CanonMode;
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::routing_decision::RoutingDecisionProjection;
    use crate::domain::session::{ContinuityAuthority, SessionStatus, SessionStatusView};
    use crate::domain::step::{StepKind, StepStatus};
    use crate::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{
        ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
    };

    fn temp_output_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    #[test]
    fn push_advanced_context_lines_surfaces_semantic_summary() {
        let advanced_context = AdvancedContextProjection {
            query_id: "query-output-semantic".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Selected,
            retrieval_index_state: RetrievalIndexState::Ready,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Ready,
            hybrid_outcome: HybridOutcome::Expanded,
            budgets: Default::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some(
                "semantic expansion selected one additional bounded evidence candidate".to_string(),
            ),
            selected_evidence: vec![RetrievedEvidenceCandidate {
                candidate_id: "candidate-output-semantic".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                source_ref: "src/lib.rs".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::Fts,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "goal keyword matched the implementation surface".to_string(),
                provenance_summary: "workspace file selected through local retrieval".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            }],
            rejected_candidates: vec![RetrievedEvidenceCandidate {
                candidate_id: "candidate-output-rejected".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                source_ref: "src/semantic.rs".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::SemanticExpand,
                selection_state: CandidateSelectionState::Rejected,
                selection_reason: "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged".to_string(),
                provenance_summary: "workspace file evaluated through semantic expansion".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: RetrievalScore::from_raw(0.812),
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            }],
            semantic_trace_records: vec![
                SemanticTraceRecord {
                    record_id: "trace-output-extension".to_string(),
                    event_kind: SemanticTraceEventKind::ExtensionLoadAttempted,
                    candidate_ref: None,
                    match_origin: None,
                    compatibility_state: None,
                    semantic_score: None,
                    canon_artifact_class: None,
                    canon_semantic_contract_line: None,
                    canon_semantic_provenance_boundary: None,
                    canon_semantic_provenance_ref: None,
                    reason:
                        "trusted sqlite-vec extension load attempted: capability=ready retrieval_index_state=ready"
                            .to_string(),
                },
                SemanticTraceRecord {
                    record_id: "trace-output-vector-query".to_string(),
                    event_kind: SemanticTraceEventKind::VectorQueryExecuted,
                    candidate_ref: None,
                    match_origin: None,
                    compatibility_state: None,
                    semantic_score: None,
                    canon_artifact_class: None,
                    canon_semantic_contract_line: None,
                    canon_semantic_provenance_boundary: None,
                    canon_semantic_provenance_ref: None,
                    reason: "vector query executed through semantic engine: engine=sqlite_vec"
                        .to_string(),
                },
                SemanticTraceRecord {
                    record_id: "trace-output-vector-candidates".to_string(),
                    event_kind: SemanticTraceEventKind::VectorCandidatesReturned,
                    candidate_ref: None,
                    match_origin: None,
                    compatibility_state: None,
                    semantic_score: None,
                    canon_artifact_class: None,
                    canon_semantic_contract_line: None,
                    canon_semantic_provenance_boundary: None,
                    canon_semantic_provenance_ref: None,
                    reason:
                        "vector query returned chunk candidates before source collapse: 1"
                            .to_string(),
                },
                SemanticTraceRecord {
                    record_id: "trace-output-semantic".to_string(),
                    event_kind: SemanticTraceEventKind::CandidateRejected,
                    candidate_ref: Some("src/semantic.rs".to_string()),
                    match_origin: Some(RetrievalMatchOrigin::SemanticExpand),
                    compatibility_state: Some(RetrievalCompatibilityState::Compatible),
                    semantic_score: RetrievalScore::from_raw(0.812),
                    canon_artifact_class: None,
                    canon_semantic_contract_line: None,
                    canon_semantic_provenance_boundary: None,
                    canon_semantic_provenance_ref: None,
                    reason:
                        "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
                            .to_string(),
                },
            ],
            relationships: Vec::new(),
            impact_findings: Vec::new(),
        };
        let mut lines = Vec::new();

        push_advanced_context_lines(&mut lines, Some(&advanced_context));

        assert!(lines.iter().any(|line| line == "semantic_policy_state: local"));
        assert!(lines.iter().any(|line| line == "semantic_capability_state: ready"));
        assert!(lines.iter().any(|line| line == "hybrid_outcome: expanded"));
        assert!(lines.iter().any(|line| {
            line == "selected_evidence: src/lib.rs [workspace_file] origin=fts goal keyword matched the implementation surface"
        }));
        assert!(lines.iter().any(|line| {
            line == "rejected_candidate: src/semantic.rs [workspace_file] origin=semantic_expand semantic_score=0.812 semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
        }));
        assert!(lines.iter().any(|line| {
            line == "semantic_trace: candidate_rejected ref=src/semantic.rs origin=semantic_expand compatibility=compatible semantic_score=0.812 semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
        }));
        assert!(lines.iter().any(|line| {
            line == "semantic_trace: extension_load_attempted trusted sqlite-vec extension load attempted: capability=ready retrieval_index_state=ready"
        }));
        assert!(lines.iter().any(|line| {
            line == "semantic_trace: vector_query_executed vector query executed through semantic engine: engine=sqlite_vec"
        }));
        assert!(lines.iter().any(|line| {
            line == "semantic_trace: vector_candidates_returned vector query returned chunk candidates before source collapse: 1"
        }));
    }

    #[test]
    fn push_advanced_context_lines_surfaces_recovery_guidance_for_corrupt_index() {
        let advanced_context = AdvancedContextProjection {
            query_id: "query-output-recovery".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Degraded,
            retrieval_index_state: RetrievalIndexState::Corrupt,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Corrupt,
            hybrid_outcome: HybridOutcome::Fallback,
            budgets: Default::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some("derived index is corrupt; using bounded fallback".to_string()),
            selected_evidence: Vec::new(),
            rejected_candidates: Vec::new(),
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: Vec::new(),
        };
        let mut lines = Vec::new();

        push_advanced_context_lines(&mut lines, Some(&advanced_context));

        assert!(lines.iter().any(|line| {
            line == "retrieval_recovery_guidance: run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene"
        }));
    }

    #[test]
    fn push_advanced_context_lines_surfaces_recovery_guidance_for_other_index_states() {
        for (retrieval_index_state, expected_guidance) in [
            (
                RetrievalIndexState::Missing,
                "retrieval_recovery_guidance: run boundline index refresh in the target workspace before relying on semantic retrieval",
            ),
            (
                RetrievalIndexState::Stale,
                "retrieval_recovery_guidance: run boundline index refresh in the target workspace before relying on semantic retrieval",
            ),
            (
                RetrievalIndexState::Incompatible,
                "retrieval_recovery_guidance: run boundline index rebuild or boundline index doctor in the target workspace",
            ),
            (
                RetrievalIndexState::Degraded,
                "retrieval_recovery_guidance: run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene",
            ),
            (
                RetrievalIndexState::SemanticUnavailable,
                "retrieval_recovery_guidance: run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene",
            ),
            (
                RetrievalIndexState::Building,
                "retrieval_recovery_guidance: rerun boundline index status or refresh after the derived index is available",
            ),
            (
                RetrievalIndexState::Insufficient,
                "retrieval_recovery_guidance: rerun boundline index status or refresh after the derived index is available",
            ),
        ] {
            let advanced_context = AdvancedContextProjection {
                query_id: format!("query-output-{retrieval_index_state:?}"),
                retrieval_mode: RetrievalMode::Local,
                retrieval_state: RetrievalState::Degraded,
                retrieval_index_state,
                semantic_policy_state: SemanticPolicyState::Local,
                semantic_capability_state: SemanticCapabilityState::Unsupported,
                hybrid_outcome: HybridOutcome::Fallback,
                budgets: Default::default(),
                remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
                used_remote: false,
                terminal_reason: Some("bounded fallback in use".to_string()),
                selected_evidence: Vec::new(),
                rejected_candidates: Vec::new(),
                semantic_trace_records: Vec::new(),
                relationships: Vec::new(),
                impact_findings: Vec::new(),
            };
            let mut lines = Vec::new();

            push_advanced_context_lines(&mut lines, Some(&advanced_context));

            assert!(lines.iter().any(|line| line == expected_guidance), "{lines:?}");
        }
    }

    #[test]
    fn host_command_json_covers_exit_status_labels_and_optional_payloads() {
        for (status, label) in [
            (CommandExitStatus::Succeeded, "succeeded"),
            (CommandExitStatus::NonSuccess, "non_success"),
            (CommandExitStatus::InvalidInvocation, "invalid_invocation"),
            (CommandExitStatus::TraceReadFailure, "trace_read_failure"),
        ] {
            let rendered = render_host_command_json("doctor", status, "rendered", None, None, None);
            let parsed: Value = serde_json::from_str(&rendered).unwrap();
            assert_eq!(parsed["command_name"], "doctor");
            assert_eq!(parsed["exit_status"], label);
            assert_eq!(parsed["rendered_output"], "rendered");
            assert!(parsed["trace_location"].is_null());
            assert!(parsed["session_status"].is_null());
            assert!(parsed["trace_summary"].is_null());
        }

        let session_status = SessionStatusView {
            session_id: "session-host-json".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            latest_status: SessionStatus::Succeeded,
            explanation: "session completed successfully".to_string(),
            ..SessionStatusView::default()
        };
        let trace_summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
            goal: "Render host JSON".to_string(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            ..TraceSummaryView::default()
        };

        let rendered = render_host_command_json(
            "run",
            CommandExitStatus::Succeeded,
            "terminal_status: succeeded",
            Some("/tmp/workspace/.boundline/traces/task.json"),
            Some(&session_status),
            Some(&trace_summary),
        );
        let parsed: Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["trace_location"], "/tmp/workspace/.boundline/traces/task.json");
        assert_eq!(parsed["session_status"]["session_id"], "session-host-json");
        assert_eq!(
            parsed["trace_summary"]["trace_ref"],
            "/tmp/workspace/.boundline/traces/task.json"
        );
    }

    #[test]
    fn output_covers_canon_semantic_evidence_metadata_and_empty_section_paths() {
        use crate::domain::context_intelligence::RetrievalBudgets;
        use crate::domain::governance::CanonSemanticProvenanceBoundary;

        // Evidence candidate with both canon semantic metadata fields set covers lines 348-355.
        let advanced_context = AdvancedContextProjection {
            query_id: "query-output-canon-meta".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Selected,
            retrieval_index_state: RetrievalIndexState::Ready,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Ready,
            hybrid_outcome: HybridOutcome::Expanded,
            budgets: RetrievalBudgets::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some("expanded".to_string()),
            selected_evidence: vec![RetrievedEvidenceCandidate {
                candidate_id: "candidate-canon-meta".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                source_ref: "src/lib.rs".to_string(),
                authority_rank: AuthorityRank::Canon,
                match_origin: RetrievalMatchOrigin::SemanticExpand,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "canon semantic match".to_string(),
                provenance_summary: "canon artifact matched semantic query".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: RetrievalScore::from_raw(0.91),
                canon_semantic_contract_line: Some("v1".to_string()),
                canon_semantic_provenance_ref: Some(".canon/arch.md".to_string()),
            }],
            rejected_candidates: Vec::new(),
            semantic_trace_records: vec![SemanticTraceRecord {
                record_id: "trace-output-canon".to_string(),
                event_kind: SemanticTraceEventKind::CandidateExpanded,
                candidate_ref: Some("src/lib.rs".to_string()),
                match_origin: Some(RetrievalMatchOrigin::SemanticExpand),
                compatibility_state: Some(RetrievalCompatibilityState::Compatible),
                semantic_score: RetrievalScore::from_raw(0.91),
                canon_artifact_class: Some("stable".to_string()),
                canon_semantic_contract_line: Some("v1".to_string()),
                canon_semantic_provenance_boundary: Some(
                    CanonSemanticProvenanceBoundary::ManagedBlock,
                ),
                canon_semantic_provenance_ref: Some(".canon/arch.md".to_string()),
                reason: "canon evidence expanded the bounded set".to_string(),
            }],
            relationships: Vec::new(),
            impact_findings: Vec::new(),
        };
        let mut lines = Vec::new();
        push_advanced_context_lines(&mut lines, Some(&advanced_context));
        assert!(
            lines.iter().any(|line| line.contains("canon_contract=v1")),
            "missing canon_contract in: {lines:?}"
        );
        assert!(
            lines.iter().any(|line| line.contains("canon_provenance=.canon/arch.md")),
            "missing canon_provenance in: {lines:?}"
        );
    }

    #[test]
    fn diagnostics_render_install_follow_up_when_actions_are_missing() {
        let rendered = render_diagnostics(&DiagnosticsReport {
            subject: DiagnosticsSubject::Install,
            workspace_ref: None,
            installation_ref: None,
            checks: vec![DiagnosticsCheck {
                name: "boundline_binary".to_string(),
                status: DiagnosticsStatus::Passed,
                message: "install is ready".to_string(),
            }],
            ready: true,
            missing_prerequisites: Vec::new(),
            suggested_actions: Vec::new(),
            boundline_version: None,
            supported_canon_version: None,
            companion_state: None,
            channel_candidates: Vec::new(),
        });

        assert!(
            rendered.contains("doctor: ready for installation <current-machine>"),
            "{rendered}"
        );
        assert!(
            rendered.contains("verify a workspace next: boundline doctor --workspace <workspace>"),
            "{rendered}"
        );
    }

    #[test]
    fn command_name_covers_every_developer_subcommand() {
        let commands = [
            (
                DeveloperCommand::Doctor {
                    workspace: Some("/tmp/workspace".into()),
                    install: false,
                },
                "doctor",
            ),
            (
                DeveloperCommand::Goal {
                    workspace: None,
                    cluster: None,
                    update: false,
                    new_session: false,
                    goal: Some("goal".to_string()),
                    brief: Vec::new(),
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                    slug: None,
                },
                "goal",
            ),
            (
                DeveloperCommand::Flow {
                    name: "bug-fix".to_string(),
                    workspace: None,
                    cluster: None,
                },
                "flow",
            ),
            (
                DeveloperCommand::Plan {
                    workspace: None,
                    cluster: None,
                    input: None,
                    flow: None,
                    no_flow: false,
                    no_canon: false,
                },
                "plan",
            ),
            (DeveloperCommand::Step { workspace: None, cluster: None }, "step"),
            (
                DeveloperCommand::Run {
                    workspace: None,
                    cluster: None,
                    goal: None,
                    compatibility: false,
                    brief: Vec::new(),
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                    mode: None,
                    no_canon: false,
                },
                "run",
            ),
            (
                DeveloperCommand::Workflow {
                    command: crate::cli::WorkflowSubcommand::Run {
                        name: "default".to_string(),
                        workspace: None,
                        goal: None,
                    },
                },
                "workflow",
            ),
            (
                DeveloperCommand::Checkpoint {
                    command: CheckpointSubcommand::List {
                        workspace: None,
                        cluster: None,
                        session: None,
                    },
                },
                "checkpoint",
            ),
            (
                DeveloperCommand::Inspect {
                    trace: None,
                    workspace: None,
                    cluster: None,
                    session: None,
                    audit: false,
                },
                "inspect",
            ),
            (DeveloperCommand::Status { workspace: None, cluster: None, session: None }, "status"),
            (DeveloperCommand::Next { workspace: None, cluster: None, session: None }, "next"),
            (
                DeveloperCommand::Continue { workspace: None, cluster: None, session: None },
                "continue",
            ),
            (
                DeveloperCommand::Govern {
                    workspace: None,
                    mode: Some(CanonMode::Review),
                    goal: Some("Prepare review packet".to_string()),
                    brief: Vec::new(),
                    base: None,
                    head: None,
                    risk: None,
                    structural_impact: false,
                    public_contract_change: false,
                    validation_exhausted: false,
                    pr_ready: false,
                    preserved_behavior_evidence: false,
                },
                "govern",
            ),
            (
                DeveloperCommand::Assistant {
                    command: AssistantSubcommand::Install {
                        host: AssistantHost::Copilot,
                        scope: AssistantInstallScope::User,
                    },
                },
                "assistant",
            ),
            (
                DeveloperCommand::Init {
                    scope: InitConfigScope::Workspace,
                    workspace: "/tmp/workspace".into(),
                    non_interactive: false,
                    template: None,
                    assistant: Vec::new(),
                    adapter: None,
                    ide: Vec::new(),
                    auto_approve: None,
                    semantic_index_hook_action: None,
                    domain: Vec::new(),
                    domain_standard: Vec::new(),
                    context_binding: Vec::new(),
                    required_context_binding: Vec::new(),
                    canon_mode_selection: None,
                    risk: None,
                    zone: None,
                    owner: None,
                    export_docs: false,
                    refresh: false,
                    diff: false,
                    to: None,
                    route: Vec::new(),
                    force: false,
                },
                "init",
            ),
            (
                DeveloperCommand::Config {
                    command: ConfigSubcommand::Show { workspace: None, cluster: None, scope: None },
                },
                "config",
            ),
            (
                DeveloperCommand::Cluster {
                    command: ClusterSubcommand::Status { workspace: "/tmp/workspace".into() },
                },
                "cluster",
            ),
        ];

        for (command, expected) in commands {
            assert_eq!(command_name(&command), expected);
        }
    }

    #[test]
    fn render_run_trace_covers_stage_replan_and_stage_failure_fallbacks() {
        let mut trace = ExecutionTrace::new("task-output", "session-output", "Render output");
        trace.record_event(TraceEventType::StageReplanned, None, 0, json!({}));
        trace.record_event(TraceEventType::StageFailed, None, 0, json!({}));

        let response = TaskRunResponse {
            task_id: "task-output".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "stage failed",
                None,
            ),
            final_context: TaskContext::new(
                "session-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("stage replan after unknown-step: replan scheduled"), "{text}");
        assert!(text.contains("stage unknown-stage failed: stage failed"), "{text}");
    }

    #[test]
    fn render_run_trace_ignores_project_scale_and_voting_projection_events() {
        let mut trace = ExecutionTrace::new("task-output", "session-output", "Render output");
        trace.record_event(TraceEventType::ProjectScalePathProposed, None, 0, json!({}));
        trace.record_event(TraceEventType::ProjectScaleStageTransitioned, None, 0, json!({}));
        trace.record_event(TraceEventType::VotingDecisionRecorded, None, 0, json!({}));

        let response = TaskRunResponse {
            task_id: "task-output".to_string(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "run finished",
                None,
            ),
            final_context: TaskContext::new(
                "session-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("terminal_status: succeeded"), "{text}");
        assert!(!text.contains("project_scale"), "{text}");
        assert!(!text.contains("voting_decision"), "{text}");
    }

    #[test]
    fn render_trace_summary_labels_flow_stage_and_stage_failure_events() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-output.json".to_string(),
            goal: "Render trace summary".to_string(),
            trace_started_at: None,
            advanced_context: None,
            guidance_guardian: crate::domain::guidance::GuidanceGuardianProjection::default(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            routing_summary: None,
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            goal_brief_ref: None,
            session_plan_brief_ref: None,
            run_brief_ref: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: Vec::new(),
            context_provenance: Vec::new(),
            context_staleness_reason: None,
            plan_quality_state: None,
            plan_quality_findings: Vec::new(),
            plan_quality_assumptions: Vec::new(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            executed_steps: vec![TraceStepSummary {
                step_id: "verify".to_string(),
                step_kind: StepKind::Tool,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "validation passed".to_string(),
            }],
            recovery_events: vec![
                TraceRecoveryEvent {
                    event_type: TraceEventType::FlowSelected,
                    trigger: "bug-fix @ investigate".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageTransitioned,
                    trigger: "investigate -> implement".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageFailed,
                    trigger: "verify failed".to_string(),
                    related_step_id: Some("verify".to_string()),
                },
            ],
            governance_timeline: Vec::new(),
            governance_runtime_state: None,
            governance_rollout_profile: None,
            governance_reason: None,
            governance_approval_provenance: None,
            governance_next_action: None,
            reasoning_profile: None,
            delegation: None,
            inspect_context: None,
            inspect_council: None,
            inspect_timeline: None,
            review_timeline: Vec::new(),
            session_audit: None,
            delight_feedback: None,
            framework_adapter_stage_routing: None,
            framework_adapter_hook_dispatch: None,
            framework_adapter_stage_failure: None,
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            duration: None,
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("flow: bug-fix @ investigate"), "{text}");
        assert!(text.contains("stage: investigate -> implement"), "{text}");
        assert!(text.contains("stage_failure: verify failed"), "{text}");
    }

    #[test]
    fn render_session_status_covers_invalid_status_without_changed_files() {
        let view = SessionStatusView {
            session_id: "session-output".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            session_started_at: None,
            goal: None,
            advanced_context: None,
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: None,
            context_provenance: None,
            context_staleness_reason: None,
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: None,
            clarification_questions: None,
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            active_flow: None,
            flow_state: None,
            goal_plan_state: None,
            goal_plan_revision: None,
            planning_rationale: None,
            verification_strategy: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            delegation: None,
            compatibility_follow_up: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Invalid,
            execution_path: None,
            goal_brief_ref: None,
            session_plan_brief_ref: None,
            run_brief_ref: None,
            latest_trace_ref: None,
            latest_decision_status: None,
            latest_decision_target: None,
            latest_changed_files: Some(Vec::new()),
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_candidate_family: None,
            latest_selection_reason: None,
            latest_rejected_candidates: None,
            latest_attempt_lineage: None,
            latest_validation_status: None,
            latest_exhaustion_reason: None,
            latest_review_trigger: None,
            latest_review_vote: None,
            latest_review_outcome: None,
            latest_review_council_profile: None,
            latest_review_independence_state: None,
            latest_review_stop_semantics: None,
            latest_review_selection_summary: None,
            latest_review_headline: None,
            latest_governance_stage: None,
            latest_governance_runtime: None,
            latest_governance_mode: None,
            latest_governance_run_ref: None,
            latest_governance_state: None,
            latest_governance_runtime_state: None,
            latest_governance_rollout_profile: None,
            latest_governance_reason: None,
            latest_governance_contract_lines: None,
            latest_governance_approval_provenance: None,
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: None,
            latest_governance_packet_source_stage: None,
            latest_governance_packet_binding_reason: None,
            latest_governance_approval: None,
            latest_governance_decision: None,
            latest_governance_candidates: None,
            latest_governance_confidence_level: None,
            latest_governance_admission_effect: None,
            latest_governance_confidence_summary: None,
            governance_next_action: None,
            governance_lifecycle_runtime: None,
            governance_lifecycle_opt_out: None,
            governance_lifecycle_mode_selection: None,
            governance_lifecycle_selected_mode: None,
            governance_lifecycle_selected_mode_sequence: None,
            latest_reasoning_profile: None,
            project_scale_path: None,
            project_scale_current_stage: None,
            project_scale_next_action: None,
            project_scale_checkpoint_refs: None,
            latest_voting_trigger: None,
            latest_voting_result: None,
            latest_voting_adjudication: None,
            latest_voting_reviewed_evidence: None,
            latest_voting_blocking: None,
            latest_voting_next_action: None,
            delight_feedback: None,
            next_command: None,
            explanation: "session is invalid".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_status: invalid"), "{text}");
        assert!(!text.contains("latest_changed_files:"), "{text}");
    }

    #[test]
    fn render_session_status_brief_stays_compact_and_surfaces_authoritative_fields() {
        let view = SessionStatusView {
            session_id: "session-brief".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Implement the bounded checkout repair with the attached spec".to_string()),
            authored_input_sources: Some(vec![
                "attached_markdown: docs/checkout-spec.md".to_string(),
                "attached_markdown: docs/api-notes.md".to_string(),
            ]),
            context_provenance: Some(vec!["workspace_file: docs/checkout-spec.md".to_string()]),
            latest_status: SessionStatus::Planned,
            execution_path: Some("native_goal_plan".to_string()),
            goal_brief_ref: Some(".boundline/briefs/goal.md".to_string()),
            session_plan_brief_ref: Some(".boundline/briefs/plan.md".to_string()),
            goal_plan_state: Some("proposed".to_string()),
            goal_plan_revision: Some(2),
            latest_governance_stage: Some("plan:discovery".to_string()),
            latest_trace_ref: Some("/tmp/workspace/.boundline/traces/task.json".to_string()),
            latest_governance_packet_ref: Some(".canon/artifacts/R-20260522/discovery".to_string()),
            next_command: Some("boundline run".to_string()),
            explanation: "planned the active goal into 3 bounded goal-plan task(s)".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status_brief(&view);

        assert!(text.contains("goal: Implement the bounded checkout repair"), "{text}");
        assert!(
            text.contains("authored_input_sources: attached_markdown: docs/checkout-spec.md"),
            "{text}"
        );
        assert!(text.contains("routing: native (goal_plan)"), "{text}");
        assert!(
            text.contains(
                "execution_condition: waiting - planning is complete and execution can begin"
            ),
            "{text}"
        );
        assert!(text.contains("summary: goal_plan_state=proposed r2"), "{text}");
        assert!(
            text.contains("artifacts: goal_brief_ref=.boundline/briefs/goal.md; session_plan_brief_ref=.boundline/briefs/plan.md; latest_trace_ref=/tmp/workspace/.boundline/traces/task.json; plan_brief_ref=.boundline/governance/planning/discovery/brief.md; latest_governance_packet_ref=.canon/artifacts/R-20260522/discovery"),
            "{text}"
        );
        assert!(text.contains("latest_status: planned"), "{text}");
        assert!(text.contains("next_command: boundline run"), "{text}");
        assert!(!text.contains("context_provenance:"), "{text}");
        assert!(!text.contains("route_config_projection:"), "{text}");
    }

    #[test]
    fn render_session_status_brief_surfaces_framework_adapter_built_in_default() {
        let workspace = temp_output_workspace("output-framework-adapter-built-in");
        let view = SessionStatusView {
            session_id: "session-brief-adapter".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            latest_status: SessionStatus::GoalCaptured,
            explanation: "captured the goal and preserved built-in execution".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status_brief(&view);

        assert!(text.contains("framework_adapter_status: built_in_default"), "{text}");
        assert!(text.contains("framework_adapter_execution_source: built_in"), "{text}");
    }

    #[test]
    fn render_guidance_projection_brief_lines_keeps_story_without_source_dump() {
        let guidance = crate::domain::guidance::GuidanceGuardianProjection {
            capability_resolution_summary: Some(
                "resolved 2 guidance capability entries from 1 source(s)".to_string(),
            ),
            loaded_guardian_sources: vec!["assistant/packs/verification.toml".to_string()],
            guardian_timeline: vec!["verification-only: completed".to_string()],
            guardian_findings_summary: Some("no blocking guardian findings".to_string()),
            ..crate::domain::guidance::GuidanceGuardianProjection::default()
        };

        let lines = render_guidance_projection_brief_lines(&guidance);
        let rendered = lines.join("\n");

        assert!(rendered.contains("guidance_resolution_summary: resolved 2 guidance capability entries from 1 source(s)"), "{rendered}");
        assert!(rendered.contains("guardian_timeline: verification-only: completed"), "{rendered}");
        assert!(
            rendered.contains("guardian_findings_summary: no blocking guardian findings"),
            "{rendered}"
        );
        assert!(!rendered.contains("loaded_guardian_sources"), "{rendered}");
    }

    #[test]
    fn render_trace_summary_brief_stays_compact_and_surfaces_authoritative_fields() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
            goal: "Repair the broken checkout flow with the attached bounded brief".to_string(),
            authored_input_sources: vec![
                "attached_markdown: docs/checkout-spec.md".to_string(),
                "attached_markdown: docs/api-notes.md".to_string(),
            ],
            goal_brief_ref: Some(".boundline/briefs/goal.md".to_string()),
            session_plan_brief_ref: Some(".boundline/briefs/plan.md".to_string()),
            run_brief_ref: Some(".boundline/briefs/run.md".to_string()),
            routing_summary: Some(
                "routing: compatibility (execution_profile) - declarative manifest remains authoritative"
                    .to_string(),
            ),
            context_provenance: vec!["workspace_file: docs/checkout-spec.md".to_string()],
            latest_checkpoint_id: Some("checkpoint-7".to_string()),
            latest_checkpoint_scope: Some("workspace".to_string()),
            governance_next_action: Some("request Canon approval before continuing".to_string()),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::TaskNotCredible,
                "clarification is still required before the fix can continue",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary_brief(
            &summary,
            Some("latest-workspace-trace"),
            "boundline inspect --trace /tmp/workspace/.boundline/traces/task.json",
        );

        assert!(text.contains("inspection_target: latest-workspace-trace"), "{text}");
        assert!(text.contains("goal: Repair the broken checkout flow"), "{text}");
        assert!(
            text.contains("authored_input_sources: attached_markdown: docs/checkout-spec.md"),
            "{text}"
        );
        assert!(text.contains("routing: compatibility (execution_profile) - declarative manifest remains authoritative"), "{text}");
        assert!(text.contains("execution_condition: terminal - clarifi"), "{text}");
        assert!(text.contains("artifacts: goal_brief_ref=.boundline/briefs/goal.md; session_plan_brief_ref=.boundline/briefs/plan.md; run_brief_ref=.boundline/briefs/run.md; trace=/tmp/workspace/.boundline/traces/task.json; latest_checkpoint_id=checkpoint-7 (workspace)"), "{text}");
        assert!(
            text.contains(
                "governance: governance_next_action=request Canon approval before continuing"
            ),
            "{text}"
        );
        assert!(text.contains("latest_status: failed"), "{text}");
        assert!(
            text.contains(
                "next_command: boundline inspect --trace /tmp/workspace/.boundline/traces/task.json"
            ),
            "{text}"
        );
        assert!(!text.contains("context_provenance:"), "{text}");
        assert!(!text.contains("decision_timeline:"), "{text}");
    }

    #[test]
    fn render_trace_summary_includes_explicit_audit_mapping_lines() {
        let audit_projection = SessionAuditProjection::from_entries(
            "session-audit-1",
            vec![SessionAuditEntry::new_with_timestamp(
                "session-audit-1",
                1,
                1_717_000_000_000,
                SessionAuditEntryKind::TraceEventProjected,
                "decision verified: collected validation evidence",
                SessionAuditIdentity::default(),
                SessionAuditActor {
                    kind: SessionAuditActorKind::Agent,
                    id: "boundline-decision-loop".to_string(),
                    display_name: Some("Boundline Decision Loop".to_string()),
                    role: None,
                    runtime_kind: Some("copilot".to_string()),
                    provider: Some("copilot".to_string()),
                    route_slot: Some("implementation".to_string()),
                    model_name: Some("gpt-5.4".to_string()),
                    participant_routes: Vec::new(),
                    mixed_routes: false,
                },
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Run,
                    "decision_loop",
                    "run_with_options_and_context",
                ),
                SessionAuditOutcome::new(
                    SessionAuditOutcomeStatus::Succeeded,
                    "validation collected",
                ),
                SessionAuditSource {
                    kind: SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some("/tmp/.boundline/traces/task.json".to_string()),
                    trace_event_id: Some("event-1".to_string()),
                    trace_event_type: Some("decision_verified".to_string()),
                    step_id: Some("verify".to_string()),
                    plan_revision: Some(1),
                },
                json!({"summary": "collected validation evidence"}),
            )],
        );
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
            goal: "Validate the implementation path".to_string(),
            session_audit: Some(audit_projection),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "all checks passed",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "boundline next");

        assert!(text.contains("audit_entry_count: 1"), "{text}");
        assert!(
            text.contains(
                "audit_latest: event=decision_verified algorithm=run::decision_loop::run_with_options_and_context actor=agent:boundline-decision-loop outcome=succeeded"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "event=decision_verified algorithm=run::decision_loop::run_with_options_and_context actor=agent:boundline-decision-loop outcome=succeeded message=decision verified: collected validation evidence"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_trace_summary_surfaces_multi_route_audit_actor_projection() {
        let audit_projection = SessionAuditProjection::from_entries(
            "session-audit-2",
            vec![SessionAuditEntry::new_with_timestamp(
                "session-audit-2",
                1,
                1_717_000_000_000,
                SessionAuditEntryKind::TraceEventProjected,
                "review vote resolved: council accepted with mixed routes",
                SessionAuditIdentity::default(),
                SessionAuditActor {
                    kind: SessionAuditActorKind::Reviewer,
                    id: "review-council".to_string(),
                    display_name: Some("Review Council".to_string()),
                    role: Some("multi-reviewer".to_string()),
                    runtime_kind: Some("copilot".to_string()),
                    provider: Some("copilot".to_string()),
                    route_slot: Some("review".to_string()),
                    model_name: Some("gpt-5.4".to_string()),
                    participant_routes: vec![
                        "review:copilot:gpt-5.4".to_string(),
                        "adjudication:copilot:gpt-5.4".to_string(),
                    ],
                    mixed_routes: true,
                },
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Review,
                    "review_council",
                    "resolve_vote",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "review accepted"),
                SessionAuditSource {
                    kind: SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some("/tmp/.boundline/traces/task.json".to_string()),
                    trace_event_id: Some("event-2".to_string()),
                    trace_event_type: Some("review_vote_resolved".to_string()),
                    step_id: Some("review".to_string()),
                    plan_revision: Some(1),
                },
                json!({"summary": "council accepted with mixed routes"}),
            )],
        );
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
            goal: "Validate review attribution".to_string(),
            session_audit: Some(audit_projection),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "review passed",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "boundline next");

        assert!(
            text.contains(
                "participant_routes=review:copilot:gpt-5.4, adjudication:copilot:gpt-5.4"
            ),
            "{text}"
        );
        assert!(text.contains("mixed_routes=true"), "{text}");
    }

    #[test]
    fn render_trace_audit_summary_focuses_on_audit_projection() {
        let audit_projection = SessionAuditProjection::from_entries(
            "session-audit-3",
            vec![SessionAuditEntry::new_with_timestamp(
                "session-audit-3",
                1,
                1_717_000_000_000,
                SessionAuditEntryKind::TraceEventProjected,
                "review vote resolved: council accepted with mixed routes",
                SessionAuditIdentity::default(),
                SessionAuditActor {
                    kind: SessionAuditActorKind::Reviewer,
                    id: "review-council".to_string(),
                    display_name: Some("Review Council".to_string()),
                    role: Some("multi-reviewer".to_string()),
                    runtime_kind: Some("copilot".to_string()),
                    provider: Some("copilot".to_string()),
                    route_slot: Some("review".to_string()),
                    model_name: Some("gpt-5.4".to_string()),
                    participant_routes: vec![
                        "review:copilot:gpt-5.4".to_string(),
                        "adjudication:copilot:gpt-5.4".to_string(),
                    ],
                    mixed_routes: true,
                },
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Review,
                    "review_council",
                    "resolve_vote",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "review accepted"),
                SessionAuditSource {
                    kind: SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some("/tmp/.boundline/traces/task.json".to_string()),
                    trace_event_id: Some("event-3".to_string()),
                    trace_event_type: Some("review_vote_resolved".to_string()),
                    step_id: Some("review".to_string()),
                    plan_revision: Some(1),
                },
                json!({"summary": "council accepted with mixed routes"}),
            )],
        );
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
            goal: "Inspect audit only".to_string(),
            session_audit: Some(audit_projection),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "audit loaded",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_audit_summary(&summary, "latest-workspace-trace", "boundline next");

        assert!(text.contains("audit_session_ref: session-audit-3"), "{text}");
        assert!(text.contains("audit_outcomes: completed (1)"), "{text}");
        assert!(
            text.contains(
                "participant_routes=review:copilot:gpt-5.4, adjudication:copilot:gpt-5.4"
            ),
            "{text}"
        );
        assert!(!text.contains("route_owner:"), "{text}");
    }

    #[test]
    fn render_session_status_surfaces_delegation_projection() {
        let view = SessionStatusView {
            session_id: "session-delegation-status".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Repair blocked native continuity".to_string()),
            latest_status: SessionStatus::Planned,
            continuity_authority: Some(ContinuityAuthority::NativeSession),
            delegation: Some(crate::domain::session::DelegationStatusView {
                mode: crate::domain::session::DelegationContinuityMode::HandoffRequired,
                packet_id: Some("packet-1".to_string()),
                packet_kind: Some(crate::domain::session::DelegationPacketKind::Handoff),
                packet_state: Some(crate::domain::session::DelegationPacketState::Active),
                target_owner: Some("codex".to_string()),
                headline: "handoff required: implementation route cannot continue".to_string(),
                evidence_summary: "claude lacks continuation support for implementation"
                    .to_string(),
            }),
            next_command: Some("boundline status".to_string()),
            explanation: "delegated continuity is now authoritative".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status(&view);

        assert!(text.contains("delegation_mode: handoff_required"), "{text}");
        assert!(text.contains("delegation_packet_id: packet-1"), "{text}");
        assert!(text.contains("delegation_target_owner: codex"), "{text}");
        assert!(
            text.contains(
                "delegation_evidence_summary: claude lacks continuation support for implementation"
            ),
            "{text}"
        );
        assert!(text.contains("execution_condition: blocked - handoff required: implementation route cannot continue"), "{text}");
    }

    #[test]
    fn render_run_trace_surfaces_review_events() {
        let mut trace =
            ExecutionTrace::new("task-review-output", "session-review-output", "Render review");
        trace.record_event(
            TraceEventType::ReviewStarted,
            Some("review-safety".to_string()),
            0,
            json!({"review_trigger": "pr_ready"}),
        );
        trace.record_event(
            TraceEventType::ReviewerCompleted,
            Some("review-safety".to_string()),
            0,
            json!({
                "reviewer_id": "safety",
                "reviewer_role": "Safety",
                "finding": {
                    "disposition": "approve",
                    "summary": "No blockers"
                }
            }),
        );
        trace.record_event(
            TraceEventType::ReviewVoteResolved,
            Some("review-vote".to_string()),
            0,
            json!({"summary": "strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"}),
        );
        trace.record_event(
            TraceEventType::ReviewTerminalRecorded,
            Some("review-finalize".to_string()),
            0,
            json!({"review_outcome": "accepted"}),
        );

        let response = TaskRunResponse {
            task_id: "task-review-output".to_string(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            final_context: TaskContext::new(
                "session-review-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-review-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("review_trigger: pr_ready"), "{text}");
        assert!(text.contains("reviewer safety (Safety) approve: No blockers"), "{text}");
        assert!(
            text.contains(
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
            ),
            "{text}"
        );
        assert!(text.contains("review_outcome: accepted"), "{text}");
    }

    #[test]
    fn render_run_trace_surfaces_canon_memory_projection_from_governance_events() {
        let mut trace =
            ExecutionTrace::new("task-canon-output", "session-canon-output", "Render canon");
        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some("governance-step".to_string()),
            0,
            json!({
                "stage_key": "change:verify",
                "runtime": "canon",
                "required": true,
                "reason": "refresh_required",
                "run_ref": "run-8",
                "packet_ref": ".canon/runs/run-8",
                "document_refs": [".canon/runs/run-8/verification.md"],
                "canon_memory_summary": "Canon verification packet [stale]",
                "canon_memory_credibility": "stale",
                "canon_memory_compatibility": "warning",
                "canon_memory_reason_code": "refresh_required",
                "canon_next_action": "refresh: refresh the governed packet and reassess its credibility",
                "authority_provenance_lines": ["authority_control_class: council_review"],
                "adaptive_provenance_lines": ["adaptive_contract_line: adaptive-governance-v1"]
            }),
        );

        let response = TaskRunResponse {
            task_id: "task-canon-output".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::TaskNotCredible,
                "governed work is blocked pending intervention",
                None,
            ),
            final_context: TaskContext::new(
                "session-canon-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-canon-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("context_summary: Canon verification packet [stale]"), "{text}");
        assert!(text.contains("context_credibility: stale"), "{text}");
        assert!(
            text.contains("context_primary_inputs: .canon/runs/run-8/verification.md"),
            "{text}"
        );
        assert!(
            text.contains("context_provenance: canon_memory: Canon verification packet [stale]"),
            "{text}"
        );
        assert!(text.contains("canon_memory_compatibility: warning"), "{text}");
        assert!(text.contains("canon_memory_run_ref: run-8"), "{text}");
        assert!(text.contains("canon_memory_packet: .canon/runs/run-8"), "{text}");
        assert!(text.contains("canon_memory_reason: refresh_required"), "{text}");
        assert!(text.contains("authority_control_class: council_review"), "{text}");
        assert!(text.contains("adaptive_contract_line: adaptive-governance-v1"), "{text}");
        assert!(
            text.contains(
                "canon_memory_next_action: refresh: refresh the governed packet and reassess its credibility"
            ),
            "{text}"
        );
        assert!(text.contains("context_staleness_reason: refresh_required"), "{text}");
        assert!(
            text.contains(
                "governance_next_action: refresh: refresh the governed packet and reassess its credibility"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_run_trace_includes_reasoning_profile_projection() {
        let mut trace = ExecutionTrace::new(
            "task-reasoning-output",
            "session-reasoning-output",
            "Render reasoning output",
        );
        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some("governance-step".to_string()),
            0,
            json!({
                "stage_key": "bug-fix:investigate",
                "runtime": "canon",
                "reason": "distinct_routes=1 < required=2",
                "reasoning_profile_record": {
                    "activation_id": "reasoning-attempt-1",
                    "stage_key": "bug-fix:investigate",
                    "profile_id": "independent_pair_review",
                    "trigger": "operator_policy",
                    "activation_reason": "stage governance activated stronger challenge",
                    "status": "blocked",
                    "participants": [
                        {
                            "role_id": "reviewer_primary",
                            "participant_id": "independent_pair_review-reviewer_primary",
                            "effective_route": "reviewer_roles.reviewer_primary:claude:sonnet-4",
                            "provider_family": "claude",
                            "context_basis": "governance_stage:bug-fix:investigate",
                            "prompting_pattern": "blind_reviewer",
                            "status": "pending",
                            "result_summary": null
                        },
                        {
                            "role_id": "reviewer_secondary",
                            "participant_id": "independent_pair_review-reviewer_secondary",
                            "effective_route": "review:codex:o4-mini",
                            "provider_family": "codex",
                            "context_basis": "governance_stage:bug-fix:investigate",
                            "prompting_pattern": "blind_reviewer",
                            "status": "pending",
                            "result_summary": null
                        }
                    ],
                    "budget": {
                        "max_participants": 2,
                        "max_branches": 1,
                        "max_debate_rounds": 0,
                        "max_reflexion_revisions": 0,
                        "max_calls": 2,
                        "max_tokens": 8000,
                        "max_adjudication_steps": 1
                    },
                    "posture": {
                        "contract_line": "governed_reasoning_posture_v1",
                        "compatibility_window": {
                            "boundline_min": "0.62.0",
                            "boundline_max_exclusive": "0.63.0",
                            "canon_min": "0.59.0",
                            "canon_max_exclusive": "0.61.0",
                            "contract_line": "governed_reasoning_posture_v1"
                        },
                        "required_profile_family": "blind_review",
                        "required_profile_id": "independent_pair_review",
                        "minimum_independence": {
                            "route_distinct": true,
                            "provider_distinct": true,
                            "context_distinct": false,
                            "prompt_pattern_distinct": false,
                            "minimum_participants": 2
                        },
                        "admission_priority": "required_before_continue",
                        "confidence_handoff_required": true,
                        "provenance_ref": "governance_attempt:attempt-1"
                    },
                    "independence": {
                        "requested_floor": {
                            "route_distinct": true,
                            "provider_distinct": true,
                            "context_distinct": false,
                            "prompt_pattern_distinct": false,
                            "minimum_participants": 2
                        },
                        "observed_distinctions": {
                            "distinct_routes": 1,
                            "distinct_providers": 2,
                            "distinct_contexts": 1,
                            "distinct_prompt_patterns": 1
                        },
                        "result": "failed",
                        "reason": "distinct_routes=1 < required=2"
                    },
                    "outcome": {
                        "outcome_kind": "blocked",
                        "headline": "independent pair review blocked",
                        "disagreement_summary": "reviewers collapsed onto one route",
                        "next_action": "configure distinct reviewer routes",
                        "iterations": []
                    },
                    "confidence": {
                        "confidence_level": "low",
                        "basis": [
                            "independence=failed",
                            "posture_contract=governed_reasoning_posture_v1"
                        ],
                        "admission_effect": "gate",
                        "summary": "reasoning independence failed; block progression until challenge distinctness is restored"
                    }
                }
            }),
        );

        let response = TaskRunResponse {
            task_id: "task-reasoning-output".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::TaskNotCredible,
                "governed work is blocked pending intervention",
                None,
            ),
            final_context: TaskContext::new(
                "session-reasoning-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-reasoning-output.json"
                .to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("reasoning_profile_id: independent_pair_review"), "{text}");
        assert!(text.contains("reasoning_profile_status: blocked"), "{text}");
        assert!(text.contains("reasoning_independence_result: failed"), "{text}");
        assert!(
            text.contains("reasoning_posture_contract: governed_reasoning_posture_v1"),
            "{text}"
        );
        assert!(text.contains("reasoning_confidence_level: low"), "{text}");
        assert!(
            text.contains("reasoning_next_action: configure distinct reviewer routes"),
            "{text}"
        );
    }

    #[test]
    fn render_session_status_surfaces_review_projection() {
        let view = SessionStatusView {
            session_id: "session-review-status".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            session_started_at: None,
            goal: Some("Ship review output".to_string()),
            advanced_context: None,
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: None,
            context_provenance: None,
            context_staleness_reason: None,
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: None,
            clarification_questions: None,
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            active_flow: None,
            flow_state: None,
            goal_plan_state: None,
            goal_plan_revision: None,
            planning_rationale: None,
            verification_strategy: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            delegation: None,
            compatibility_follow_up: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Running,
            execution_path: Some("fixture_compatibility".to_string()),
            goal_brief_ref: None,
            session_plan_brief_ref: None,
            run_brief_ref: None,
            latest_trace_ref: None,
            latest_decision_status: None,
            latest_decision_target: None,
            latest_changed_files: None,
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            latest_workspace_slice: None,
            latest_selection_headline: None,
            latest_candidate_family: None,
            latest_selection_reason: None,
            latest_rejected_candidates: None,
            latest_attempt_lineage: None,
            latest_validation_status: Some("passed".to_string()),
            latest_exhaustion_reason: None,
            latest_review_trigger: Some("pr_ready".to_string()),
            latest_review_vote: Some(
                "strategy=majority approvals=2 concerns=0 blocks=0 decision=accepted".to_string(),
            ),
            latest_review_outcome: Some("accepted".to_string()),
            latest_review_council_profile: Some("yellow_pair".to_string()),
            latest_review_independence_state: Some("passed".to_string()),
            latest_review_stop_semantics: Some("council_required".to_string()),
            latest_review_selection_summary: Some(
                "profile=yellow_pair quorum=met independence=passed selected_roles=Safety, Maintainability"
                    .to_string(),
            ),
            latest_review_headline: Some("safety approve: No blockers".to_string()),
            latest_governance_stage: Some("bug-fix:implement".to_string()),
            latest_governance_runtime: Some("canon".to_string()),
            latest_governance_mode: Some("implementation".to_string()),
            latest_governance_run_ref: Some("canon-run-1".to_string()),
            latest_governance_state: Some("awaiting_approval".to_string()),
            latest_governance_runtime_state: None,
            latest_governance_rollout_profile: None,
            latest_governance_reason: None,
            latest_governance_contract_lines: None,
            latest_governance_approval_provenance: None,
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: Some(".canon/runs/canon-run-1".to_string()),
            latest_governance_packet_source_stage: Some("bug-fix:investigate".to_string()),
            latest_governance_packet_binding_reason: Some("upstream_stage_context".to_string()),
            latest_governance_approval: Some("requested".to_string()),
            latest_governance_decision: Some(
                "await approval for governed implementation".to_string(),
            ),
            latest_governance_candidates: Some(vec![
                "await_approval".to_string(),
                "block_stage".to_string(),
            ]),
            latest_governance_confidence_level: None,
            latest_governance_admission_effect: None,
            latest_governance_confidence_summary: None,
            governance_next_action: Some(
                "wait for approval and rerun boundline status".to_string(),
            ),
            governance_lifecycle_runtime: None,
            governance_lifecycle_opt_out: None,
            governance_lifecycle_mode_selection: None,
            governance_lifecycle_selected_mode: None,
            governance_lifecycle_selected_mode_sequence: None,
            latest_reasoning_profile: None,
            project_scale_path: None,
            project_scale_current_stage: None,
            project_scale_next_action: None,
            project_scale_checkpoint_refs: None,
            latest_voting_trigger: None,
            latest_voting_result: None,
            latest_voting_adjudication: None,
            latest_voting_reviewed_evidence: None,
            latest_voting_blocking: None,
            latest_voting_next_action: None,
            delight_feedback: None,
            next_command: Some("boundline step".to_string()),
            explanation: "review is in progress".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status(&view);

        assert!(
            text.contains(
                "routing: compatibility (execution_profile) - compatibility execution remains active from the persisted task"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "execution_condition: waiting - governance approval is still pending before execution can continue"
            ),
            "{text}"
        );
        assert!(text.contains("latest_review_trigger: pr_ready"), "{text}");
        assert!(text.contains("latest_review_vote: strategy=majority approvals=2 concerns=0 blocks=0 decision=accepted"), "{text}");
        assert!(text.contains("latest_review_outcome: accepted"), "{text}");
        assert!(text.contains("latest_review_council_profile: yellow_pair"), "{text}");
        assert!(text.contains("latest_review_independence_state: passed"), "{text}");
        assert!(text.contains("latest_review_stop_semantics: council_required"), "{text}");
        assert!(
            text.contains(
                "latest_review_selection_summary: profile=yellow_pair quorum=met independence=passed selected_roles=Safety, Maintainability"
            ),
            "{text}"
        );
        assert!(text.contains("latest_review_headline: safety approve: No blockers"), "{text}");
        assert!(text.contains("latest_governance_mode: implementation"), "{text}");
        assert!(text.contains("latest_governance_run_ref: canon-run-1"), "{text}");
        assert!(text.contains("latest_governance_state: awaiting_approval"), "{text}");
        assert!(text.contains("execution_path: fixture_compatibility"), "{text}");
        assert!(
            text.contains("latest_governance_candidates: await_approval, block_stage"),
            "{text}"
        );
        assert!(
            text.contains("governance_next_action: wait for approval and rerun boundline status"),
            "{text}"
        );
    }

    #[test]
    fn render_session_status_includes_reasoning_profile_projection() {
        let view = SessionStatusView {
            session_id: "session-reasoning".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            explanation: "reasoning profile is active".to_string(),
            latest_reasoning_profile: Some(crate::domain::reasoning::ProfileActivationRecord {
                activation_id: "reasoning-attempt-1".to_string(),
                stage_key: "bug-fix:investigate".to_string(),
                profile_id: crate::domain::reasoning::ReasoningProfileId::IndependentPairReview,
                trigger: crate::domain::reasoning::ReasoningActivationTrigger::OperatorPolicy,
                activation_reason: "stage governance activated stronger challenge".to_string(),
                status: crate::domain::reasoning::ReasoningActivationStatus::Blocked,
                participants: vec![
                    crate::domain::reasoning::ParticipantAssignment {
                        role_id: "reviewer_primary".to_string(),
                        participant_id: "independent_pair_review-reviewer_primary".to_string(),
                        effective_route: "reviewer_roles.reviewer_primary:claude:sonnet-4"
                            .to_string(),
                        provider_family: Some("claude".to_string()),
                        context_basis: "governance_stage:bug-fix:investigate".to_string(),
                        prompting_pattern: "blind_reviewer".to_string(),
                        status: crate::domain::reasoning::ReasoningParticipantStatus::Pending,
                        result_summary: None,
                    },
                    crate::domain::reasoning::ParticipantAssignment {
                        role_id: "reviewer_secondary".to_string(),
                        participant_id: "independent_pair_review-reviewer_secondary".to_string(),
                        effective_route: "review:codex:o4-mini".to_string(),
                        provider_family: Some("codex".to_string()),
                        context_basis: "governance_stage:bug-fix:investigate".to_string(),
                        prompting_pattern: "blind_reviewer".to_string(),
                        status: crate::domain::reasoning::ReasoningParticipantStatus::Pending,
                        result_summary: None,
                    },
                ],
                budget: crate::domain::reasoning::ReasoningBudget {
                    max_participants: 2,
                    max_branches: 1,
                    max_debate_rounds: 0,
                    max_reflexion_revisions: 0,
                    max_calls: 2,
                    max_tokens: 8_000,
                    max_adjudication_steps: 1,
                },
                posture: None,
                independence: Some(crate::domain::reasoning::IndependenceAssessment {
                    requested_floor: crate::domain::reasoning::IndependenceFloor {
                        route_distinct: true,
                        provider_distinct: true,
                        context_distinct: false,
                        prompt_pattern_distinct: false,
                        minimum_participants: 2,
                    },
                    observed_distinctions:
                        crate::domain::reasoning::ReasoningObservedDistinctness {
                            distinct_routes: 1,
                            distinct_providers: 2,
                            distinct_contexts: 1,
                            distinct_prompt_patterns: 1,
                        },
                    result: crate::domain::reasoning::IndependenceAssessmentResult::Failed,
                    reason: "distinct_routes=1 < required=2".to_string(),
                }),
                outcome: Some(crate::domain::reasoning::ReasoningOutcome {
                    outcome_kind: crate::domain::reasoning::ReasoningOutcomeKind::Blocked,
                    headline: "independent pair review blocked".to_string(),
                    disagreement_summary: Some("reviewers collapsed onto one route".to_string()),
                    next_action: Some("configure distinct reviewer routes".to_string()),
                    iterations: Vec::new(),
                }),
                confidence: Some(crate::domain::reasoning::ReasoningConfidenceContribution {
                    confidence_level: crate::domain::reasoning::ReasoningConfidenceLevel::Low,
                    basis: vec!["independence=failed".to_string()],
                    admission_effect: crate::domain::reasoning::ReasoningAdmissionEffect::Gate,
                    summary: "reasoning independence failed; block progression until challenge distinctness is restored".to_string(),
                }),
            }),
            ..SessionStatusView::default()
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_reasoning_profile_id: independent_pair_review"), "{text}");
        assert!(text.contains("latest_reasoning_profile_status: blocked"), "{text}");
        assert!(
            text.contains(
                "latest_reasoning_selection_reason: stage governance activated stronger challenge"
            ),
            "{text}"
        );
        assert!(text.contains("latest_reasoning_independence_result: failed"), "{text}");
        assert!(text.contains("latest_reasoning_confidence_level: low"), "{text}");
        assert!(
            text.contains("latest_reasoning_contribution: independent pair review blocked"),
            "{text}"
        );
        assert!(
            text.contains(
                "latest_reasoning_fallback_disclosure: reasoning profile is blocked: stage governance activated stronger challenge"
            ),
            "{text}"
        );
        assert!(
            text.contains("latest_reasoning_next_action: configure distinct reviewer routes"),
            "{text}"
        );
        assert!(text.contains("latest_reasoning_participants: reviewer_primary=reviewer_roles.reviewer_primary:claude:sonnet-4"), "{text}");
        assert!(text.contains("confidence_level: low"), "{text}");
        assert!(text.contains("next_best_action: configure distinct reviewer routes"), "{text}");
        assert!(
            text.contains("explain_plan_validation: configure distinct reviewer routes"),
            "{text}"
        );
        assert!(
            text.contains("explain_plan_recovery: configure distinct reviewer routes"),
            "{text}"
        );
        assert!(
            text.contains(
                "explain_plan_governance: reasoning_profile=independent_pair_review; status=blocked; confidence=low; admission_effect=gate"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_session_status_surfaces_project_scale_voting_and_governance_lifecycle() {
        let text = render_session_status(&SessionStatusView {
            session_id: "session-project-scale-status".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Deliver bounded project-scale workflow".to_string()),
            latest_status: SessionStatus::Running,
            project_scale_path: Some("system_modification".to_string()),
            project_scale_current_stage: Some("security_assessment".to_string()),
            project_scale_next_action: Some("repair_context".to_string()),
            project_scale_checkpoint_refs: Some(vec![
                "checkpoint-a".to_string(),
                "checkpoint-b".to_string(),
            ]),
            latest_voting_trigger: Some("governance boundary reached".to_string()),
            latest_voting_result: Some("rejected".to_string()),
            latest_voting_adjudication: Some("human escalation required".to_string()),
            latest_voting_reviewed_evidence: Some("governance packet, execution trace".to_string()),
            latest_voting_blocking: Some(true),
            latest_voting_next_action: Some("collect missing evidence".to_string()),
            governance_lifecycle_runtime: Some("canon".to_string()),
            governance_lifecycle_opt_out: Some(false),
            governance_lifecycle_mode_selection: Some("manual".to_string()),
            governance_lifecycle_selected_mode: Some("implementation".to_string()),
            governance_lifecycle_selected_mode_sequence: Some(vec![
                "requirements".to_string(),
                "architecture".to_string(),
                "backlog".to_string(),
                "implementation".to_string(),
            ]),
            governance_next_action: Some("refresh governance packet".to_string()),
            explanation: "project scale and voting metadata are active".to_string(),
            ..SessionStatusView::default()
        });

        assert!(text.contains("project_scale_path: system_modification"), "{text}");
        assert!(text.contains("project_scale_current_stage: security_assessment"), "{text}");
        assert!(text.contains("project_scale_next_action: repair_context"), "{text}");
        assert!(
            text.contains("project_scale_checkpoint_refs: checkpoint-a, checkpoint-b"),
            "{text}"
        );
        assert!(text.contains("latest_voting_trigger: governance boundary reached"), "{text}");
        assert!(text.contains("latest_voting_result: rejected"), "{text}");
        assert!(text.contains("latest_voting_adjudication: human escalation required"), "{text}");
        assert!(
            text.contains("latest_voting_reviewed_evidence: governance packet, execution trace"),
            "{text}"
        );
        assert!(text.contains("latest_voting_blocking: true"), "{text}");
        assert!(text.contains("latest_voting_next_action: collect missing evidence"), "{text}");
        assert!(text.contains("governance_next_action: refresh governance packet"), "{text}");
        assert!(text.contains("governance_lifecycle_runtime: canon"), "{text}");
        assert!(text.contains("governance_lifecycle_opt_out: false"), "{text}");
        assert!(text.contains("governance_lifecycle_mode_selection: manual"), "{text}");
        assert!(text.contains("governance_lifecycle_selected_mode: implementation"), "{text}");
        assert!(
            text.contains(
                "governance_lifecycle_selected_mode_sequence: requirements, architecture, backlog, implementation"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_trace_summary_includes_review_timeline() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-review-output.json".to_string(),
            goal: "Render trace summary".to_string(),
            trace_started_at: None,
            advanced_context: None,
            guidance_guardian: crate::domain::guidance::GuidanceGuardianProjection::default(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            routing_summary: None,
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            goal_brief_ref: None,
            session_plan_brief_ref: None,
            run_brief_ref: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: Vec::new(),
            context_provenance: Vec::new(),
            context_staleness_reason: None,
            plan_quality_state: None,
            plan_quality_findings: Vec::new(),
            plan_quality_assumptions: Vec::new(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
            executed_steps: vec![],
            recovery_events: vec![],
            governance_timeline: vec![
                "governance_selected: bug-fix:implement -> canon".to_string(),
                "governance_awaiting_approval: bug-fix:implement (requested)".to_string(),
            ],
            governance_runtime_state: None,
            governance_rollout_profile: None,
            governance_reason: None,
            governance_approval_provenance: None,
            governance_next_action: Some(
                "wait for approval and rerun boundline status".to_string(),
            ),
            reasoning_profile: None,
            delegation: None,
            inspect_context: None,
            inspect_council: None,
            inspect_timeline: None,
            review_timeline: vec![
                "review_trigger: pr_ready".to_string(),
                "reviewer safety (Safety) approve: No blockers".to_string(),
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
                    .to_string(),
                "review_outcome: accepted".to_string(),
            ],
            session_audit: None,
            delight_feedback: None,
            framework_adapter_stage_routing: None,
            framework_adapter_hook_dispatch: None,
            framework_adapter_stage_failure: None,
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            duration: Some(42),
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("governance_selected: bug-fix:implement -> canon"), "{text}");
        assert!(
            text.contains("governance_next_action: wait for approval and rerun boundline status"),
            "{text}"
        );
        assert!(text.contains("review_trigger: pr_ready"), "{text}");
        assert!(text.contains("review_outcome: accepted"), "{text}");
    }

    #[test]
    fn render_trace_summary_includes_reasoning_profile_projection() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/reasoning-trace.json".to_string(),
            goal: "Render reasoning trace summary".to_string(),
            reasoning_profile: Some(crate::domain::reasoning::ProfileActivationRecord {
                activation_id: "reasoning-attempt-2".to_string(),
                stage_key: "bug-fix:verify".to_string(),
                profile_id: crate::domain::reasoning::ReasoningProfileId::BoundedReflexion,
                trigger:
                    crate::domain::reasoning::ReasoningActivationTrigger::CanonRequiredChallenge,
                activation_reason: "Canon governance activated stronger challenge".to_string(),
                status: crate::domain::reasoning::ReasoningActivationStatus::Degraded,
                participants: Vec::new(),
                budget: crate::domain::reasoning::ReasoningBudget {
                    max_participants: 1,
                    max_branches: 1,
                    max_debate_rounds: 0,
                    max_reflexion_revisions: 2,
                    max_calls: 2,
                    max_tokens: 6_000,
                    max_adjudication_steps: 1,
                },
                posture: Some(crate::domain::reasoning::CanonChallengePostureInput {
                    contract_line: "governed_reasoning_posture_v1".to_string(),
                    compatibility_window: crate::domain::reasoning::ReasoningCompatibilityWindow {
                        boundline_min: "0.62.0".to_string(),
                        boundline_max_exclusive: "0.63.0".to_string(),
                        canon_min: "0.59.0".to_string(),
                        canon_max_exclusive: "0.61.0".to_string(),
                        contract_line: "governed_reasoning_posture_v1".to_string(),
                    },
                    required_profile_family: Some(
                        crate::domain::reasoning::ReasoningProfileFamily::Reflexion,
                    ),
                    required_profile_id: Some(
                        crate::domain::reasoning::ReasoningProfileId::BoundedReflexion,
                    ),
                    minimum_independence: crate::domain::reasoning::IndependenceFloor {
                        route_distinct: false,
                        provider_distinct: false,
                        context_distinct: false,
                        prompt_pattern_distinct: false,
                        minimum_participants: 1,
                    },
                    admission_priority:
                        crate::domain::reasoning::CanonAdmissionPriority::RequiredBeforeContinue,
                    confidence_handoff_required: true,
                    provenance_ref: "governance_attempt:attempt-2".to_string(),
                }),
                independence: Some(crate::domain::reasoning::IndependenceAssessment {
                    requested_floor: crate::domain::reasoning::IndependenceFloor {
                        route_distinct: false,
                        provider_distinct: false,
                        context_distinct: false,
                        prompt_pattern_distinct: false,
                        minimum_participants: 1,
                    },
                    observed_distinctions:
                        crate::domain::reasoning::ReasoningObservedDistinctness {
                            distinct_routes: 1,
                            distinct_providers: 1,
                            distinct_contexts: 1,
                            distinct_prompt_patterns: 1,
                        },
                    result: crate::domain::reasoning::IndependenceAssessmentResult::Degraded,
                    reason: "reflexion remained bounded but shared one runtime".to_string(),
                }),
                outcome: Some(crate::domain::reasoning::ReasoningOutcome {
                    outcome_kind: crate::domain::reasoning::ReasoningOutcomeKind::Degraded,
                    headline: "bounded reflexion degraded".to_string(),
                    disagreement_summary: Some("shared runtime reduced independence".to_string()),
                    next_action: Some(
                        "escalate to blind review if confidence remains low".to_string(),
                    ),
                    iterations: Vec::new(),
                }),
                confidence: Some(crate::domain::reasoning::ReasoningConfidenceContribution {
                    confidence_level: crate::domain::reasoning::ReasoningConfidenceLevel::Medium,
                    basis: vec![
                        "independence=degraded".to_string(),
                        "posture_contract=governed_reasoning_posture_v1".to_string(),
                    ],
                    admission_effect: crate::domain::reasoning::ReasoningAdmissionEffect::Warn,
                    summary: "reasoning independence degraded; continue only with explicit caution"
                        .to_string(),
                }),
            }),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("reasoning_profile_id: bounded_reflexion"), "{text}");
        assert!(text.contains("reasoning_profile_status: degraded"), "{text}");
        assert!(
            text.contains(
                "reasoning_selection_reason: Canon governance activated stronger challenge"
            ),
            "{text}"
        );
        assert!(text.contains("reasoning_independence_result: degraded"), "{text}");
        assert!(
            text.contains("reasoning_posture_contract: governed_reasoning_posture_v1"),
            "{text}"
        );
        assert!(text.contains("reasoning_confidence_level: medium"), "{text}");
        assert!(text.contains("reasoning_contribution: bounded reflexion degraded"), "{text}");
        assert!(
            text.contains(
                "reasoning_fallback_disclosure: reasoning profile is degraded: Canon governance activated stronger challenge"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "reasoning_next_action: escalate to blind review if confidence remains low"
            ),
            "{text}"
        );
        assert!(
            text.contains("next_best_action: escalate to blind review if confidence remains low"),
            "{text}"
        );
        assert!(
            text.contains(
                "explain_plan_validation: escalate to blind review if confidence remains low"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "explain_plan_governance: reasoning_profile=bounded_reflexion; status=degraded; confidence=medium; admission_effect=warn; posture_contract=governed_reasoning_posture_v1"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_trace_summary_surfaces_delegation_projection() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-delegation.json".to_string(),
            goal: "Render delegation trace summary".to_string(),
            delegation: Some(crate::domain::session::DelegationStatusView {
                mode: crate::domain::session::DelegationContinuityMode::EscalationRequired,
                packet_id: Some("packet-2".to_string()),
                packet_kind: Some(crate::domain::session::DelegationPacketKind::Escalation),
                packet_state: Some(crate::domain::session::DelegationPacketState::Active),
                target_owner: Some("operator".to_string()),
                headline: "escalation required: no declared continuation path remains".to_string(),
                evidence_summary: "all declared routes are blocked by capability policy"
                    .to_string(),
            }),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::TaskNotCredible,
                "escalation required: no declared continuation path remains",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("delegation_mode: escalation_required"), "{text}");
        assert!(text.contains("delegation_packet_id: packet-2"), "{text}");
        assert!(text.contains("delegation_target_owner: operator"), "{text}");
        assert!(text.contains("execution_condition: blocked - escalation required: no declared continuation path remains"), "{text}");
        assert!(
            text.contains("follow_through_evidence_source: trace:delegation_packet:packet-2"),
            "{text}"
        );
    }

    #[test]
    fn render_trace_summary_includes_guidance_projection_lines() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-guidance.json".to_string(),
            goal: "Render guidance trace summary".to_string(),
            guidance_guardian: crate::domain::guidance::GuidanceGuardianProjection {
                capability_resolution_summary: Some(
                    "resolved 1 guidance capability entries from 1 source(s) for verification"
                        .to_string(),
                ),
                loaded_packs: vec![
                    "assistant/packs/guidance-catalog (pack=boundline-guidance-catalog, catalog=boundline-guidance-catalog)".to_string(),
                ],
                skipped_packs: vec![
                    "assistant/packs/legacy-pack (failed to read catalog manifest)".to_string(),
                ],
                catalog_validation_findings: vec![
                    "warning: assistant/packs/guidance-catalog/catalog/guidance-index.toml (legacy alias normalized)".to_string(),
                ],
                loaded_guidance_sources: vec![
                    "assistant/packs/shared/guidance/clean-code.md".to_string(),
                ],
                skipped_guidance_sources: vec![".canon/boundline/guidance (missing)".to_string()],
                loaded_guardian_sources: vec![".boundline/guardians/verification.toml".to_string()],
                skipped_guardian_sources: vec![
                    "assistant/packs/shared/guardians/verification.toml (shadowed)".to_string(),
                ],
                guardian_timeline: vec!["verification_guardian: completed".to_string()],
                guardian_findings_summary: Some(
                    "1 guardian finding(s); blocking=false".to_string(),
                ),
                guardian_findings: vec![crate::domain::guidance::GuardianFinding {
                    finding_id: "finding-1".to_string(),
                    guardian_id: "verification_guardian".to_string(),
                    rule_id: "verification".to_string(),
                    disposition: crate::domain::guidance::GuardianDisposition::Warn,
                    summary: "verification evidence is stale".to_string(),
                    evidence_refs: vec!["tests/red_to_green.rs".to_string()],
                    confidence: crate::domain::guidance::FindingConfidence::Medium,
                    recommended_action: "rerun the bounded verification command".to_string(),
                    authority_source:
                        crate::domain::guidance::GuidanceAuthoritySource::WorkspaceOverride,
                    source_ref: ".boundline/guardians/verification.toml".to_string(),
                    phase: crate::domain::guidance::CapabilityPhase::Verification,
                }],
                guardian_degradations: vec!["verification route unavailable".to_string()],
                guardian_blocking_outcome: Some(
                    "guardian findings recorded without a blocking outcome".to_string(),
                ),
            },
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("guidance_resolution_summary: resolved 1 guidance capability entries from 1 source(s) for verification"), "{text}");
        assert!(
            text.contains("loaded_packs: assistant/packs/guidance-catalog (pack=boundline-guidance-catalog, catalog=boundline-guidance-catalog)"),
            "{text}"
        );
        assert!(
            text.contains(
                "skipped_packs: assistant/packs/legacy-pack (failed to read catalog manifest)"
            ),
            "{text}"
        );
        assert!(
            text.contains("catalog_validation_findings: warning: assistant/packs/guidance-catalog/catalog/guidance-index.toml (legacy alias normalized)"),
            "{text}"
        );
        assert!(
            text.contains("loaded_guidance_sources: assistant/packs/shared/guidance/clean-code.md"),
            "{text}"
        );
        assert!(
            text.contains("loaded_guardian_sources: .boundline/guardians/verification.toml"),
            "{text}"
        );
        assert!(text.contains("guardian_timeline: verification_guardian: completed"), "{text}");
        assert!(
            text.contains("guardian_findings_summary: 1 guardian finding(s); blocking=false"),
            "{text}"
        );
        assert!(
            text.contains(
                "guardian_findings: verification_guardian:warn:verification evidence is stale"
            ),
            "{text}"
        );
        assert!(text.contains("guardian_degradations: verification route unavailable"), "{text}");
        assert!(
            text.contains(
                "guardian_blocking_outcome: guardian findings recorded without a blocking outcome"
            ),
            "{text}"
        );
    }

    #[test]
    fn render_run_trace_prefers_task_started_context_and_covers_retry_fallbacks() {
        let mut trace = ExecutionTrace::new("task-context", "session-context", "Render context");
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            0,
            json!({
                "input": {
                    "context_summary": "bounded context from src/lib.rs",
                    "context_credibility": "stale",
                    "context_primary_inputs": ["src/lib.rs"],
                    "context_provenance": ["workspace_file: src/lib.rs (failing test target) [source=symbol_scan]"],
                    "context_staleness_reason": "trace snapshot is stale"
                }
            }),
        );
        trace.record_event(TraceEventType::RetryScheduled, None, 0, json!({}));
        trace.record_event(TraceEventType::Replanned, None, 0, json!({}));
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            json!({"goal": "Render context"}),
        );
        trace.record_event(TraceEventType::FlowInferred, None, 0, json!({"flow_name": "bug-fix"}));

        let response = TaskRunResponse {
            task_id: "task-context".to_string(),
            terminal_status: TaskStatus::Running,
            terminal_reason: TerminalReason::new(
                TerminalCondition::NoCredibleNextStep,
                "waiting for approval",
                None,
            ),
            final_context: TaskContext::new(
                "session-context",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-context.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/boundline-next");

        assert!(text.contains("context_summary: bounded context from src/lib.rs"), "{text}");
        assert!(text.contains("context_credibility: stale"), "{text}");
        assert!(text.contains("context_primary_inputs: src/lib.rs"), "{text}");
        assert!(
            text.contains("context_provenance: workspace_file: src/lib.rs (failing test target)"),
            "{text}"
        );
        assert!(text.contains("context_staleness_reason: trace snapshot is stale"), "{text}");
        assert!(text.contains("retry for unknown-step: retry scheduled"), "{text}");
        assert!(text.contains("replan after unknown-step: replan scheduled"), "{text}");
        assert!(text.contains("goal plan created: Render context"), "{text}");
        assert!(text.contains("flow inferred: bug-fix"), "{text}");
        assert!(text.contains("execution_condition: waiting - waiting for approval"), "{text}");
    }

    #[test]
    fn render_trace_summary_covers_retry_and_replan_labels() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-output.json".to_string(),
            goal: "Render retry labels".to_string(),
            recovery_events: vec![
                TraceRecoveryEvent {
                    event_type: TraceEventType::RetryScheduled,
                    trigger: "verify failed".to_string(),
                    related_step_id: Some("verify".to_string()),
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::Replanned,
                    trigger: "replan scheduled".to_string(),
                    related_step_id: Some("verify".to_string()),
                },
            ],
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            ..TraceSummaryView::default()
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

        assert!(text.contains("retry: verify failed"), "{text}");
        assert!(text.contains("replan: replan scheduled"), "{text}");
    }

    #[test]
    fn render_session_status_covers_recovery_metadata_and_exhaustion_reason() {
        let view = SessionStatusView {
            session_id: "session-exhausted".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            latest_status: SessionStatus::Exhausted,
            latest_changed_files: Some(vec!["src/lib.rs".to_string()]),
            latest_workspace_slice: Some("src/lib.rs".to_string()),
            latest_selection_headline: Some("selected src/lib.rs".to_string()),
            latest_candidate_family: Some("source".to_string()),
            latest_selection_reason: Some("failing test evidence".to_string()),
            latest_rejected_candidates: Some(vec!["tests/red.rs".to_string()]),
            latest_attempt_lineage: Some("attempt-2 retried_from attempt-1".to_string()),
            latest_validation_status: Some("failed".to_string()),
            latest_exhaustion_reason: Some("limits exhausted".to_string()),
            next_command: Some("boundline inspect".to_string()),
            explanation: "session exhausted after bounded retries".to_string(),
            ..SessionStatusView::default()
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_changed_files: src/lib.rs"), "{text}");
        assert!(text.contains("latest_workspace_slice: src/lib.rs"), "{text}");
        assert!(text.contains("latest_selection_headline: selected src/lib.rs"), "{text}");
        assert!(text.contains("latest_candidate_family: source"), "{text}");
        assert!(text.contains("latest_selection_reason: failing test evidence"), "{text}");
        assert!(text.contains("latest_rejected_candidates: tests/red.rs"), "{text}");
        assert!(
            text.contains("latest_attempt_lineage: attempt-2 retried_from attempt-1"),
            "{text}"
        );
        assert!(text.contains("latest_validation_status: failed"), "{text}");
        assert!(text.contains("latest_exhaustion_reason: limits exhausted"), "{text}");
        assert!(text.contains("execution_condition: terminal - limits exhausted"), "{text}");
    }

    #[test]
    fn output_surfaces_latest_checkpoint_projection_lines() {
        let mut trace = ExecutionTrace::new(
            "task-checkpoint",
            "session-checkpoint",
            "Render checkpoint output",
        );
        trace.record_event(
            TraceEventType::CheckpointCreated,
            None,
            0,
            json!({
                "checkpoint_id": "checkpoint-123",
                "checkpoint_scope": "workspace",
                "checkpoint_restore_command": "boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
            }),
        );
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason = Some(TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "checkpoint required",
            None,
        ));
        trace.ended_at = Some(trace.started_at + 1);

        let mut final_state = serde_json::Map::new();
        final_state.insert("latest_checkpoint_id".to_string(), json!("checkpoint-123"));
        final_state.insert("latest_checkpoint_scope".to_string(), json!("workspace"));
        final_state.insert(
            "latest_checkpoint_restore_command".to_string(),
            json!("boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"),
        );
        let response = TaskRunResponse {
            task_id: "task-checkpoint".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "checkpoint required",
                None,
            ),
            final_context: TaskContext::new(
                "session-checkpoint",
                "/tmp/workspace",
                RunLimits::default(),
                final_state,
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-checkpoint.json".to_string(),
        };

        let run_text = render_run_trace("run", Some(&trace), &response, "/boundline-next");
        assert!(run_text.contains("checkpoint checkpoint-123 created (workspace)"), "{run_text}");
        assert!(run_text.contains("latest_checkpoint_id: checkpoint-123"), "{run_text}");
        assert!(run_text.contains("latest_checkpoint_scope: workspace"), "{run_text}");
        assert!(
            run_text.contains(
                "latest_checkpoint_restore_command: boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
            ),
            "{run_text}"
        );

        let session_text = render_session_status(&SessionStatusView {
            session_id: "session-checkpoint".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            latest_checkpoint_id: Some("checkpoint-123".to_string()),
            latest_checkpoint_scope: Some("workspace".to_string()),
            latest_checkpoint_restore_command: Some(
                "boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
                    .to_string(),
            ),
            explanation: "checkpoint available".to_string(),
            ..SessionStatusView::default()
        });
        assert!(session_text.contains("latest_checkpoint_id: checkpoint-123"), "{session_text}");
        assert!(session_text.contains("latest_checkpoint_scope: workspace"), "{session_text}");

        let trace_text = render_trace_summary(
            &TraceSummaryView {
                trace_ref: "/tmp/workspace/.boundline/traces/task-checkpoint.json".to_string(),
                goal: "Render checkpoint output".to_string(),
                latest_checkpoint_id: Some("checkpoint-123".to_string()),
                latest_checkpoint_scope: Some("workspace".to_string()),
                latest_checkpoint_restore_command: Some(
                    "boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
                        .to_string(),
                ),
                terminal_status: TaskStatus::Failed,
                terminal_reason: TerminalReason::new(
                    TerminalCondition::UnrecoverableError,
                    "checkpoint required",
                    None,
                ),
                ..TraceSummaryView::default()
            },
            "latest-workspace-trace",
            "/boundline-next",
        );
        assert!(trace_text.contains("latest_checkpoint_id: checkpoint-123"), "{trace_text}");
        assert!(trace_text.contains("latest_checkpoint_scope: workspace"), "{trace_text}");
    }

    #[test]
    fn output_helper_functions_cover_review_governance_and_execution_conditions() {
        assert_eq!(
            review_event_line(
                TraceEventType::ReviewTriggerIgnored,
                &json!({"review_trigger": "manual"}),
            ),
            Some("review_trigger_ignored: manual".to_string())
        );
        assert_eq!(
            review_event_line(
                TraceEventType::ReviewVoteResolved,
                &json!({"vote_resolution": {"decision": "accepted"}}),
            )
            .unwrap(),
            "review_vote: {\"decision\":\"accepted\"}"
        );
        assert_eq!(
            review_event_line(
                TraceEventType::ReviewAdjudicated,
                &json!({
                    "reviewer_id": "safety",
                    "finding": {"disposition": "approve", "summary": "No blockers"}
                }),
            ),
            Some("review_adjudication: reviewer safety approve: No blockers".to_string())
        );
        assert_eq!(
            review_event_line(
                TraceEventType::ReviewTerminalRecorded,
                &json!({"failure_reason": "timed out"}),
            ),
            Some("review_reason: timed out".to_string())
        );
        assert_eq!(
            reviewer_event_line(&json!({"reviewer_id": "safety", "failure_reason": "timed out"})),
            Some("reviewer safety failed: timed out".to_string())
        );

        assert_eq!(
            governance_event_line(
                TraceEventType::GovernanceDecisionRecorded,
                &json!({"blocked_reason": "needs approval"}),
            ),
            Some("governance_decision_blocked: needs approval".to_string())
        );
        assert_eq!(
            governance_event_line(
                TraceEventType::GovernanceAwaitingApproval,
                &json!({
                    "stage_key": "bug-fix:implement",
                    "approval_state": "requested",
                    "run_ref": "canon-run-1",
                    "packet_source_stage": "bug-fix:investigate",
                    "packet_binding_reason": "upstream_stage_context"
                }),
            ),
            Some(
                "governance_awaiting_approval: bug-fix:implement (requested) [canon-run-1] from bug-fix:investigate (upstream_stage_context)"
                    .to_string(),
            )
        );
        assert_eq!(
            governance_event_line(
                TraceEventType::GovernanceCompleted,
                &json!({"packet_ref": ".canon/runs/canon-run-1"}),
            ),
            Some(
                "governance_completed: governed packet ready [.canon/runs/canon-run-1]".to_string()
            )
        );
        assert_eq!(
            governance_event_line(TraceEventType::GovernanceBlocked, &json!({})),
            Some("governance_blocked: blocked".to_string())
        );
        assert_eq!(
            governance_event_line(TraceEventType::GovernancePacketRejected, &json!({})),
            Some("governance_packet_rejected: packet rejected".to_string())
        );

        let review_terminal = session_execution_condition_parts(&SessionStatusView {
            workflow_phase: Some("review".to_string()),
            latest_status: SessionStatus::Failed,
            ..SessionStatusView::default()
        });
        assert_eq!(review_terminal.0, "terminal");

        let govern_blocked = session_execution_condition_parts(&SessionStatusView {
            workflow_phase: Some("govern".to_string()),
            latest_status: SessionStatus::Running,
            ..SessionStatusView::default()
        });
        assert_eq!(govern_blocked.0, "blocked");

        let govern_waiting = session_execution_condition_parts(&SessionStatusView {
            workflow_phase: Some("govern".to_string()),
            latest_governance_state: Some("completed".to_string()),
            latest_status: SessionStatus::Running,
            ..SessionStatusView::default()
        });
        assert_eq!(govern_waiting.0, "waiting");

        let reasoning_blocked = session_execution_condition_parts(&SessionStatusView {
            latest_reasoning_profile: Some(crate::domain::reasoning::ProfileActivationRecord {
                activation_id: "reasoning-attempt-3".to_string(),
                stage_key: "bug-fix:verify".to_string(),
                profile_id: crate::domain::reasoning::ReasoningProfileId::IndependentPairReview,
                trigger: crate::domain::reasoning::ReasoningActivationTrigger::OperatorPolicy,
                activation_reason: "stronger challenge required".to_string(),
                status: crate::domain::reasoning::ReasoningActivationStatus::Blocked,
                participants: Vec::new(),
                budget: crate::domain::reasoning::ReasoningBudget {
                    max_participants: 2,
                    max_branches: 1,
                    max_debate_rounds: 0,
                    max_reflexion_revisions: 0,
                    max_calls: 2,
                    max_tokens: 8_000,
                    max_adjudication_steps: 1,
                },
                posture: None,
                independence: None,
                outcome: Some(crate::domain::reasoning::ReasoningOutcome {
                    outcome_kind: crate::domain::reasoning::ReasoningOutcomeKind::Blocked,
                    headline: "independent pair review blocked".to_string(),
                    disagreement_summary: Some("reviewers collapsed onto one route".to_string()),
                    next_action: Some("configure distinct reviewer routes".to_string()),
                    iterations: Vec::new(),
                }),
                confidence: None,
            }),
            ..SessionStatusView::default()
        });
        assert_eq!(reasoning_blocked.0, "blocked");
        assert!(reasoning_blocked.1.contains("configure distinct reviewer routes"));

        let flow_confirmation = session_execution_condition_parts(&SessionStatusView {
            execution_path: Some("native_goal_plan_pending_plan_confirmation".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(flow_confirmation.0, "blocked");

        let delegated_blocked = session_execution_condition_parts(&SessionStatusView {
            delegation: Some(crate::domain::session::DelegationStatusView {
                mode: crate::domain::session::DelegationContinuityMode::Stuck,
                packet_id: Some("packet-stuck".to_string()),
                packet_kind: Some(crate::domain::session::DelegationPacketKind::Handoff),
                packet_state: Some(crate::domain::session::DelegationPacketState::Stuck),
                target_owner: Some("operator".to_string()),
                headline: "stuck delegated continuity requires recovery".to_string(),
                evidence_summary: "the same blocked continuity reason repeated three times"
                    .to_string(),
            }),
            ..SessionStatusView::default()
        });
        assert_eq!(delegated_blocked.0, "blocked");
        assert!(delegated_blocked.1.contains("stuck delegated continuity"));

        let planned_step = session_execution_condition_parts(&SessionStatusView {
            latest_status: SessionStatus::Planned,
            current_step_id: Some("step-1".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(planned_step.0, "waiting");
        assert!(planned_step.1.contains("bounded task is ready"));

        let waiting_trace = trace_execution_condition_parts(&TraceSummaryView {
            governance_timeline: vec![
                "governance_awaiting_approval: bug-fix:implement".to_string(),
            ],
            terminal_status: TaskStatus::Running,
            terminal_reason: TerminalReason::new(
                TerminalCondition::NoCredibleNextStep,
                "still running",
                None,
            ),
            ..TraceSummaryView::default()
        });
        assert_eq!(waiting_trace.0, "waiting");

        let blocked_trace = trace_execution_condition_parts(&TraceSummaryView {
            governance_timeline: vec!["governance_packet_rejected: blocked".to_string()],
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            ..TraceSummaryView::default()
        });
        assert_eq!(blocked_trace.0, "blocked");

        let exhausted_trace = trace_execution_condition_parts(&TraceSummaryView {
            adaptive_evidence: vec!["adaptive_exhaustion: limits exhausted".to_string()],
            terminal_status: TaskStatus::Exhausted,
            terminal_reason: TerminalReason::new(
                TerminalCondition::RetryBudgetExhausted,
                "trace exhausted",
                None,
            ),
            ..TraceSummaryView::default()
        });
        assert_eq!(exhausted_trace.0, "terminal");
        assert_eq!(exhausted_trace.1, "limits exhausted");

        let reasoning_blocked_trace = trace_execution_condition_parts(&TraceSummaryView {
            reasoning_profile: Some(crate::domain::reasoning::ProfileActivationRecord {
                activation_id: "reasoning-attempt-4".to_string(),
                stage_key: "bug-fix:verify".to_string(),
                profile_id: crate::domain::reasoning::ReasoningProfileId::BoundedReflexion,
                trigger:
                    crate::domain::reasoning::ReasoningActivationTrigger::CanonRequiredChallenge,
                activation_reason: "Canon governance activated stronger challenge".to_string(),
                status: crate::domain::reasoning::ReasoningActivationStatus::Blocked,
                participants: Vec::new(),
                budget: crate::domain::reasoning::ReasoningBudget {
                    max_participants: 1,
                    max_branches: 1,
                    max_debate_rounds: 0,
                    max_reflexion_revisions: 2,
                    max_calls: 2,
                    max_tokens: 6_000,
                    max_adjudication_steps: 1,
                },
                posture: None,
                independence: None,
                outcome: Some(crate::domain::reasoning::ReasoningOutcome {
                    outcome_kind: crate::domain::reasoning::ReasoningOutcomeKind::Blocked,
                    headline: "bounded reflexion blocked".to_string(),
                    disagreement_summary: Some("shared runtime reduced independence".to_string()),
                    next_action: Some("escalate to blind review".to_string()),
                    iterations: Vec::new(),
                }),
                confidence: None,
            }),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            ..TraceSummaryView::default()
        });
        assert_eq!(reasoning_blocked_trace.0, "blocked");
        assert!(reasoning_blocked_trace.1.contains("escalate to blind review"));

        let failed_trace = trace_execution_condition_parts(&TraceSummaryView {
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed without adaptive exhaustion",
                None,
            ),
            ..TraceSummaryView::default()
        });
        assert_eq!(failed_trace.0, "terminal");
        assert_eq!(failed_trace.1, "trace failed without adaptive exhaustion");

        let waiting_run = render_run_execution_condition(&TaskRunResponse {
            task_id: "task-run".to_string(),
            terminal_status: TaskStatus::Running,
            terminal_reason: TerminalReason::new(
                TerminalCondition::NoCredibleNextStep,
                "waiting for approval",
                None,
            ),
            final_context: TaskContext::new(
                "session-run",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-run.json".to_string(),
        });
        assert!(waiting_run.contains("execution_condition: waiting - waiting for approval"));

        let running_run = render_run_execution_condition(&TaskRunResponse {
            task_id: "task-run".to_string(),
            terminal_status: TaskStatus::Running,
            terminal_reason: TerminalReason::new(
                TerminalCondition::NoCredibleNextStep,
                "bounded execution is in progress",
                None,
            ),
            final_context: TaskContext::new(
                "session-run",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-run.json".to_string(),
        });
        assert!(
            running_run.contains("execution_condition: running - bounded execution is in progress")
        );

        let planned_run = render_run_execution_condition(&TaskRunResponse {
            task_id: "task-planned".to_string(),
            terminal_status: TaskStatus::Planned,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "task is planned and ready",
                None,
            ),
            final_context: TaskContext::new(
                "session-planned",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.boundline/traces/task-planned.json".to_string(),
        });
        assert!(planned_run.contains("execution_condition: waiting"));
    }
}
