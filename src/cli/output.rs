//! Operator-facing text and JSON renderers for CLI commands.

use std::io::{self, IsTerminal};
use std::path::Path;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use serde::Serialize;
use serde_json::Value;

use crate::cli::diagnostics::{DiagnosticsReport, DiagnosticsStatus};
use crate::cli::{CliValidationError, CommandExitStatus, DeveloperCommand};
use crate::domain::cluster::{
    ClusterDeliveryStory, ClusterInspectReport, ClusterMemberState, ClusteredExecutionKind,
    WorkspaceParticipationKind,
};
use crate::domain::configuration::{
    ModelRoute, RoutingConfig, RoutingOverrides, resolve_effective_routing,
    resolve_effective_runtime_capabilities, resolve_effective_slot_effort_policies,
};
use crate::domain::context_intelligence::{
    AdvancedContextProjection, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
    RelationshipCredibilityState, RelationshipKind, RetrievalSourceKind,
    RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState, SemanticTraceRecord,
};
use crate::domain::follow_through::FollowThroughProjection;
use crate::domain::goal_plan::GoalPlanFlowState;
use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::reasoning::ProfileActivationRecord;
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::RoutingOutcome;
use crate::domain::session::{
    CompatibilityFollowUpView, ContinuityAuthority, RoutingMode, RoutingSource, SessionStatus,
    SessionStatusView, governance_packet_provenance_text,
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
const S7_CONFIDENCE_HIGH: &str = "high";
const S7_CONFIDENCE_LOW: &str = "low";
const S7_CONFIDENCE_MEDIUM: &str = "medium";
const S7_FALLBACK_READY: &str =
    "Using authoritative Boundline runtime and available Canon signals.";
const S7_FALLBACK_CANON_MISSING: &str =
    "Canon input not yet available; using Boundline runtime evidence only";
const S7_FALLBACK_CLARIFICATION_PREFIX: &str = "Clarification is still required for: ";
const S7_FALLBACK_CONTEXT_STALE_PREFIX: &str =
    "Context is stale; refresh before treating this answer as fully current: ";
const S7_MISSING_CANON_SOURCE: &str = "canon_input";
const S7_MISSING_CLARIFICATION_SOURCE: &str = "clarification_fields";
const S7_MISSING_CONTEXT_SOURCE: &str = "fresh_context";
const S7_NONE: &str = "none";
const S7_RUNTIME_SOURCE_AUTHORED_INPUT: &str = "authored_input";
const S7_RUNTIME_SOURCE_CONTEXT: &str = "context";
const S7_RUNTIME_SOURCE_DECISION_TIMELINE: &str = "decision_timeline";
const S7_RUNTIME_SOURCE_REASONING_PROFILE: &str = "reasoning_profile";
const S7_RUNTIME_SOURCE_REVIEW_TIMELINE: &str = "review_timeline";
const S7_RUNTIME_SOURCE_SESSION_STATE: &str = "session_state";
const S7_RUNTIME_SOURCE_TRACE_EVIDENCE: &str = "trace_evidence";
const S7_RUNTIME_SOURCE_TRACE_STEPS: &str = "trace_steps";
const S7_CANON_SOURCE_APPROVAL_PROVENANCE: &str = "approval_provenance";
const S7_CANON_SOURCE_GOVERNANCE_ACTION: &str = "governance_next_action";
const S7_CANON_SOURCE_GOVERNANCE_DECISION: &str = "governance_decision";
const S7_CANON_SOURCE_GOVERNANCE_PACKET: &str = "governance_packet";
const S7_CANON_SOURCE_GOVERNANCE_TIMELINE: &str = "governance_timeline";
const S7_LABEL_CONFIDENCE_LEVEL: &str = "confidence_level";
const S7_LABEL_EVIDENCE_SUMMARY: &str = "evidence_summary";
const S7_LABEL_FALLBACK_DISCLOSURE: &str = "fallback_disclosure";
const S7_LABEL_NEXT_BEST_ACTION: &str = "next_best_action";
const S7_LABEL_RISK_SUMMARY: &str = "risk_summary";
const S7_LABEL_SOURCE_ATTRIBUTION: &str = "source_attribution";
const S7_LABEL_WHY_SUMMARY: &str = "why_summary";
const S7_LABEL_ASSUMPTIONS_SUMMARY: &str = "assumptions_summary";
const S7_LABEL_ASSUMPTION_GROUP: &str = "assumption_group";
const S7_LABEL_HIDDEN_IMPACT_SUMMARY: &str = "hidden_impact_summary";
const S7_LABEL_HIDDEN_IMPACT_FALLBACK_DISCLOSURE: &str = "hidden_impact_fallback_disclosure";
const S7_LABEL_CHALLENGE_COUNCIL_REQUIRED: &str = "challenge_council_required";
const S7_LABEL_CHALLENGE_FAILURE_MODE: &str = "challenge_failure_mode";
const S7_LABEL_CHALLENGE_MISSING_EVIDENCE: &str = "challenge_missing_evidence";
const S7_LABEL_CHALLENGE_REQUIRED_REVIEW: &str = "challenge_required_review";
const S7_LABEL_CHALLENGE_STRONGEST_OBJECTION: &str = "challenge_strongest_objection";
const S7_LABEL_CHALLENGE_WEAKEST_ASSUMPTION: &str = "challenge_weakest_assumption";
const S7_LABEL_EXPLAIN_PLAN_GOVERNANCE: &str = "explain_plan_governance";
const S7_LABEL_EXPLAIN_PLAN_RECOVERY: &str = "explain_plan_recovery";
const S7_LABEL_EXPLAIN_PLAN_SUMMARY: &str = "explain_plan_summary";
const S7_LABEL_EXPLAIN_PLAN_VALIDATION: &str = "explain_plan_validation";
const S7_ASSUMPTION_CATEGORY_ARCHITECTURE: &str = "architecture";
const S7_ASSUMPTION_CATEGORY_DOMAIN: &str = "domain";
const S7_ASSUMPTION_CATEGORY_GOVERNANCE: &str = "governance";
const S7_ASSUMPTION_CATEGORY_IMPLEMENTATION: &str = "implementation";
const S7_ASSUMPTION_CATEGORY_VALIDATION: &str = "validation";
const S7_ASSUMPTION_RISK_HIGH: &str = "high";
const S7_ASSUMPTION_RISK_LOW: &str = "low";
const S7_ASSUMPTION_RISK_MEDIUM: &str = "medium";
const S7_ASSUMPTION_SOURCE_CANON: &str = "Canon";
const S7_ASSUMPTION_SOURCE_TRACE: &str = "trace";
const S7_ASSUMPTION_SOURCE_WORKSPACE: &str = "workspace";
const S7_ASSUMPTION_STATUS_EXPLICIT: &str = "explicit";
const S7_ASSUMPTION_STATUS_INFERRED: &str = "inferred";
const S7_ASSUMPTION_STATUS_MISSING: &str = "missing";
const S7_COUNCIL_REQUIRED_NO: &str = "no";
const S7_COUNCIL_REQUIRED_YES: &str = "yes";
const S7_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS: &str = "affected_domains";
const S7_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS: &str = "affected_systems";
const S7_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES: &str = "contract_exposures";
const S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE: &str = "missing_evidence";
const S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS: &str = "missing_tests";
const S7_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS: &str = "required_reviewers";
const S7_HIDDEN_IMPACT_LABEL_AFFECTED_DOMAINS: &str = "hidden_impact_affected_domains";
const S7_HIDDEN_IMPACT_LABEL_AFFECTED_SYSTEMS: &str = "hidden_impact_affected_systems";
const S7_HIDDEN_IMPACT_LABEL_CONTRACT_EXPOSURES: &str = "hidden_impact_contract_exposures";
const S7_HIDDEN_IMPACT_LABEL_MISSING_EVIDENCE: &str = "hidden_impact_missing_evidence";
const S7_HIDDEN_IMPACT_LABEL_MISSING_TESTS: &str = "hidden_impact_missing_tests";
const S7_HIDDEN_IMPACT_LABEL_REQUIRED_REVIEWERS: &str = "hidden_impact_required_reviewers";
const S7_REVIEW_RUNTIME_ONLY: &str = "bounded runtime review only";
const S7_WEAK_ASSUMPTION_NONE: &str = "none";
const S7_RISK_CANON_GAP: &str =
    "Canon confirmation is missing, so risk remains bounded by runtime-only evidence.";
const S7_RISK_NO_EXPLICIT_FAILURE: &str =
    "No explicit runtime failure evidence is currently reported.";
const S7_WHY_FALLBACK: &str =
    "Boundline has current runtime state but no richer explanation summary yet.";

#[derive(Debug, Clone, PartialEq, Eq)]
struct S7Projection {
    why_summary: String,
    risk_summary: String,
    evidence_summary: String,
    source_attribution: String,
    fallback_disclosure: String,
    confidence_level: &'static str,
    next_best_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct S7AssumptionEntry {
    category: &'static str,
    subject_ref: String,
    status: &'static str,
    source: &'static str,
    risk: &'static str,
    explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct S7HiddenImpactEntry {
    group: &'static str,
    label: &'static str,
    subject_ref: String,
    status: &'static str,
    severity: &'static str,
    follow_up: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct S7CognitiveProjection {
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

fn checkpoint_projection_from_state(
    state: &serde_json::Map<String, Value>,
) -> (Option<String>, Option<String>, Option<String>) {
    (
        state.get("latest_checkpoint_id").and_then(Value::as_str).map(str::to_string),
        state.get("latest_checkpoint_scope").and_then(Value::as_str).map(str::to_string),
        state.get("latest_checkpoint_restore_command").and_then(Value::as_str).map(str::to_string),
    )
}

/// Exit-code families used by host-facing command wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExitCode {
    Success,
    NonSuccess,
    InvalidInvocation,
    TraceReadFailure,
}

impl CommandExitCode {
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::NonSuccess => 1,
            Self::InvalidInvocation => 2,
            Self::TraceReadFailure => 3,
        }
    }

    pub const fn for_status(status: CommandExitStatus) -> Self {
        match status {
            CommandExitStatus::Succeeded => Self::Success,
            CommandExitStatus::NonSuccess => Self::NonSuccess,
            CommandExitStatus::InvalidInvocation => Self::InvalidInvocation,
            CommandExitStatus::TraceReadFailure => Self::TraceReadFailure,
        }
    }
}

/// Structured host payload used when commands need machine-readable output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HostCommandEnvelope {
    pub command_name: String,
    pub exit_status: String,
    pub rendered_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<SessionStatusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_summary: Option<TraceSummaryView>,
}

fn command_exit_status_label(status: CommandExitStatus) -> &'static str {
    match status {
        CommandExitStatus::Succeeded => "succeeded",
        CommandExitStatus::NonSuccess => "non_success",
        CommandExitStatus::InvalidInvocation => "invalid_invocation",
        CommandExitStatus::TraceReadFailure => "trace_read_failure",
    }
}

/// Renders a command result as structured JSON for host integrations.
pub fn render_host_command_json(
    command_name: &str,
    exit_status: CommandExitStatus,
    rendered_output: &str,
    trace_location: Option<&str>,
    session_status: Option<&SessionStatusView>,
    trace_summary: Option<&TraceSummaryView>,
) -> String {
    match serde_json::to_string_pretty(&HostCommandEnvelope {
        command_name: command_name.to_string(),
        exit_status: command_exit_status_label(exit_status).to_string(),
        rendered_output: rendered_output.to_string(),
        trace_location: trace_location.map(str::to_string),
        session_status: session_status.cloned(),
        trace_summary: trace_summary.cloned(),
    }) {
        Ok(rendered) => rendered,
        Err(error) => serde_json::json!({
            "command_name": command_name,
            "exit_status": command_exit_status_label(exit_status),
            "rendered_output": rendered_output,
            "trace_location": trace_location,
            "session_status": session_status,
            "trace_summary": trace_summary,
            "serialization_error": error.to_string(),
        })
        .to_string(),
    }
}

/// Returns the default not-implemented message for a developer command.
pub fn unimplemented_message(command: &DeveloperCommand) -> String {
    format!("`{}` is not implemented yet", command_name(command))
}

/// Returns the stable CLI command name used in output and host envelopes.
pub fn command_name(command: &DeveloperCommand) -> &'static str {
    match command {
        DeveloperCommand::Doctor { .. } => "doctor",
        DeveloperCommand::Checkpoint { .. } => "checkpoint",
        DeveloperCommand::Start { .. } => "start",
        DeveloperCommand::Capture { .. } => "capture",
        DeveloperCommand::Flow { .. } => "flow",
        DeveloperCommand::Plan { .. } => "plan",
        DeveloperCommand::Step { .. } => "step",
        DeveloperCommand::Run { .. } => "run",
        DeveloperCommand::Workflow { .. } => "workflow",
        DeveloperCommand::Inspect { .. } => "inspect",
        DeveloperCommand::Status { .. } => "status",
        DeveloperCommand::Next { .. } => "next",
        DeveloperCommand::Continue { .. } => "continue",
        DeveloperCommand::Govern { .. } => "govern",
        DeveloperCommand::Assistant { .. } => "assistant",
        DeveloperCommand::Init { .. } => "init",
        DeveloperCommand::Config { .. } => "config",
        DeveloperCommand::Cluster { .. } => "cluster",
    }
}

/// Renders the result of initializing a workspace cluster.
pub fn render_cluster_init(cluster_id: &str, cluster_path: &str, members: &[String]) -> String {
    let mut lines = vec![
        "cluster: initialized".to_string(),
        format!("cluster_id: {cluster_id}"),
        format!("cluster_file: {cluster_path}"),
        "members:".to_string(),
    ];
    for member in members {
        lines.push(format!("- {member}"));
    }
    lines.join("\n")
}

/// Renders the current status projection for a workspace cluster.
pub fn render_cluster_status(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: status".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let mut line =
            format!("- {} [{}]", member.workspace_ref, cluster_member_state_text(member.state));
        if let Some(status) = member.latest_status {
            line.push_str(&format!(" status={}", session_status_text(status)));
        }
        line.push_str(&format!(" {}", member.headline));
        lines.push(line);
    }

    lines.join("\n")
}

/// Renders the current inspect projection for a workspace cluster.
pub fn render_cluster_inspect(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: inspect".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let trace_text = member.latest_trace_ref.as_deref().unwrap_or("<missing>");
        lines.push(format!(
            "- {} [{}] trace={} {}",
            member.workspace_ref,
            cluster_member_state_text(member.state),
            trace_text,
            member.headline
        ));
    }

    lines.join("\n")
}

/// Returns the user-facing validation error message for CLI argument failures.
pub fn validation_error_message(error: &CliValidationError) -> String {
    error.to_string()
}

/// Human-readable route summary shared by status and inspect surfaces.
pub fn render_route_outcome(outcome: &RoutingOutcome) -> String {
    format!("routing: {} ({}) - {}", outcome.mode.as_str(), outcome.source.as_str(), outcome.reason)
}

/// Human-readable flow-state summary shared by status and inspect surfaces.
pub fn render_goal_plan_flow_state(flow_state: &GoalPlanFlowState) -> String {
    format!("flow_state: {}", flow_state.summary_text())
}

fn push_context_projection_lines(
    lines: &mut Vec<String>,
    context_summary: Option<&str>,
    context_credibility: Option<&str>,
    context_primary_inputs: &[String],
    context_provenance: &[String],
    context_staleness_reason: Option<&str>,
) {
    if let Some(context_summary) = context_summary {
        lines.push(format!("context_summary: {context_summary}"));
    }

    if let Some(context_credibility) = context_credibility {
        lines.push(format!("context_credibility: {context_credibility}"));
    }

    if !context_primary_inputs.is_empty() {
        lines.push(format!("context_primary_inputs: {}", context_primary_inputs.join(", ")));
    }

    if !context_provenance.is_empty() {
        lines.push(format!("context_provenance: {}", context_provenance.join(" | ")));
    }

    if let Some(context_staleness_reason) = context_staleness_reason {
        lines.push(format!("context_staleness_reason: {context_staleness_reason}"));
    }
}

// Render the compact advanced-context projection so `status` and `inspect`
// can explain retrieval state without hiding the runtime reasoning.
fn push_advanced_context_lines(
    lines: &mut Vec<String>,
    advanced_context: Option<&AdvancedContextProjection>,
) {
    let Some(advanced_context) = advanced_context else {
        return;
    };

    lines.push(format!("retrieval_mode: {}", advanced_context.retrieval_mode.as_str()));
    lines.push(format!("retrieval_state: {}", advanced_context.retrieval_state.as_str()));
    lines.push(format!("retrieval_authority_order: {}", advanced_context.authority_order_text()));
    lines.push(format!(
        "retrieval_index_state: {}",
        advanced_context.retrieval_index_state.as_str()
    ));
    lines.push(format!(
        "semantic_policy_state: {}",
        advanced_context.semantic_policy_state.as_str()
    ));
    lines.push(format!(
        "semantic_capability_state: {}",
        advanced_context.semantic_capability_state.as_str()
    ));
    lines.push(format!("hybrid_outcome: {}", advanced_context.hybrid_outcome.as_str()));
    if let Some(terminal_reason) = advanced_context.terminal_reason.as_deref() {
        lines.push(format!("retrieval_terminal_reason: {terminal_reason}"));
    }
    lines.push(format!("selected_evidence_count: {}", advanced_context.selected_evidence_count()));
    lines.push(format!("semantic_selected_count: {}", advanced_context.semantic_selected_count()));
    lines.push(format!("semantic_rejected_count: {}", advanced_context.semantic_rejected_count()));
    lines.push(format!("impact_finding_count: {}", advanced_context.impact_finding_count()));

    for candidate in &advanced_context.selected_evidence {
        lines.push(format_candidate_line("selected_evidence", candidate));
    }

    for candidate in &advanced_context.rejected_candidates {
        lines.push(format_candidate_line("rejected_candidate", candidate));
    }

    for record in &advanced_context.semantic_trace_records {
        lines.push(format_semantic_trace_line(record));
    }

    for relationship in &advanced_context.relationships {
        lines.push(format!(
            "relationship: {} [{}] {}",
            relationship.subject_ref,
            relationship.relationship_kind.as_str(),
            relationship.explanation
        ));
    }

    for finding in &advanced_context.impact_findings {
        lines.push(format!(
            "impact_finding: {} [{}] {}",
            finding.subject_ref,
            finding.finding_kind.as_str(),
            finding.recommended_follow_up
        ));
    }
}

fn format_candidate_line(prefix: &str, candidate: &RetrievedEvidenceCandidate) -> String {
    let mut line = format!(
        "{prefix}: {} [{}] origin={}{} {}",
        candidate.source_ref,
        candidate.source_kind.as_str(),
        candidate.match_origin.as_str(),
        candidate_score_suffix(candidate),
        candidate.selection_reason
    );
    if let (Some(contract_line), Some(provenance_ref)) = (
        candidate.canon_semantic_contract_line.as_deref(),
        candidate.canon_semantic_provenance_ref.as_deref(),
    ) {
        line.push_str(&format!(
            " canon_contract={} canon_provenance={}",
            contract_line, provenance_ref
        ));
    }
    if candidate.compatibility_state.as_str() != "compatible" {
        line.push_str(&format!(" compatibility={}", candidate.compatibility_state.as_str()));
    }
    line
}

fn format_semantic_trace_line(record: &SemanticTraceRecord) -> String {
    let mut line = format!("semantic_trace: {}", record.event_kind.as_str());
    if let Some(candidate_ref) = record.candidate_ref.as_deref() {
        line.push_str(&format!(" ref={candidate_ref}"));
    }
    if let Some(match_origin) = record.match_origin {
        line.push_str(&format!(" origin={}", match_origin.as_str()));
    }
    if let Some(compatibility_state) = record.compatibility_state {
        line.push_str(&format!(" compatibility={}", compatibility_state.as_str()));
    }
    if let Some(semantic_score) = record.semantic_score {
        line.push_str(&format!(" semantic_score={:.3}", semantic_score.as_raw()));
    }
    if let Some(artifact_class) = record.canon_artifact_class.as_deref() {
        line.push_str(&format!(" canon_artifact_class={artifact_class}"));
    }
    if let Some(contract_line) = record.canon_semantic_contract_line.as_deref() {
        line.push_str(&format!(" canon_contract={contract_line}"));
    }
    if let Some(boundary) = record.canon_semantic_provenance_boundary {
        line.push_str(&format!(" canon_boundary={}", boundary.as_str()));
    }
    if let Some(provenance_ref) = record.canon_semantic_provenance_ref.as_deref() {
        line.push_str(&format!(" canon_provenance={provenance_ref}"));
    }
    line.push(' ');
    line.push_str(&record.reason);
    line
}

fn candidate_score_suffix(candidate: &RetrievedEvidenceCandidate) -> String {
    let mut suffix = String::new();
    if let Some(lexical_score) = candidate.lexical_score {
        suffix.push_str(&format!(" lexical_score={:.3}", lexical_score.as_raw()));
    }
    if let Some(semantic_score) = candidate.semantic_score {
        suffix.push_str(&format!(" semantic_score={:.3}", semantic_score.as_raw()));
    }
    suffix
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputPresentation {
    Plain,
    Rich,
}

fn stdout_presentation() -> OutputPresentation {
    if io::stdout().is_terminal() { OutputPresentation::Rich } else { OutputPresentation::Plain }
}

fn push_output_section(
    lines: &mut Vec<String>,
    presentation: OutputPresentation,
    title: &str,
    section_lines: Vec<String>,
) {
    if section_lines.is_empty() {
        return;
    }

    if matches!(presentation, OutputPresentation::Rich) && !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push(format!("{title}:"));
    lines.extend(section_lines);
}

fn s7_source_bucket_text(labels: &[&str]) -> String {
    if labels.is_empty() { S7_NONE.to_string() } else { labels.join(", ") }
}

fn s7_confidence_level(
    has_failure_signal: bool,
    canon_missing: bool,
    context_stale: bool,
    clarification_required: bool,
) -> &'static str {
    if has_failure_signal || clarification_required {
        S7_CONFIDENCE_LOW
    } else if canon_missing || context_stale {
        S7_CONFIDENCE_MEDIUM
    } else {
        S7_CONFIDENCE_HIGH
    }
}

fn s7_fallback_disclosure(
    canon_missing: bool,
    context_staleness_reason: Option<&str>,
    clarification_missing_fields: &[String],
) -> String {
    if canon_missing {
        S7_FALLBACK_CANON_MISSING.to_string()
    } else if let Some(reason) = context_staleness_reason {
        format!("{S7_FALLBACK_CONTEXT_STALE_PREFIX}{reason}")
    } else if !clarification_missing_fields.is_empty() {
        format!("{S7_FALLBACK_CLARIFICATION_PREFIX}{}", clarification_missing_fields.join(", "))
    } else {
        S7_FALLBACK_READY.to_string()
    }
}

fn s7_projection_lines(projection: &S7Projection) -> Vec<String> {
    vec![
        format!("{S7_LABEL_WHY_SUMMARY}: {}", projection.why_summary),
        format!("{S7_LABEL_RISK_SUMMARY}: {}", projection.risk_summary),
        format!("{S7_LABEL_EVIDENCE_SUMMARY}: {}", projection.evidence_summary),
        format!("{S7_LABEL_SOURCE_ATTRIBUTION}: {}", projection.source_attribution),
        format!("{S7_LABEL_FALLBACK_DISCLOSURE}: {}", projection.fallback_disclosure),
        format!("{S7_LABEL_CONFIDENCE_LEVEL}: {}", projection.confidence_level),
        format!("{S7_LABEL_NEXT_BEST_ACTION}: {}", projection.next_best_action),
    ]
}

fn s7_cognitive_projection_lines(projection: &S7CognitiveProjection) -> Vec<String> {
    let mut lines =
        vec![format!("{S7_LABEL_ASSUMPTIONS_SUMMARY}: {}", projection.assumptions_summary)];
    lines.extend(projection.assumption_groups.iter().cloned());
    lines.push(format!("{S7_LABEL_HIDDEN_IMPACT_SUMMARY}: {}", projection.hidden_impact_summary));
    lines.extend(projection.hidden_impact_lines.iter().cloned());
    if let Some(fallback_disclosure) = projection.hidden_impact_fallback_disclosure.as_deref() {
        lines.push(format!("{S7_LABEL_HIDDEN_IMPACT_FALLBACK_DISCLOSURE}: {fallback_disclosure}"));
    }
    lines.push(format!(
        "{S7_LABEL_CHALLENGE_STRONGEST_OBJECTION}: {}",
        projection.challenge_strongest_objection
    ));
    lines.push(format!(
        "{S7_LABEL_CHALLENGE_WEAKEST_ASSUMPTION}: {}",
        projection.challenge_weakest_assumption
    ));
    lines.push(format!(
        "{S7_LABEL_CHALLENGE_MISSING_EVIDENCE}: {}",
        projection.challenge_missing_evidence
    ));
    lines.push(format!("{S7_LABEL_CHALLENGE_FAILURE_MODE}: {}", projection.challenge_failure_mode));
    lines.push(format!(
        "{S7_LABEL_CHALLENGE_REQUIRED_REVIEW}: {}",
        projection.challenge_required_review
    ));
    lines.push(format!(
        "{S7_LABEL_CHALLENGE_COUNCIL_REQUIRED}: {}",
        projection.challenge_council_required
    ));
    lines.push(format!("{S7_LABEL_EXPLAIN_PLAN_SUMMARY}: {}", projection.explain_plan_summary));
    lines.push(format!(
        "{S7_LABEL_EXPLAIN_PLAN_VALIDATION}: {}",
        projection.explain_plan_validation
    ));
    lines.push(format!(
        "{S7_LABEL_EXPLAIN_PLAN_GOVERNANCE}: {}",
        projection.explain_plan_governance
    ));
    lines.push(format!("{S7_LABEL_EXPLAIN_PLAN_RECOVERY}: {}", projection.explain_plan_recovery));
    lines
}

fn reasoning_projection_why_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile
            .outcome
            .as_ref()
            .map(|outcome| outcome.headline.clone())
            .or_else(|| Some(profile.activation_reason.clone()))
    })
}

fn reasoning_projection_risk_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile
            .confidence
            .as_ref()
            .map(|confidence| confidence.summary.clone())
            .or_else(|| {
                profile.outcome.as_ref().and_then(|outcome| outcome.disagreement_summary.clone())
            })
            .or_else(|| profile.outcome.as_ref().map(|outcome| outcome.headline.clone()))
    })
}

fn reasoning_projection_confidence_level(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<&'static str> {
    reasoning_profile.and_then(|profile| {
        profile.confidence.as_ref().map(|confidence| confidence.confidence_level.as_str())
    })
}

fn reasoning_projection_next_action(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile.outcome.as_ref().and_then(|outcome| outcome.next_action.clone())
    })
}

fn reasoning_projection_governance_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.map(|profile| {
        let mut parts = vec![
            format!("reasoning_profile={}", profile.profile_id),
            format!("status={}", profile.status.as_str()),
        ];
        if let Some(confidence) = &profile.confidence {
            parts.push(format!("confidence={}", confidence.confidence_level.as_str()));
            parts.push(format!("admission_effect={}", confidence.admission_effect.as_str()));
        }
        if let Some(posture) = &profile.posture {
            parts.push(format!("posture_contract={}", posture.contract_line));
        }
        parts.join("; ")
    })
}

fn s7_projection_for_trace_summary(summary: &TraceSummaryView, next_command: &str) -> S7Projection {
    let mut runtime_sources = Vec::new();
    if summary.authored_input_summary.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_AUTHORED_INPUT);
    }
    if summary.context_summary.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_CONTEXT);
    }
    if !summary.executed_steps.is_empty() {
        runtime_sources.push(S7_RUNTIME_SOURCE_TRACE_STEPS);
    }
    if !summary.decision_timeline.is_empty() || !summary.failure_evidence.is_empty() {
        runtime_sources.push(S7_RUNTIME_SOURCE_TRACE_EVIDENCE);
    }
    if !summary.review_timeline.is_empty() {
        runtime_sources.push(S7_RUNTIME_SOURCE_REVIEW_TIMELINE);
    }
    if summary.reasoning_profile.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_REASONING_PROFILE);
    }

    let mut canon_sources = Vec::new();
    if !summary.governance_timeline.is_empty() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_TIMELINE);
    }
    if summary.governance_approval_provenance.is_some() {
        canon_sources.push(S7_CANON_SOURCE_APPROVAL_PROVENANCE);
    }
    if summary.governance_reason.is_some() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_DECISION);
    }
    if summary.governance_next_action.is_some() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_ACTION);
    }

    let mut missing_sources = Vec::new();
    if canon_sources.is_empty() {
        missing_sources.push(S7_MISSING_CANON_SOURCE);
    }
    if summary.context_staleness_reason.is_some() {
        missing_sources.push(S7_MISSING_CONTEXT_SOURCE);
    }
    if !summary.clarification_missing_fields.is_empty() {
        missing_sources.push(S7_MISSING_CLARIFICATION_SOURCE);
    }

    let why_summary = reasoning_projection_why_summary(summary.reasoning_profile.as_ref())
        .or_else(|| summary.goal_plan_summary.clone())
        .or_else(|| summary.negotiation_goal_summary.clone())
        .or_else(|| {
            summary.executed_steps.last().map(|step| {
                format!("latest bounded step {} reports {}", step.step_id, step.headline)
            })
        })
        .unwrap_or_else(|| {
            if summary.terminal_reason.message.trim().is_empty() {
                S7_WHY_FALLBACK.to_string()
            } else {
                summary.terminal_reason.message.clone()
            }
        });

    let risk_summary = if let Some(reasoning_risk) =
        reasoning_projection_risk_summary(summary.reasoning_profile.as_ref())
    {
        reasoning_risk
    } else if !summary.failure_evidence.is_empty() {
        summary.failure_evidence[0].clone()
    } else if let Some(reason) = summary.context_staleness_reason.as_ref() {
        format!("stale context reduces confidence: {reason}")
    } else if canon_sources.is_empty() {
        S7_RISK_CANON_GAP.to_string()
    } else if summary.terminal_reason.message.trim().is_empty() {
        S7_RISK_NO_EXPLICIT_FAILURE.to_string()
    } else {
        summary.terminal_reason.message.clone()
    };

    let evidence_summary = format!(
        "runtime({}): {}; canon({}): {}; missing({}): {}",
        runtime_sources.len(),
        s7_source_bucket_text(&runtime_sources),
        canon_sources.len(),
        s7_source_bucket_text(&canon_sources),
        missing_sources.len(),
        s7_source_bucket_text(&missing_sources)
    );

    let source_attribution = format!(
        "runtime={}; canon={}; missing={}",
        s7_source_bucket_text(&runtime_sources),
        s7_source_bucket_text(&canon_sources),
        s7_source_bucket_text(&missing_sources)
    );

    let fallback_disclosure = s7_fallback_disclosure(
        canon_sources.is_empty(),
        summary.context_staleness_reason.as_deref(),
        &summary.clarification_missing_fields,
    );

    let confidence_level = reasoning_projection_confidence_level(
        summary.reasoning_profile.as_ref(),
    )
    .unwrap_or_else(|| {
        s7_confidence_level(
            !summary.failure_evidence.is_empty(),
            canon_sources.is_empty(),
            summary.context_staleness_reason.is_some(),
            !summary.clarification_missing_fields.is_empty(),
        )
    });

    let next_best_action = reasoning_projection_next_action(summary.reasoning_profile.as_ref())
        .or_else(|| summary.governance_next_action.clone())
        .unwrap_or_else(|| next_command.to_string());

    S7Projection {
        why_summary,
        risk_summary,
        evidence_summary,
        source_attribution,
        fallback_disclosure,
        confidence_level,
        next_best_action,
    }
}

fn s7_projection_for_session_status(view: &SessionStatusView) -> S7Projection {
    let mut runtime_sources = vec![S7_RUNTIME_SOURCE_SESSION_STATE];
    if view.authored_input_summary.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_AUTHORED_INPUT);
    }
    if view.context_summary.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_CONTEXT);
    }
    if view.latest_selection_reason.is_some() || view.latest_validation_status.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_DECISION_TIMELINE);
    }
    if view.latest_review_headline.is_some() || view.latest_review_outcome.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_REVIEW_TIMELINE);
    }
    if view.latest_reasoning_profile.is_some() {
        runtime_sources.push(S7_RUNTIME_SOURCE_REASONING_PROFILE);
    }

    let mut canon_sources = Vec::new();
    if view.latest_governance_packet_ref.is_some() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_PACKET);
    }
    if view.latest_governance_approval_provenance.is_some() {
        canon_sources.push(S7_CANON_SOURCE_APPROVAL_PROVENANCE);
    }
    if view.latest_governance_decision.is_some() || view.latest_governance_reason.is_some() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_DECISION);
    }
    if view.governance_next_action.is_some() {
        canon_sources.push(S7_CANON_SOURCE_GOVERNANCE_ACTION);
    }

    let clarification_missing_fields =
        view.clarification_missing_fields.clone().unwrap_or_default();

    let mut missing_sources = Vec::new();
    if canon_sources.is_empty() {
        missing_sources.push(S7_MISSING_CANON_SOURCE);
    }
    if view.context_staleness_reason.is_some() {
        missing_sources.push(S7_MISSING_CONTEXT_SOURCE);
    }
    if !clarification_missing_fields.is_empty() {
        missing_sources.push(S7_MISSING_CLARIFICATION_SOURCE);
    }

    let why_summary = reasoning_projection_why_summary(view.latest_reasoning_profile.as_ref())
        .or_else(|| view.planning_rationale.clone())
        .or_else(|| view.latest_selection_reason.clone())
        .or_else(|| view.goal.clone())
        .unwrap_or_else(|| view.explanation.clone());

    let risk_summary = if let Some(reasoning_risk) =
        reasoning_projection_risk_summary(view.latest_reasoning_profile.as_ref())
    {
        reasoning_risk
    } else if let Some(reason) = view.latest_exhaustion_reason.as_ref() {
        reason.clone()
    } else if let Some(reason) = view.latest_governance_blocked_reason.as_ref() {
        reason.clone()
    } else if let Some(reason) = view.context_staleness_reason.as_ref() {
        format!("stale context reduces confidence: {reason}")
    } else if canon_sources.is_empty() {
        S7_RISK_CANON_GAP.to_string()
    } else if let Some(status) = view.latest_validation_status.as_ref() {
        status.clone()
    } else {
        S7_RISK_NO_EXPLICIT_FAILURE.to_string()
    };

    let evidence_summary = format!(
        "runtime({}): {}; canon({}): {}; missing({}): {}",
        runtime_sources.len(),
        s7_source_bucket_text(&runtime_sources),
        canon_sources.len(),
        s7_source_bucket_text(&canon_sources),
        missing_sources.len(),
        s7_source_bucket_text(&missing_sources)
    );

    let source_attribution = format!(
        "runtime={}; canon={}; missing={}",
        s7_source_bucket_text(&runtime_sources),
        s7_source_bucket_text(&canon_sources),
        s7_source_bucket_text(&missing_sources)
    );

    let fallback_disclosure = s7_fallback_disclosure(
        canon_sources.is_empty(),
        view.context_staleness_reason.as_deref(),
        &clarification_missing_fields,
    );

    let confidence_level =
        reasoning_projection_confidence_level(view.latest_reasoning_profile.as_ref())
            .unwrap_or_else(|| {
                s7_confidence_level(
                    view.latest_exhaustion_reason.is_some()
                        || view.latest_governance_blocked_reason.is_some(),
                    canon_sources.is_empty(),
                    view.context_staleness_reason.is_some(),
                    !clarification_missing_fields.is_empty(),
                )
            });

    let next_best_action = reasoning_projection_next_action(view.latest_reasoning_profile.as_ref())
        .or_else(|| view.governance_next_action.clone())
        .or_else(|| view.next_command.clone())
        .or_else(|| view.workflow_next_action.clone())
        .unwrap_or_else(|| view.explanation.clone());

    S7Projection {
        why_summary,
        risk_summary,
        evidence_summary,
        source_attribution,
        fallback_disclosure,
        confidence_level,
        next_best_action,
    }
}

fn s7_assumption_entries(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Vec<S7AssumptionEntry> {
    let Some(advanced_context) = advanced_context else {
        return Vec::new();
    };

    advanced_context
        .relationships
        .iter()
        .map(|relationship| S7AssumptionEntry {
            category: s7_assumption_category(relationship.relationship_kind),
            subject_ref: relationship.subject_ref.clone(),
            status: s7_assumption_status(relationship.relationship_kind),
            source: s7_assumption_source(
                advanced_context,
                relationship.supporting_candidate_ids.as_slice(),
            ),
            risk: s7_assumption_risk(
                relationship.relationship_kind,
                relationship.credibility_state,
            ),
            explanation: relationship.explanation.clone(),
        })
        .collect()
}

fn s7_hidden_impact_entries(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Vec<S7HiddenImpactEntry> {
    let Some(advanced_context) = advanced_context else {
        return Vec::new();
    };

    advanced_context
        .impact_findings
        .iter()
        .map(|finding| S7HiddenImpactEntry {
            group: s7_hidden_impact_group(finding.finding_kind),
            label: s7_hidden_impact_label(finding.finding_kind),
            subject_ref: finding.subject_ref.clone(),
            status: s7_hidden_impact_status(finding.status),
            severity: s7_hidden_impact_severity(finding.severity),
            follow_up: finding.recommended_follow_up.clone(),
        })
        .collect()
}

fn s7_assumption_category(kind: RelationshipKind) -> &'static str {
    match kind {
        RelationshipKind::AffectsSystem | RelationshipKind::ExposesContract => {
            S7_ASSUMPTION_CATEGORY_ARCHITECTURE
        }
        RelationshipKind::AffectsDomain => S7_ASSUMPTION_CATEGORY_DOMAIN,
        RelationshipKind::SuggestsReviewer => S7_ASSUMPTION_CATEGORY_GOVERNANCE,
        RelationshipKind::SupportsRisk => S7_ASSUMPTION_CATEGORY_IMPLEMENTATION,
        RelationshipKind::ExercisesTest | RelationshipKind::RequiresEvidence => {
            S7_ASSUMPTION_CATEGORY_VALIDATION
        }
    }
}

fn s7_assumption_status(kind: RelationshipKind) -> &'static str {
    match kind {
        RelationshipKind::ExercisesTest | RelationshipKind::ExposesContract => {
            S7_ASSUMPTION_STATUS_EXPLICIT
        }
        RelationshipKind::RequiresEvidence => S7_ASSUMPTION_STATUS_MISSING,
        RelationshipKind::AffectsSystem
        | RelationshipKind::AffectsDomain
        | RelationshipKind::SuggestsReviewer
        | RelationshipKind::SupportsRisk => S7_ASSUMPTION_STATUS_INFERRED,
    }
}

fn s7_assumption_source(
    advanced_context: &AdvancedContextProjection,
    supporting_candidate_ids: &[String],
) -> &'static str {
    for candidate_id in supporting_candidate_ids {
        if let Some(source_kind) = advanced_context
            .selected_evidence
            .iter()
            .chain(advanced_context.rejected_candidates.iter())
            .find(|candidate| candidate.candidate_id == *candidate_id)
            .map(|candidate| candidate.source_kind)
        {
            return match source_kind {
                RetrievalSourceKind::CanonArtifact => S7_ASSUMPTION_SOURCE_CANON,
                RetrievalSourceKind::Trace
                | RetrievalSourceKind::ReviewFinding
                | RetrievalSourceKind::VerificationEvidence => S7_ASSUMPTION_SOURCE_TRACE,
                RetrievalSourceKind::WorkspaceFile | RetrievalSourceKind::ProjectMemory => {
                    S7_ASSUMPTION_SOURCE_WORKSPACE
                }
            };
        }
    }

    S7_ASSUMPTION_SOURCE_TRACE
}

fn s7_assumption_risk(
    kind: RelationshipKind,
    credibility_state: RelationshipCredibilityState,
) -> &'static str {
    if matches!(kind, RelationshipKind::RequiresEvidence) {
        return S7_ASSUMPTION_RISK_HIGH;
    }

    match credibility_state {
        RelationshipCredibilityState::Credible => S7_ASSUMPTION_RISK_LOW,
        RelationshipCredibilityState::Tentative => S7_ASSUMPTION_RISK_MEDIUM,
        RelationshipCredibilityState::Insufficient => S7_ASSUMPTION_RISK_HIGH,
    }
}

fn s7_assumption_summary(entries: &[S7AssumptionEntry]) -> String {
    s7_group_summary(
        entries.iter().map(|entry| entry.category),
        &[
            S7_ASSUMPTION_CATEGORY_DOMAIN,
            S7_ASSUMPTION_CATEGORY_ARCHITECTURE,
            S7_ASSUMPTION_CATEGORY_IMPLEMENTATION,
            S7_ASSUMPTION_CATEGORY_VALIDATION,
            S7_ASSUMPTION_CATEGORY_GOVERNANCE,
        ],
    )
}

fn s7_assumption_group_lines(entries: &[S7AssumptionEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{S7_LABEL_ASSUMPTION_GROUP}: {} -> {} [{}] source={} risk={} {}",
                entry.category,
                entry.subject_ref,
                entry.status,
                entry.source,
                entry.risk,
                entry.explanation
            )
        })
        .collect()
}

fn s7_hidden_impact_group(kind: ImpactFindingKind) -> &'static str {
    match kind {
        ImpactFindingKind::AffectedSystem => S7_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS,
        ImpactFindingKind::AffectedDomain => S7_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS,
        ImpactFindingKind::MissingTest => S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS,
        ImpactFindingKind::ContractExposure => S7_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES,
        ImpactFindingKind::ReviewerGap => S7_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS,
        ImpactFindingKind::EvidenceGap => S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE,
    }
}

fn s7_hidden_impact_label(kind: ImpactFindingKind) -> &'static str {
    match kind {
        ImpactFindingKind::AffectedSystem => S7_HIDDEN_IMPACT_LABEL_AFFECTED_SYSTEMS,
        ImpactFindingKind::AffectedDomain => S7_HIDDEN_IMPACT_LABEL_AFFECTED_DOMAINS,
        ImpactFindingKind::MissingTest => S7_HIDDEN_IMPACT_LABEL_MISSING_TESTS,
        ImpactFindingKind::ContractExposure => S7_HIDDEN_IMPACT_LABEL_CONTRACT_EXPOSURES,
        ImpactFindingKind::ReviewerGap => S7_HIDDEN_IMPACT_LABEL_REQUIRED_REVIEWERS,
        ImpactFindingKind::EvidenceGap => S7_HIDDEN_IMPACT_LABEL_MISSING_EVIDENCE,
    }
}

fn s7_hidden_impact_status(status: ImpactFindingStatus) -> &'static str {
    match status {
        ImpactFindingStatus::Open => "open",
        ImpactFindingStatus::Acknowledged => "acknowledged",
        ImpactFindingStatus::Resolved => "resolved",
        ImpactFindingStatus::Invalidated => "invalidated",
    }
}

fn s7_hidden_impact_severity(severity: ImpactFindingSeverity) -> &'static str {
    match severity {
        ImpactFindingSeverity::Low => S7_ASSUMPTION_RISK_LOW,
        ImpactFindingSeverity::Medium => S7_ASSUMPTION_RISK_MEDIUM,
        ImpactFindingSeverity::High => S7_ASSUMPTION_RISK_HIGH,
    }
}

fn s7_hidden_impact_summary(entries: &[S7HiddenImpactEntry]) -> String {
    s7_group_summary(
        entries.iter().map(|entry| entry.group),
        &[
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS,
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS,
            S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS,
            S7_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES,
            S7_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS,
            S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE,
        ],
    )
}

fn s7_hidden_impact_lines(entries: &[S7HiddenImpactEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{}: {} [{}/{}] {}",
                entry.label, entry.subject_ref, entry.status, entry.severity, entry.follow_up
            )
        })
        .collect()
}

fn s7_group_summary<'a>(entries: impl Iterator<Item = &'a str>, ordered_groups: &[&str]) -> String {
    let collected = entries.collect::<Vec<_>>();
    let mut parts = Vec::new();
    for group in ordered_groups {
        let count = collected.iter().filter(|entry| **entry == *group).count();
        if count > 0 {
            parts.push(format!("{group}({count})"));
        }
    }

    if parts.is_empty() { format!("{S7_NONE}(0)") } else { parts.join(", ") }
}

fn s7_hidden_impact_fallback_disclosure(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Option<String> {
    let advanced_context = advanced_context?;
    if advanced_context.semantic_policy_state == SemanticPolicyState::Local
        && advanced_context.semantic_capability_state != SemanticCapabilityState::Ready
    {
        let reason = advanced_context
            .terminal_reason
            .as_deref()
            .unwrap_or("semantic acceleration is unavailable; using baseline structured retrieval");
        return Some(format!("higher-order impact inference is unavailable because {reason}"));
    }

    None
}

fn s7_highest_priority_impact(entries: &[S7HiddenImpactEntry]) -> Option<&S7HiddenImpactEntry> {
    entries.iter().max_by_key(|entry| {
        (
            usize::from(entry.status == "open"),
            match entry.severity {
                S7_ASSUMPTION_RISK_HIGH => 3,
                S7_ASSUMPTION_RISK_MEDIUM => 2,
                _ => 1,
            },
        )
    })
}

fn s7_weakest_assumption(entries: &[S7AssumptionEntry]) -> String {
    entries
        .iter()
        .max_by_key(|entry| {
            (
                usize::from(entry.status == S7_ASSUMPTION_STATUS_MISSING),
                match entry.risk {
                    S7_ASSUMPTION_RISK_HIGH => 3,
                    S7_ASSUMPTION_RISK_MEDIUM => 2,
                    _ => 1,
                },
            )
        })
        .map(|entry| {
            format!("{} -> {} [{}/{}]", entry.category, entry.subject_ref, entry.status, entry.risk)
        })
        .unwrap_or_else(|| S7_WEAK_ASSUMPTION_NONE.to_string())
}

fn s7_challenge_strongest_objection(
    impacts: &[S7HiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    if let Some(impact) = s7_highest_priority_impact(impacts) {
        return match impact.group {
            S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS => {
                format!("missing test coverage is still open for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE => {
                format!("required evidence is still missing for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS => {
                format!("required reviewer coverage is still missing for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES => {
                format!("contract exposure still needs review for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS => {
                format!("system impact extends beyond the current slice for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS => {
                format!("domain impact extends beyond the current slice for {}", impact.subject_ref)
            }
            _ => impact.follow_up.clone(),
        };
    }

    hidden_impact_fallback_disclosure.unwrap_or(S7_RISK_NO_EXPLICIT_FAILURE).to_string()
}

fn s7_challenge_missing_evidence(
    impacts: &[S7HiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    let evidence_refs = impacts
        .iter()
        .filter(|impact| {
            impact.group == S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS
                || impact.group == S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE
        })
        .map(|impact| impact.subject_ref.clone())
        .collect::<Vec<_>>();
    if !evidence_refs.is_empty() {
        return evidence_refs.join(", ");
    }
    hidden_impact_fallback_disclosure.map(str::to_string).unwrap_or_else(|| S7_NONE.to_string())
}

fn s7_challenge_failure_mode(
    impacts: &[S7HiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    if let Some(impact) = s7_highest_priority_impact(impacts) {
        return match impact.group {
            S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS => {
                format!(
                    "bounded validation can regress without a focused test for {}",
                    impact.subject_ref
                )
            }
            S7_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE => {
                format!("the plan can proceed without required evidence for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS => {
                format!("review can miss critical dissent for {}", impact.subject_ref)
            }
            S7_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES => {
                format!(
                    "downstream consumers can break if {} changes without review",
                    impact.subject_ref
                )
            }
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS => {
                format!(
                    "cross-system impact can escape the bounded slice for {}",
                    impact.subject_ref
                )
            }
            S7_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS => {
                format!("domain invariants can drift for {}", impact.subject_ref)
            }
            _ => impact.follow_up.clone(),
        };
    }

    hidden_impact_fallback_disclosure
        .map(str::to_string)
        .unwrap_or_else(|| S7_RISK_NO_EXPLICIT_FAILURE.to_string())
}

fn s7_council_required(governance_present: bool) -> &'static str {
    if governance_present { S7_COUNCIL_REQUIRED_YES } else { S7_COUNCIL_REQUIRED_NO }
}

fn s7_cognitive_projection_for_trace_summary(
    summary: &TraceSummaryView,
    next_command: &str,
    fallback_disclosure: &str,
) -> S7CognitiveProjection {
    let assumptions = s7_assumption_entries(summary.advanced_context.as_ref());
    let impacts = s7_hidden_impact_entries(summary.advanced_context.as_ref());
    let hidden_impact_fallback_disclosure =
        s7_hidden_impact_fallback_disclosure(summary.advanced_context.as_ref());
    let governance_present = !summary.governance_timeline.is_empty()
        || summary.governance_reason.is_some()
        || summary.governance_approval_provenance.is_some()
        || summary.governance_next_action.is_some();
    let stage_text = if !summary.executed_steps.is_empty() {
        format!("{} step(s)", summary.executed_steps.len())
    } else {
        "trace_inspect".to_string()
    };
    let reasoning_why = reasoning_projection_why_summary(summary.reasoning_profile.as_ref())
        .unwrap_or_else(|| "none".to_string());

    S7CognitiveProjection {
        assumptions_summary: s7_assumption_summary(&assumptions),
        assumption_groups: s7_assumption_group_lines(&assumptions),
        hidden_impact_summary: s7_hidden_impact_summary(&impacts),
        hidden_impact_lines: s7_hidden_impact_lines(&impacts),
        hidden_impact_fallback_disclosure: hidden_impact_fallback_disclosure.clone(),
        challenge_strongest_objection: s7_challenge_strongest_objection(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_weakest_assumption: s7_weakest_assumption(&assumptions),
        challenge_missing_evidence: s7_challenge_missing_evidence(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_failure_mode: s7_challenge_failure_mode(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_required_review: if governance_present {
            "governance timeline remains authoritative".to_string()
        } else {
            S7_REVIEW_RUNTIME_ONLY.to_string()
        },
        challenge_council_required: s7_council_required(governance_present),
        explain_plan_summary: format!(
            "goal={}; stages={stage_text}; reasoning={reasoning_why}; risks={}; assumptions={}",
            summary.goal,
            s7_hidden_impact_summary(&impacts),
            s7_assumption_summary(&assumptions)
        ),
        explain_plan_validation: reasoning_projection_next_action(
            summary.reasoning_profile.as_ref(),
        )
        .or_else(|| {
            impacts
                .iter()
                .find(|impact| impact.group == S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS)
                .map(|impact| impact.follow_up.clone())
        })
        .unwrap_or_else(|| next_command.to_string()),
        explain_plan_governance: if let Some(reasoning_summary) =
            reasoning_projection_governance_summary(summary.reasoning_profile.as_ref())
        {
            reasoning_summary
        } else if governance_present {
            summary
                .governance_next_action
                .clone()
                .or_else(|| summary.governance_approval_provenance.clone())
                .or_else(|| summary.governance_reason.clone())
                .unwrap_or_else(|| fallback_disclosure.to_string())
        } else {
            fallback_disclosure.to_string()
        },
        explain_plan_recovery: reasoning_projection_next_action(summary.reasoning_profile.as_ref())
            .or_else(|| summary.latest_checkpoint_restore_command.clone())
            .unwrap_or_else(|| next_command.to_string()),
    }
}

fn s7_cognitive_projection_for_session_status(
    view: &SessionStatusView,
    fallback_disclosure: &str,
) -> S7CognitiveProjection {
    let assumptions = s7_assumption_entries(view.advanced_context.as_ref());
    let impacts = s7_hidden_impact_entries(view.advanced_context.as_ref());
    let hidden_impact_fallback_disclosure =
        s7_hidden_impact_fallback_disclosure(view.advanced_context.as_ref());
    let governance_present = view.latest_governance_packet_ref.is_some()
        || view.latest_governance_runtime.is_some()
        || view.latest_governance_decision.is_some()
        || view.latest_governance_reason.is_some()
        || view.governance_next_action.is_some();
    let stage_text = match (view.active_flow.as_deref(), view.flow_state.as_deref()) {
        (Some(flow), Some(state)) => format!("{flow}/{state}"),
        (Some(flow), None) => flow.to_string(),
        _ => view.current_stage_id.clone().unwrap_or_else(|| "session_state".to_string()),
    };
    let goal = view.goal.clone().unwrap_or_else(|| view.explanation.clone());
    let reasoning_why = reasoning_projection_why_summary(view.latest_reasoning_profile.as_ref())
        .unwrap_or_else(|| "none".to_string());

    S7CognitiveProjection {
        assumptions_summary: s7_assumption_summary(&assumptions),
        assumption_groups: s7_assumption_group_lines(&assumptions),
        hidden_impact_summary: s7_hidden_impact_summary(&impacts),
        hidden_impact_lines: s7_hidden_impact_lines(&impacts),
        hidden_impact_fallback_disclosure: hidden_impact_fallback_disclosure.clone(),
        challenge_strongest_objection: s7_challenge_strongest_objection(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_weakest_assumption: s7_weakest_assumption(&assumptions),
        challenge_missing_evidence: s7_challenge_missing_evidence(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_failure_mode: s7_challenge_failure_mode(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_required_review: if let Some(packet_ref) =
            view.latest_governance_packet_ref.as_deref()
        {
            format!("governance packet {packet_ref} remains authoritative")
        } else if governance_present {
            "governance runtime remains authoritative".to_string()
        } else {
            S7_REVIEW_RUNTIME_ONLY.to_string()
        },
        challenge_council_required: s7_council_required(governance_present),
        explain_plan_summary: format!(
            "goal={goal}; stages={stage_text}; reasoning={reasoning_why}; risks={}; assumptions={}",
            s7_hidden_impact_summary(&impacts),
            s7_assumption_summary(&assumptions)
        ),
        explain_plan_validation: reasoning_projection_next_action(
            view.latest_reasoning_profile.as_ref(),
        )
        .or_else(|| view.verification_strategy.clone())
        .or_else(|| {
            impacts
                .iter()
                .find(|impact| impact.group == S7_HIDDEN_IMPACT_GROUP_MISSING_TESTS)
                .map(|impact| impact.follow_up.clone())
        })
        .or_else(|| view.next_command.clone())
        .unwrap_or_else(|| view.explanation.clone()),
        explain_plan_governance: if let Some(reasoning_summary) =
            reasoning_projection_governance_summary(view.latest_reasoning_profile.as_ref())
        {
            reasoning_summary
        } else if let Some(packet_ref) = view.latest_governance_packet_ref.as_deref() {
            format!(
                "governance_packet={packet_ref}; council_required={}",
                s7_council_required(governance_present)
            )
        } else if governance_present {
            view.latest_governance_decision
                .clone()
                .or_else(|| view.governance_next_action.clone())
                .unwrap_or_else(|| fallback_disclosure.to_string())
        } else {
            fallback_disclosure.to_string()
        },
        explain_plan_recovery: reasoning_projection_next_action(
            view.latest_reasoning_profile.as_ref(),
        )
        .or_else(|| view.latest_checkpoint_restore_command.clone())
        .or_else(|| view.next_command.clone())
        .unwrap_or_else(|| view.explanation.clone()),
    }
}

fn diagnostic_follow_up_actions(report: &DiagnosticsReport) -> Vec<String> {
    if !report.ready {
        return Vec::new();
    }

    match report.subject {
        crate::cli::diagnostics::DiagnosticsSubject::Workspace => vec![format!(
            "- start a session: boundline start --workspace {}",
            report.workspace_ref.as_deref().unwrap_or("<workspace>")
        )],
        crate::cli::diagnostics::DiagnosticsSubject::Install => {
            vec!["- verify a workspace next: boundline doctor --workspace <workspace>".to_string()]
        }
    }
}

/// Renders doctor output for either a workspace or installation diagnostic run.
pub fn render_diagnostics(report: &DiagnosticsReport) -> String {
    let readiness = if report.ready { "ready" } else { "not ready" };
    let subject = match report.subject {
        crate::cli::diagnostics::DiagnosticsSubject::Workspace => format!(
            "workspace {}",
            report.workspace_ref.as_deref().unwrap_or("<unknown-workspace>")
        ),
        crate::cli::diagnostics::DiagnosticsSubject::Install => format!(
            "installation {}",
            report.installation_ref.as_deref().unwrap_or("<current-machine>")
        ),
    };
    let presentation = stdout_presentation();
    let mut lines = vec![format!("doctor: {readiness} for {subject}")];
    let mut summary_lines = vec![
        "- assistant_hint: Diagnostic output format is optimized for chat parsing.".to_string(),
    ];

    if let Some(boundline_version) = &report.boundline_version {
        summary_lines.push(format!("- boundline_version: {boundline_version}"));
    }
    if let Some(supported_canon_version) = &report.supported_canon_version {
        summary_lines.push(format!("- supported_canon_version: {supported_canon_version}"));
    }
    if let Some(companion_state) = report.companion_state {
        summary_lines.push(format!("- companion_state: {companion_state}"));
    }
    if !report.channel_candidates.is_empty() {
        summary_lines
            .push(format!("- channel_candidates: {}", report.channel_candidates.join(", ")));
    }
    push_output_section(&mut lines, presentation, "summary", summary_lines);

    let check_lines = report
        .checks
        .iter()
        .map(|check| {
            let status = match check.status {
                DiagnosticsStatus::Passed => "passed",
                DiagnosticsStatus::Advisory => "advisory",
                DiagnosticsStatus::Failed => "failed",
            };
            format!("- {}: {} - {}", check.name, status, check.message)
        })
        .collect::<Vec<_>>();
    push_output_section(&mut lines, presentation, "checks", check_lines);

    let mut action_lines = Vec::new();
    for action in &report.suggested_actions {
        let rendered = format!("- {action}");
        if !action_lines.iter().any(|existing| existing == &rendered) {
            action_lines.push(rendered);
        }
    }
    for action in diagnostic_follow_up_actions(report) {
        if !action_lines.iter().any(|existing| existing == &action) {
            action_lines.push(action);
        }
    }
    push_output_section(&mut lines, presentation, "actions", action_lines);

    lines.join("\n")
}

/// Renders the result of a `run` command from the persisted trace and terminal response.
pub fn render_run_trace(
    command_name: &str,
    trace: Option<&ExecutionTrace>,
    response: &TaskRunResponse,
    next_command: &str,
) -> String {
    let mut lines = vec![format!("{command_name}: {}", response.terminal_reason.message)];

    if let Some(trace) = trace {
        let mut context_summary: Option<String> = None;
        let mut context_credibility: Option<String> = None;
        let mut context_primary_inputs: Vec<String> = Vec::new();
        let mut context_provenance: Vec<String> = Vec::new();
        let mut context_staleness_reason: Option<String> = None;
        let mut governance_next_action: Option<String> = None;
        let mut reasoning_profile: Option<ProfileActivationRecord> = None;
        lines.insert(0, format!("goal: {}", trace.goal));
        lines.insert(1, format!("route_owner: {}", run_trace_route_owner(trace)));
        if let Some(route_config_projection) = render_route_config_projection(
            route_config_projection_for_run_trace(trace, Path::new(&response.trace_location)),
        ) {
            lines.insert(2, route_config_projection);
        }

        if let Some(input) = trace.events.iter().find_map(|event| {
            (event.event_type == TraceEventType::TaskStarted)
                .then(|| event.payload.get("input"))
                .flatten()
        }) {
            if let Some(authored_input_summary) =
                input.get("authored_input_summary").and_then(Value::as_str)
            {
                lines.push(format!("authored_input_summary: {authored_input_summary}"));
            }
            if let Some(clarification_headline) =
                input.get("clarification_headline").and_then(Value::as_str)
            {
                lines.push(format!("clarification_headline: {clarification_headline}"));
            }
            if let Some(clarification_prompt) =
                input.get("clarification_prompt").and_then(Value::as_str)
            {
                lines.push(format!("clarification_prompt: {clarification_prompt}"));
            }
            if let Some(negotiation_goal_summary) =
                input.get("negotiation_goal_summary").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
            }
            if let Some(negotiation_resolution) =
                input.get("negotiation_resolution").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
            }
            if let Some(negotiation_acceptance_boundary) =
                input.get("negotiation_acceptance_boundary").and_then(Value::as_str)
            {
                lines.push(format!(
                    "negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"
                ));
            }
            context_summary = input
                .get("context_summary")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_summary);
            context_credibility = input
                .get("context_credibility")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_credibility);
            if context_primary_inputs.is_empty() {
                context_primary_inputs = input
                    .get("context_primary_inputs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if context_provenance.is_empty() {
                context_provenance = input
                    .get("context_provenance")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            context_staleness_reason = input
                .get("context_staleness_reason")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_staleness_reason);
        }

        if let Some(goal_plan_created) =
            trace.events.iter().find(|event| event.event_type == TraceEventType::GoalPlanCreated)
        {
            if let Some(negotiation_goal_summary) =
                goal_plan_created.payload.get("negotiation_goal_summary").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
            }
            if let Some(negotiation_resolution) =
                goal_plan_created.payload.get("negotiation_resolution").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
            }
            if let Some(negotiation_acceptance_boundary) = goal_plan_created
                .payload
                .get("negotiation_acceptance_boundary")
                .and_then(Value::as_str)
            {
                lines.push(format!(
                    "negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"
                ));
            }
            context_summary = goal_plan_created
                .payload
                .get("context_summary")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_summary);
            context_credibility = goal_plan_created
                .payload
                .get("context_credibility")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_credibility);
            if context_primary_inputs.is_empty() {
                context_primary_inputs = goal_plan_created
                    .payload
                    .get("context_primary_inputs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if context_provenance.is_empty() {
                context_provenance = goal_plan_created
                    .payload
                    .get("context_provenance")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            context_staleness_reason = goal_plan_created
                .payload
                .get("context_staleness_reason")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_staleness_reason);
        }

        for event in &trace.events {
            if !matches!(
                event.event_type,
                TraceEventType::GovernanceAwaitingApproval
                    | TraceEventType::GovernanceCompleted
                    | TraceEventType::GovernanceBlocked
                    | TraceEventType::GovernancePacketRejected
            ) {
                continue;
            }

            if context_summary.is_none() {
                context_summary = event
                    .payload
                    .get("canon_memory_summary")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_summary);
            }
            if context_credibility.is_none() {
                context_credibility = event
                    .payload
                    .get("canon_memory_credibility")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_credibility);
            }
            if context_primary_inputs.is_empty() {
                context_primary_inputs = event
                    .payload
                    .get("document_refs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if let Some(canon_memory_summary) =
                event.payload.get("canon_memory_summary").and_then(Value::as_str)
            {
                let line = format!("canon_memory: {canon_memory_summary}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_compatibility) =
                event.payload.get("canon_memory_compatibility").and_then(Value::as_str)
            {
                let line = format!("canon_memory_compatibility: {canon_memory_compatibility}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_run_ref) = event
                .payload
                .get("canon_memory_run_ref")
                .or_else(|| event.payload.get("run_ref"))
                .and_then(Value::as_str)
            {
                let line = format!("canon_memory_run_ref: {canon_memory_run_ref}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_packet_ref) = event
                .payload
                .get("canon_memory_packet_ref")
                .or_else(|| event.payload.get("packet_ref"))
                .and_then(Value::as_str)
            {
                let line = format!("canon_memory_packet: {canon_memory_packet_ref}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_reason_code) =
                event.payload.get("canon_memory_reason_code").and_then(Value::as_str)
            {
                let line = format!("canon_memory_reason: {canon_memory_reason_code}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_next_action) =
                event.payload.get("canon_next_action").and_then(Value::as_str)
            {
                let line = format!("canon_memory_next_action: {canon_next_action}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(authority_provenance_lines) =
                event.payload.get("authority_provenance_lines").and_then(Value::as_array)
            {
                for line in authority_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if let Some(adaptive_provenance_lines) =
                event.payload.get("adaptive_provenance_lines").and_then(Value::as_array)
            {
                for line in adaptive_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if let Some(semantic_provenance_lines) =
                event.payload.get("semantic_provenance_lines").and_then(Value::as_array)
            {
                for line in semantic_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if context_staleness_reason.is_none()
                && event
                    .payload
                    .get("canon_memory_credibility")
                    .and_then(Value::as_str)
                    .is_some_and(|credibility| credibility != "credible")
            {
                context_staleness_reason = event
                    .payload
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_staleness_reason);
            }
            if governance_next_action.is_none() {
                governance_next_action = event
                    .payload
                    .get("canon_next_action")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            if let Some(record) = event
                .payload
                .get("reasoning_profile_record")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok())
            {
                reasoning_profile = Some(record);
            }
        }

        push_context_projection_lines(
            &mut lines,
            context_summary.as_deref(),
            context_credibility.as_deref(),
            &context_primary_inputs,
            &context_provenance,
            context_staleness_reason.as_deref(),
        );

        if let Some(reasoning_profile) = &reasoning_profile {
            append_reasoning_profile_lines(&mut lines, "", reasoning_profile);
        }

        for event in &trace.events {
            if let Some(governance_next_action) = governance_next_action.as_ref() {
                lines.push(format!("governance_next_action: {governance_next_action}"));
            }
            match event.event_type {
                TraceEventType::TaskStarted
                | TraceEventType::TerminalRecorded
                | TraceEventType::ReviewerStarted => {}
                TraceEventType::FlowSelected => {
                    let flow_name = event
                        .payload
                        .get("flow_name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-flow");
                    let stage_id = event
                        .payload
                        .get("current_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("flow {flow_name} selected at {stage_id}"));
                }
                TraceEventType::CheckpointCreated => {
                    let checkpoint_id = event
                        .payload
                        .get("checkpoint_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-checkpoint");
                    let checkpoint_scope = event
                        .payload
                        .get("checkpoint_scope")
                        .and_then(Value::as_str)
                        .unwrap_or("workspace");
                    lines.push(format!("checkpoint {checkpoint_id} created ({checkpoint_scope})"));
                }
                TraceEventType::StageTransitioned => {
                    let from_stage = event
                        .payload
                        .get("from_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let to_stage = event
                        .payload
                        .get("to_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("stage {from_stage} -> {to_stage}"));
                }
                TraceEventType::StepStarted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let step_kind =
                        event.payload.get("step_kind").and_then(Value::as_str).unwrap_or("step");
                    lines.push(format!("step {step_id} ({step_kind}) started"));
                }
                TraceEventType::StepCompleted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let status =
                        event.payload.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("step {step_id} {status}"));

                    if let Some(changed_files) = event
                        .payload
                        .get("output")
                        .and_then(|output| output.get("changed_files"))
                        .and_then(value_as_string_list)
                        && !changed_files.is_empty()
                    {
                        lines.push(format!("changed_files: {}", changed_files.join(", ")));
                    }

                    if let Some(validation_line) = validation_line_from_event(&event.payload) {
                        lines.push(validation_line);
                    }
                }
                TraceEventType::DecisionCreated => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let decision_type = event
                        .payload
                        .get("decision_type")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!(
                        "decision {decision_id} created: {decision_type} -> {target}"
                    ));
                }
                TraceEventType::DecisionDispatched => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("decision {decision_id} dispatched: {target}"));
                }
                TraceEventType::DecisionVerified => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} verified"));
                }
                TraceEventType::DecisionFailed => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} failed"));
                }
                TraceEventType::DecisionRecovered => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} recovered"));
                }
                TraceEventType::ReasoningProfileActivated
                | TraceEventType::ReasoningParticipantStarted
                | TraceEventType::ReasoningParticipantCompleted
                | TraceEventType::ReasoningDisagreementRecorded
                | TraceEventType::ReasoningDebateRoundCompleted
                | TraceEventType::ReasoningReflexionRevisionCompleted
                | TraceEventType::ReasoningAdjudicationRecorded
                | TraceEventType::ReasoningConfidenceRecorded
                | TraceEventType::ReasoningProfileBlocked
                | TraceEventType::ReasoningProfileInterrupted
                | TraceEventType::ReasoningProfileEscalated => {}
                TraceEventType::GovernanceSelected
                | TraceEventType::GovernanceStarted
                | TraceEventType::GovernanceDecisionRecorded
                | TraceEventType::GovernanceAwaitingApproval
                | TraceEventType::GovernanceCompleted
                | TraceEventType::GovernanceBlocked
                | TraceEventType::GovernancePacketRejected => {
                    if let Some(line) = governance_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::RetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("retry for {step_id}: {reason}"));
                }
                TraceEventType::StageRetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("stage retry for {step_id}: {reason}"));
                }
                TraceEventType::Replanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("replan after {step_id}: {reason}"));
                }
                TraceEventType::StageReplanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("stage replan after {step_id}: {reason}"));
                }
                TraceEventType::StageFailed => {
                    let stage_id = event
                        .payload
                        .get(KEY_STAGE_ID)
                        .and_then(Value::as_str)
                        .unwrap_or(UNKNOWN_STAGE_ID);
                    let reason = event
                        .payload
                        .get(KEY_REASON)
                        .and_then(Value::as_str)
                        .unwrap_or("stage failed");
                    lines.push(format!("stage {stage_id} failed: {reason}"));
                }
                TraceEventType::ReviewStarted
                | TraceEventType::ReviewTriggerIgnored
                | TraceEventType::ReviewerCompleted
                | TraceEventType::ReviewVoteResolved
                | TraceEventType::ReviewAdjudicated
                | TraceEventType::ReviewTerminalRecorded => {
                    if let Some(line) = review_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::ProjectScalePathProposed
                | TraceEventType::ProjectScaleStageTransitioned
                | TraceEventType::VotingDecisionRecorded => {}
                TraceEventType::GoalPlanCreated => {
                    let goal =
                        event.payload.get("goal").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("goal plan created: {goal}"));
                }
                TraceEventType::FlowInferred => {
                    let flow =
                        event.payload.get("flow_name").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("flow inferred: {flow}"));
                }
            }
        }

        lines.push(render_run_execution_condition(response));
    }

    if trace.is_none() {
        lines.push(render_run_execution_condition(response));
    }

    if let Some(workspace_slice) = adaptive_workspace_slice_summary(&response.final_context.state) {
        lines.push(format!("workspace_slice: {workspace_slice}"));
    }

    if let Some(attempt_lineage) = adaptive_attempt_lineage_summary(&response.final_context.state) {
        lines.push(format!("attempt_lineage: {attempt_lineage}"));
    }

    if let Some(candidate_family) = adaptive_candidate_family_summary(&response.final_context.state)
    {
        lines.push(format!("candidate_family: {candidate_family}"));
    }

    if let Some(selection_reason) = adaptive_selection_reason_summary(&response.final_context.state)
    {
        lines.push(format!("selection_reason: {selection_reason}"));
    }

    if let Some(rejected_candidates) =
        adaptive_rejected_candidates_summary(&response.final_context.state)
    {
        lines.push(format!("rejected_candidates: {rejected_candidates}"));
    }

    if let Some(exhaustion_reason) =
        adaptive_exhaustion_reason_summary(&response.final_context.state)
    {
        lines.push(format!("adaptive_exhaustion: {exhaustion_reason}"));
    }

    let (latest_checkpoint_id, latest_checkpoint_scope, latest_checkpoint_restore_command) =
        checkpoint_projection_from_state(&response.final_context.state);
    if let Some(latest_checkpoint_id) = latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }
    if let Some(latest_checkpoint_scope) = latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }
    if let Some(latest_checkpoint_restore_command) = latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    if let Ok(Some(cluster_story)) = response.final_context.cluster_delivery_story() {
        lines.extend(render_cluster_story_lines(&cluster_story));
    }

    lines.push(format!("terminal_status: {}", task_status_text(response.terminal_status)));
    lines.push(format!("terminal_reason: {}", response.terminal_reason.message));
    lines.push(format!("trace: {}", response.trace_location));
    lines.push(format!("next_command: {next_command}"));
    lines.join("\n")
}

/// Renders the persisted trace summary as operator-facing text without
/// recomputing planning, routing, or guidance state.
pub fn render_trace_summary(
    summary: &TraceSummaryView,
    inspection_target: &str,
    next_command: &str,
) -> String {
    let mut lines = vec![
        format!("inspection_target: {inspection_target}"),
        format!("trace: {}", summary.trace_ref),
        format!("goal: {}", summary.goal),
    ];

    if let Some(cluster_story) = &summary.cluster_delivery_story {
        lines.extend(render_cluster_story_lines(cluster_story));
    }

    if let Some(routing_summary) = &summary.routing_summary {
        lines.push(routing_summary.clone());
    }

    lines.push(format!("route_owner: {}", trace_route_owner(summary)));
    if let Some(route_config_projection) =
        render_route_config_projection(route_config_projection_for_trace_summary(summary))
    {
        lines.push(route_config_projection);
    }

    lines.push(render_trace_execution_condition(summary));

    if let Some(goal_plan_summary) = &summary.goal_plan_summary {
        lines.push(format!("goal_plan_summary: {goal_plan_summary}"));
    }

    if let Some(negotiation_goal_summary) = &summary.negotiation_goal_summary {
        lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
    }

    if let Some(negotiation_resolution) = &summary.negotiation_resolution {
        lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
    }

    if let Some(negotiation_acceptance_boundary) = &summary.negotiation_acceptance_boundary {
        lines.push(format!("negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"));
    }

    if let Some(authored_input_summary) = &summary.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if !summary.authored_input_sources.is_empty() {
        lines
            .push(format!("authored_input_sources: {}", summary.authored_input_sources.join(", ")));
    }

    if !summary.authored_input_deduplicated_sources.is_empty() {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            summary.authored_input_deduplicated_sources.join(", ")
        ));
    }

    push_context_projection_lines(
        &mut lines,
        summary.context_summary.as_deref(),
        summary.context_credibility.as_deref(),
        &summary.context_primary_inputs,
        &summary.context_provenance,
        summary.context_staleness_reason.as_deref(),
    );

    push_advanced_context_lines(&mut lines, summary.advanced_context.as_ref());

    lines.extend(render_guidance_projection_lines(&summary.guidance_guardian));

    if let Some(clarification_headline) = &summary.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &summary.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if !summary.clarification_missing_fields.is_empty() {
        lines.push(format!(
            "clarification_missing_fields: {}",
            summary.clarification_missing_fields.join(", ")
        ));
    }

    if let Some(requested_governance_runtime) = &summary.requested_governance_runtime {
        lines.push(format!("requested_governance_runtime: {requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &summary.requested_governance_risk {
        lines.push(format!("requested_governance_risk: {requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &summary.requested_governance_zone {
        lines.push(format!("requested_governance_zone: {requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &summary.requested_governance_owner {
        lines.push(format!("requested_governance_owner: {requested_governance_owner}"));
    }

    if !summary.decision_timeline.is_empty() {
        lines.push("decision_timeline:".to_string());
        lines.extend(summary.decision_timeline.iter().cloned());
    }

    if !summary.failure_evidence.is_empty() {
        lines.push("failure_evidence:".to_string());
        lines.extend(summary.failure_evidence.iter().cloned());
    }

    if !summary.adaptive_evidence.is_empty() {
        lines.push("adaptive_evidence:".to_string());
        lines.extend(summary.adaptive_evidence.iter().cloned());
    }

    if let Some(latest_checkpoint_id) = &summary.latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }

    if let Some(latest_checkpoint_scope) = &summary.latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }

    if let Some(latest_checkpoint_restore_command) = &summary.latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    for step in &summary.executed_steps {
        lines.push(format!(
            "step {} ({}) {} [{} attempt(s)] - {}",
            step.step_id,
            step_kind_text(step.step_kind),
            step_status_text(step.final_status),
            step.attempts,
            step.headline,
        ));
    }

    for recovery in &summary.recovery_events {
        let label = match recovery.event_type {
            TraceEventType::RetryScheduled => "retry",
            TraceEventType::StageRetryScheduled => "stage_retry",
            TraceEventType::Replanned => "replan",
            TraceEventType::StageReplanned => "stage_replan",
            TraceEventType::FlowSelected => "flow",
            TraceEventType::StageTransitioned => "stage",
            TraceEventType::StageFailed => "stage_failure",
            _ => "recovery",
        };
        lines.push(format!("{label}: {}", recovery.trigger));
    }

    lines.extend(summary.governance_timeline.iter().cloned());

    if let Some(governance_runtime_state) = &summary.governance_runtime_state {
        lines.push(format!("governance_runtime_state: {governance_runtime_state}"));
    }

    if let Some(governance_rollout_profile) = &summary.governance_rollout_profile {
        lines.push(format!("governance_rollout_profile: {governance_rollout_profile}"));
    }

    if let Some(governance_reason) = &summary.governance_reason {
        lines.push(format!("governance_reason: {governance_reason}"));
    }

    if let Some(governance_approval_provenance) = &summary.governance_approval_provenance {
        lines.push(format!("governance_approval_provenance: {governance_approval_provenance}"));
    }

    if let Some(governance_next_action) = &summary.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }

    if let Some(reasoning_profile) = &summary.reasoning_profile {
        append_reasoning_profile_lines(&mut lines, "", reasoning_profile);
    }

    if let Some(delegation) = &summary.delegation {
        lines.push(format!("delegation_mode: {}", delegation.mode.as_str()));
        if let Some(packet_id) = &delegation.packet_id {
            lines.push(format!("delegation_packet_id: {packet_id}"));
        }
        if let Some(packet_kind) = delegation.packet_kind {
            lines.push(format!("delegation_packet_kind: {}", packet_kind.as_str()));
        }
        if let Some(packet_state) = delegation.packet_state {
            lines.push(format!("delegation_packet_state: {}", packet_state.as_str()));
        }
        if let Some(target_owner) = &delegation.target_owner {
            lines.push(format!("delegation_target_owner: {target_owner}"));
        }
        lines.push(format!("delegation_headline: {}", delegation.headline));
        lines.push(format!("delegation_evidence_summary: {}", delegation.evidence_summary));
    }

    let s7_projection = s7_projection_for_trace_summary(summary, next_command);
    lines.extend(s7_projection_lines(&s7_projection));
    lines.extend(s7_cognitive_projection_lines(&s7_cognitive_projection_for_trace_summary(
        summary,
        next_command,
        &s7_projection.fallback_disclosure,
    )));

    let follow_through = FollowThroughProjection::from_trace_summary(summary, Some(next_command));
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }

    lines.extend(summary.review_timeline.iter().cloned());

    lines.push(format!("terminal_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    lines.push(format!("next_command: {next_command}"));

    if let Some(duration) = summary.duration {
        lines.push(format!("duration_ms: {duration}"));
    }

    lines.join("\n")
}

/// Renders an inspect failure while preserving the trace-resolution context.
pub fn render_inspect_failure(
    inspection_target: &str,
    trace_ref: Option<&str>,
    workspace_ref: Option<&str>,
    terminal_reason: &str,
    corrected_command: &str,
) -> String {
    let mut lines = vec![
        "inspect: trace read failure".to_string(),
        format!("inspection_target: {inspection_target}"),
        format!("terminal_reason: {terminal_reason}"),
    ];

    if let Some(trace_ref) = trace_ref {
        lines.push(format!("trace: {trace_ref}"));
    }

    if let Some(workspace_ref) = workspace_ref {
        lines.push(format!("workspace_ref: {workspace_ref}"));
    }

    lines.push("next_command: /boundline-inspect".to_string());
    lines.push(format!("corrected_command: {corrected_command}"));
    lines.join("\n")
}

/// Shared prefix for session-status style projections.
pub fn render_session_projection_prefix(view: &SessionStatusView) -> String {
    [
        render_route_outcome(&routing_outcome_for_status_view(view)),
        render_session_execution_condition(view),
    ]
    .join("\n")
}

/// Renders the persisted session view as the operator-facing status surface.
pub fn render_session_status(view: &SessionStatusView) -> String {
    let mut lines = vec![
        format!("session_id: {}", view.session_id),
        format!("workspace_ref: {}", view.workspace_ref),
    ];

    if let Some(goal) = &view.goal {
        lines.push(format!("goal: {goal}"));
    }

    if let Some(negotiation_goal_summary) = &view.negotiation_goal_summary {
        lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
    }

    if let Some(negotiation_resolution) = &view.negotiation_resolution {
        lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
    }

    if let Some(negotiation_acceptance_boundary) = &view.negotiation_acceptance_boundary {
        lines.push(format!("negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"));
    }

    lines.extend(render_session_projection_prefix(view).lines().map(str::to_string));
    lines.push(format!("route_owner: {}", session_route_owner(view)));

    if let Some(route_config_projection) =
        render_route_config_projection(route_config_projection_for_status_view(view))
    {
        lines.push(route_config_projection);
    }

    if let Some(continuity_authority) = view.continuity_authority {
        lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    }

    if let Some(delegation) = &view.delegation {
        lines.push(format!("delegation_mode: {}", delegation.mode.as_str()));
        if let Some(packet_id) = &delegation.packet_id {
            lines.push(format!("delegation_packet_id: {packet_id}"));
        }
        if let Some(packet_kind) = delegation.packet_kind {
            lines.push(format!("delegation_packet_kind: {}", packet_kind.as_str()));
        }
        if let Some(packet_state) = delegation.packet_state {
            lines.push(format!("delegation_packet_state: {}", packet_state.as_str()));
        }
        if let Some(target_owner) = &delegation.target_owner {
            lines.push(format!("delegation_target_owner: {target_owner}"));
        }
        lines.push(format!("delegation_headline: {}", delegation.headline));
        lines.push(format!("delegation_evidence_summary: {}", delegation.evidence_summary));
    }

    if let Some(compatibility_follow_up) = &view.compatibility_follow_up {
        lines.extend(render_compatibility_follow_up_lines(
            compatibility_follow_up,
            "compatibility_routing",
            "compatibility_execution_condition",
            "compatibility_follow_up_command",
        ));
    }

    if let Some(cluster_story) = &view.cluster_delivery_story {
        lines.extend(render_cluster_story_lines(cluster_story));
    }

    if let Some(authored_input_summary) = &view.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if let Some(authored_input_sources) = &view.authored_input_sources
        && !authored_input_sources.is_empty()
    {
        lines.push(format!("authored_input_sources: {}", authored_input_sources.join(", ")));
    }

    if let Some(authored_input_deduplicated_sources) = &view.authored_input_deduplicated_sources
        && !authored_input_deduplicated_sources.is_empty()
    {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            authored_input_deduplicated_sources.join(", ")
        ));
    }

    push_context_projection_lines(
        &mut lines,
        view.context_summary.as_deref(),
        view.context_credibility.as_deref(),
        view.context_primary_inputs.as_deref().unwrap_or(&[]),
        view.context_provenance.as_deref().unwrap_or(&[]),
        view.context_staleness_reason.as_deref(),
    );

    push_advanced_context_lines(&mut lines, view.advanced_context.as_ref());

    if let Some(clarification_headline) = &view.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &view.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if let Some(clarification_missing_fields) = &view.clarification_missing_fields
        && !clarification_missing_fields.is_empty()
    {
        lines.push(format!(
            "clarification_missing_fields: {}",
            clarification_missing_fields.join(", ")
        ));
    }

    if let Some(requested_governance_runtime) = &view.requested_governance_runtime {
        lines.push(format!("requested_governance_runtime: {requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &view.requested_governance_risk {
        lines.push(format!("requested_governance_risk: {requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &view.requested_governance_zone {
        lines.push(format!("requested_governance_zone: {requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &view.requested_governance_owner {
        lines.push(format!("requested_governance_owner: {requested_governance_owner}"));
    }

    if let Some(active_flow) = &view.active_flow {
        lines.push(format!("active_flow: {active_flow}"));
    }

    if let Some(flow_state) = &view.flow_state {
        lines.push(format!("flow_state: {flow_state}"));
    }

    if let Some(goal_plan_state) = &view.goal_plan_state {
        lines.push(format!("goal_plan_state: {goal_plan_state}"));
    }

    if let Some(goal_plan_revision) = view.goal_plan_revision {
        lines.push(format!("goal_plan_revision: {goal_plan_revision}"));
    }

    if let Some(planning_rationale) = &view.planning_rationale {
        lines.push(format!("planning_rationale: {planning_rationale}"));
    }

    if let Some(verification_strategy) = &view.verification_strategy {
        lines.push(format!("verification_strategy: {verification_strategy}"));
    }

    if let Some(active_workflow) = &view.active_workflow {
        lines.push(format!("workflow: {active_workflow}"));
    }

    if let Some(workflow_phase) = &view.workflow_phase {
        lines.push(format!("workflow_phase: {workflow_phase}"));
    }

    if let Some(current_stage_id) = &view.current_stage_id {
        lines.push(format!("current_stage: {current_stage_id}"));
    }

    if let (Some(current_stage_index), Some(total_stages)) =
        (view.current_stage_index, view.total_stages)
    {
        lines.push(format!("stage_progress: {}/{}", current_stage_index + 1, total_stages));
    }

    if let Some(plan_revision) = view.plan_revision {
        lines.push(format!("plan_revision: {plan_revision}"));
    }

    if let Some(current_step_index) = view.current_step_index {
        lines.push(format!("current_step_index: {current_step_index}"));
    }

    if let Some(current_step_id) = &view.current_step_id {
        lines.push(format!("current_step_id: {current_step_id}"));
    }

    lines.push(format!("latest_status: {}", session_status_text(view.latest_status)));

    if let Some(execution_path) = &view.execution_path {
        lines.push(format!("execution_path: {execution_path}"));
    }

    if let Some(latest_trace_ref) = &view.latest_trace_ref {
        lines.push(format!("latest_trace_ref: {latest_trace_ref}"));
    }

    if let Some(latest_decision_status) = &view.latest_decision_status {
        lines.push(format!("latest_decision_status: {latest_decision_status}"));
    }

    if let Some(latest_decision_target) = &view.latest_decision_target {
        lines.push(format!("latest_decision_target: {latest_decision_target}"));
    }

    if let Some(latest_changed_files) = &view.latest_changed_files
        && !latest_changed_files.is_empty()
    {
        lines.push(format!("latest_changed_files: {}", latest_changed_files.join(", ")));
    }

    if let Some(latest_checkpoint_id) = &view.latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }

    if let Some(latest_checkpoint_scope) = &view.latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }

    if let Some(latest_checkpoint_restore_command) = &view.latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    if let Some(latest_workspace_slice) = &view.latest_workspace_slice {
        lines.push(format!("latest_workspace_slice: {latest_workspace_slice}"));
    }

    if let Some(latest_selection_headline) = &view.latest_selection_headline {
        lines.push(format!("latest_selection_headline: {latest_selection_headline}"));
    }

    if let Some(latest_candidate_family) = &view.latest_candidate_family {
        lines.push(format!("latest_candidate_family: {latest_candidate_family}"));
    }

    if let Some(latest_selection_reason) = &view.latest_selection_reason {
        lines.push(format!("latest_selection_reason: {latest_selection_reason}"));
    }

    if let Some(latest_rejected_candidates) = &view.latest_rejected_candidates
        && !latest_rejected_candidates.is_empty()
    {
        lines.push(format!(
            "latest_rejected_candidates: {}",
            latest_rejected_candidates.join(" | ")
        ));
    }

    if let Some(latest_attempt_lineage) = &view.latest_attempt_lineage {
        lines.push(format!("latest_attempt_lineage: {latest_attempt_lineage}"));
    }

    if let Some(latest_validation_status) = &view.latest_validation_status {
        lines.push(format!("latest_validation_status: {latest_validation_status}"));
    }

    if let Some(latest_exhaustion_reason) = &view.latest_exhaustion_reason {
        lines.push(format!("latest_exhaustion_reason: {latest_exhaustion_reason}"));
    }

    if let Some(latest_review_trigger) = &view.latest_review_trigger {
        lines.push(format!("latest_review_trigger: {latest_review_trigger}"));
    }

    if let Some(latest_review_vote) = &view.latest_review_vote {
        lines.push(format!("latest_review_vote: {latest_review_vote}"));
    }

    if let Some(latest_review_outcome) = &view.latest_review_outcome {
        lines.push(format!("latest_review_outcome: {latest_review_outcome}"));
    }

    if let Some(latest_review_council_profile) = &view.latest_review_council_profile {
        lines.push(format!("latest_review_council_profile: {latest_review_council_profile}"));
    }

    if let Some(latest_review_independence_state) = &view.latest_review_independence_state {
        lines.push(format!("latest_review_independence_state: {latest_review_independence_state}"));
    }

    if let Some(latest_review_stop_semantics) = &view.latest_review_stop_semantics {
        lines.push(format!("latest_review_stop_semantics: {latest_review_stop_semantics}"));
    }

    if let Some(latest_review_selection_summary) = &view.latest_review_selection_summary {
        lines.push(format!("latest_review_selection_summary: {latest_review_selection_summary}"));
    }

    if let Some(latest_review_headline) = &view.latest_review_headline {
        lines.push(format!("latest_review_headline: {latest_review_headline}"));
    }

    if let Some(latest_governance_stage) = &view.latest_governance_stage {
        lines.push(format!("latest_governance_stage: {latest_governance_stage}"));
    }

    if let Some(latest_governance_runtime) = &view.latest_governance_runtime {
        lines.push(format!("latest_governance_runtime: {latest_governance_runtime}"));
    }

    if let Some(latest_governance_mode) = &view.latest_governance_mode {
        lines.push(format!("latest_governance_mode: {latest_governance_mode}"));
    }

    if let Some(latest_governance_run_ref) = &view.latest_governance_run_ref {
        lines.push(format!("latest_governance_run_ref: {latest_governance_run_ref}"));
    }

    if let Some(latest_governance_state) = &view.latest_governance_state {
        lines.push(format!("latest_governance_state: {latest_governance_state}"));
    }

    if let Some(latest_governance_runtime_state) = &view.latest_governance_runtime_state {
        lines.push(format!("latest_governance_runtime_state: {latest_governance_runtime_state}"));
    }

    if let Some(latest_governance_rollout_profile) = &view.latest_governance_rollout_profile {
        lines.push(format!(
            "latest_governance_rollout_profile: {latest_governance_rollout_profile}"
        ));
    }

    if let Some(latest_governance_reason) = &view.latest_governance_reason {
        lines.push(format!("latest_governance_reason: {latest_governance_reason}"));
    }

    if let Some(latest_governance_contract_lines) = &view.latest_governance_contract_lines
        && !latest_governance_contract_lines.is_empty()
    {
        lines.push(format!(
            "latest_governance_contract_lines: {}",
            latest_governance_contract_lines.join(" | ")
        ));
    }

    if let Some(latest_governance_approval_provenance) = &view.latest_governance_approval_provenance
    {
        lines.push(format!(
            "latest_governance_approval_provenance: {latest_governance_approval_provenance}"
        ));
    }

    if let Some(latest_governance_blocked_reason) = &view.latest_governance_blocked_reason {
        lines.push(format!("latest_governance_blocked_reason: {latest_governance_blocked_reason}"));
    }

    if let Some(latest_governance_packet_ref) = &view.latest_governance_packet_ref {
        lines.push(format!("latest_governance_packet_ref: {latest_governance_packet_ref}"));
    }

    if let Some(latest_governance_packet_source_stage) = &view.latest_governance_packet_source_stage
    {
        lines.push(format!(
            "latest_governance_packet_source_stage: {latest_governance_packet_source_stage}"
        ));
    }

    if let Some(latest_governance_packet_binding_reason) =
        &view.latest_governance_packet_binding_reason
    {
        lines.push(format!(
            "latest_governance_packet_binding_reason: {latest_governance_packet_binding_reason}"
        ));
    }

    if let Some(latest_governance_approval) = &view.latest_governance_approval {
        lines.push(format!("latest_governance_approval: {latest_governance_approval}"));
    }

    if let Some(latest_governance_decision) = &view.latest_governance_decision {
        lines.push(format!("latest_governance_decision: {latest_governance_decision}"));
    }

    if let Some(latest_governance_candidates) = &view.latest_governance_candidates
        && !latest_governance_candidates.is_empty()
    {
        lines.push(format!(
            "latest_governance_candidates: {}",
            latest_governance_candidates.join(", ")
        ));
    }

    if let Some(project_scale_path) = &view.project_scale_path {
        lines.push(format!("project_scale_path: {project_scale_path}"));
    }
    if let Some(project_scale_current_stage) = &view.project_scale_current_stage {
        lines.push(format!("project_scale_current_stage: {project_scale_current_stage}"));
    }
    if let Some(project_scale_next_action) = &view.project_scale_next_action {
        lines.push(format!("project_scale_next_action: {project_scale_next_action}"));
    }
    if let Some(project_scale_checkpoint_refs) = &view.project_scale_checkpoint_refs
        && !project_scale_checkpoint_refs.is_empty()
    {
        lines.push(format!(
            "project_scale_checkpoint_refs: {}",
            project_scale_checkpoint_refs.join(", ")
        ));
    }
    if let Some(latest_voting_trigger) = &view.latest_voting_trigger {
        lines.push(format!("latest_voting_trigger: {latest_voting_trigger}"));
    }
    if let Some(latest_voting_result) = &view.latest_voting_result {
        lines.push(format!("latest_voting_result: {latest_voting_result}"));
    }
    if let Some(latest_voting_adjudication) = &view.latest_voting_adjudication {
        lines.push(format!("latest_voting_adjudication: {latest_voting_adjudication}"));
    }
    if let Some(latest_voting_reviewed_evidence) = &view.latest_voting_reviewed_evidence {
        lines.push(format!("latest_voting_reviewed_evidence: {latest_voting_reviewed_evidence}"));
    }
    if let Some(latest_voting_blocking) = view.latest_voting_blocking {
        lines.push(format!("latest_voting_blocking: {latest_voting_blocking}"));
    }
    if let Some(latest_voting_next_action) = &view.latest_voting_next_action {
        lines.push(format!("latest_voting_next_action: {latest_voting_next_action}"));
    }

    if let Some(governance_next_action) = &view.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }
    if let Some(governance_lifecycle_runtime) = &view.governance_lifecycle_runtime {
        lines.push(format!("governance_lifecycle_runtime: {governance_lifecycle_runtime}"));
    }
    if let Some(governance_lifecycle_opt_out) = view.governance_lifecycle_opt_out {
        lines.push(format!("governance_lifecycle_opt_out: {governance_lifecycle_opt_out}"));
    }
    if let Some(governance_lifecycle_mode_selection) = &view.governance_lifecycle_mode_selection {
        lines.push(format!(
            "governance_lifecycle_mode_selection: {governance_lifecycle_mode_selection}"
        ));
    }
    if let Some(governance_lifecycle_selected_mode) = &view.governance_lifecycle_selected_mode {
        lines.push(format!(
            "governance_lifecycle_selected_mode: {governance_lifecycle_selected_mode}"
        ));
    }
    if let Some(reasoning_profile) = &view.latest_reasoning_profile {
        append_reasoning_profile_lines(&mut lines, "latest_", reasoning_profile);
    }

    let follow_through = FollowThroughProjection::from_session_view(view);
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }

    let s7_projection = s7_projection_for_session_status(view);
    lines.extend(s7_projection_lines(&s7_projection));
    lines.extend(s7_cognitive_projection_lines(&s7_cognitive_projection_for_session_status(
        view,
        &s7_projection.fallback_disclosure,
    )));

    if let Some(next_command) = view.next_command.as_ref().or(view.workflow_next_action.as_ref()) {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.push(format!("explanation: {}", view.explanation));
    lines.join("\n")
}

fn append_reasoning_profile_lines(
    lines: &mut Vec<String>,
    label_prefix: &str,
    reasoning_profile: &ProfileActivationRecord,
) {
    lines.push(format!("{label_prefix}reasoning_profile_id: {}", reasoning_profile.profile_id));
    lines.push(format!("{label_prefix}reasoning_profile_stage: {}", reasoning_profile.stage_key));
    lines.push(format!(
        "{label_prefix}reasoning_profile_status: {}",
        reasoning_profile.status.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_trigger: {}",
        reasoning_profile.trigger.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_reason: {}",
        reasoning_profile.activation_reason
    ));
    lines.push(format!(
        "{label_prefix}reasoning_budget: participants={} branches={} calls={} adjudication_steps={}",
        reasoning_profile.budget.max_participants,
        reasoning_profile.budget.max_branches,
        reasoning_profile.budget.max_calls,
        reasoning_profile.budget.max_adjudication_steps,
    ));
    if !reasoning_profile.participants.is_empty() {
        lines.push(format!(
            "{label_prefix}reasoning_participants: {}",
            reasoning_profile
                .participants
                .iter()
                .map(|participant| format!(
                    "{}={}",
                    participant.role_id, participant.effective_route
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(independence) = &reasoning_profile.independence {
        lines.push(format!(
            "{label_prefix}reasoning_independence_result: {}",
            independence.result.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_independence_reason: {}", independence.reason));
    }
    if let Some(posture) = &reasoning_profile.posture {
        lines.push(format!("{label_prefix}reasoning_posture_contract: {}", posture.contract_line));
        lines.push(format!(
            "{label_prefix}reasoning_posture_admission_priority: {}",
            posture.admission_priority.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_confidence_handoff: {}",
            posture.confidence_handoff_required
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_provenance_ref: {}",
            posture.provenance_ref
        ));
    }
    if let Some(outcome) = &reasoning_profile.outcome {
        lines.push(format!("{label_prefix}reasoning_outcome: {}", outcome.outcome_kind.as_str()));
        lines.push(format!("{label_prefix}reasoning_outcome_headline: {}", outcome.headline));
        if let Some(disagreement_summary) = &outcome.disagreement_summary {
            lines.push(format!(
                "{label_prefix}reasoning_disagreement_summary: {disagreement_summary}"
            ));
        }
        if let Some(next_action) = &outcome.next_action {
            lines.push(format!("{label_prefix}reasoning_next_action: {next_action}"));
        }
    }
    if let Some(confidence) = &reasoning_profile.confidence {
        lines.push(format!(
            "{label_prefix}reasoning_confidence_level: {}",
            confidence.confidence_level.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_confidence_effect: {}",
            confidence.admission_effect.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_confidence_summary: {}", confidence.summary));
    }
}

/// Renders the latest compatibility follow-up when no native session state is authoritative.
pub fn render_compatibility_follow_up_status(
    workspace_ref: &str,
    continuity_authority: ContinuityAuthority,
    follow_up: &CompatibilityFollowUpView,
    explanation: impl Into<String>,
) -> String {
    let mut lines = vec![format!("workspace_ref: {workspace_ref}")];
    lines.push("route_owner: compatibility".to_string());
    lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    lines.extend(render_compatibility_follow_up_lines(
        follow_up,
        "routing",
        "execution_condition",
        "next_command",
    ));
    let follow_through = FollowThroughProjection::from_compatibility_follow_up(follow_up);
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }
    lines.push(format!("explanation: {}", explanation.into()));
    lines.join("\n")
}

/// Renders a session-command failure with an optional suggested next command.
pub fn render_session_error(action: &str, message: &str, next_command: Option<&str>) -> String {
    let mut lines = vec![format!("{action}: session error"), format!("reason: {message}")];

    if let Some(next_command) = next_command {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.join("\n")
}

/// Converts the flattened guidance and guardian projection into compact summary
/// lines for status and inspect output.
pub fn render_guidance_projection_lines(
    guidance_guardian: &GuidanceGuardianProjection,
) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(summary) = &guidance_guardian.capability_resolution_summary {
        lines.push(format!("guidance_resolution_summary: {summary}"));
    }
    if !guidance_guardian.loaded_packs.is_empty() {
        lines.push(format!("loaded_packs: {}", guidance_guardian.loaded_packs.join(", ")));
    }
    if !guidance_guardian.skipped_packs.is_empty() {
        lines.push(format!("skipped_packs: {}", guidance_guardian.skipped_packs.join(" | ")));
    }
    if !guidance_guardian.catalog_validation_findings.is_empty() {
        lines.push(format!(
            "catalog_validation_findings: {}",
            guidance_guardian.catalog_validation_findings.join(" | ")
        ));
    }
    if !guidance_guardian.loaded_guidance_sources.is_empty() {
        lines.push(format!(
            "loaded_guidance_sources: {}",
            guidance_guardian.loaded_guidance_sources.join(", ")
        ));
    }
    if !guidance_guardian.skipped_guidance_sources.is_empty() {
        lines.push(format!(
            "skipped_guidance_sources: {}",
            guidance_guardian.skipped_guidance_sources.join(", ")
        ));
    }
    if !guidance_guardian.loaded_guardian_sources.is_empty() {
        lines.push(format!(
            "loaded_guardian_sources: {}",
            guidance_guardian.loaded_guardian_sources.join(", ")
        ));
    }
    if !guidance_guardian.skipped_guardian_sources.is_empty() {
        lines.push(format!(
            "skipped_guardian_sources: {}",
            guidance_guardian.skipped_guardian_sources.join(", ")
        ));
    }
    if !guidance_guardian.guardian_timeline.is_empty() {
        lines.push(format!(
            "guardian_timeline: {}",
            guidance_guardian.guardian_timeline.join(" | ")
        ));
    }
    if let Some(summary) = &guidance_guardian.guardian_findings_summary {
        lines.push(format!("guardian_findings_summary: {summary}"));
    }
    if !guidance_guardian.guardian_findings.is_empty() {
        lines.push(format!(
            "guardian_findings: {}",
            guidance_guardian
                .guardian_findings
                .iter()
                .map(|finding| format!(
                    "{}:{}:{}",
                    finding.guardian_id,
                    finding.disposition.as_str(),
                    finding.summary
                ))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !guidance_guardian.guardian_degradations.is_empty() {
        lines.push(format!(
            "guardian_degradations: {}",
            guidance_guardian.guardian_degradations.join(" | ")
        ));
    }
    if let Some(outcome) = &guidance_guardian.guardian_blocking_outcome {
        lines.push(format!("guardian_blocking_outcome: {outcome}"));
    }

    lines
}

/// Returns the next recommended command after a `run` response.
pub const fn next_command_after_run(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Succeeded => "/boundline-status",
        TaskStatus::Planned
        | TaskStatus::Running
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "/boundline-next",
    }
}

fn adaptive_workspace_slice_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let slice = state.get("latest_workspace_slice")?;
    let targets = slice.get("selected_targets")?.as_array()?;
    let targets = targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

fn adaptive_attempt_lineage_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let lineage = state.get("latest_attempt_lineage")?;
    let current = lineage.get("current_attempt_id")?.as_str()?;
    let transition = lineage.get("transition_kind")?.as_str()?;
    let previous = lineage.get("previous_attempt_id").and_then(Value::as_str);
    previous.map_or_else(
        || Some(format!("{current} ({transition})")),
        |previous| Some(format!("{current} {transition} {previous}")),
    )
}

fn adaptive_candidate_family_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_candidate_family")?.as_str().map(str::to_string)
}

fn adaptive_selection_reason_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_selection_reason")?.as_str().map(str::to_string)
}

fn adaptive_rejected_candidates_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    let rejected = state.get("latest_rejected_candidates")?.as_array()?;
    let rejected = rejected.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if rejected.is_empty() { None } else { Some(rejected.join(" | ")) }
}

fn adaptive_exhaustion_reason_summary(state: &serde_json::Map<String, Value>) -> Option<String> {
    state.get("latest_exhaustion_reason")?.as_str().map(str::to_string)
}

/// Returns the next recommended command after an `inspect` response.
pub const fn next_command_after_inspect(_: TaskStatus) -> &'static str {
    "/boundline-next"
}

/// Returns the textual execution-condition label used by inspect output.
pub fn trace_execution_condition_text(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("{kind} - {reason}")
}

fn value_as_string_list(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
    })
}

const UNKNOWN_VALIDATION_EXIT_CODE: i64 = -1;

fn validation_line_from_event(payload: &Value) -> Option<String> {
    let validation =
        payload.get("output").and_then(|output| output.get("validation")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
        })?;
    let command = validation.get("command").and_then(Value::as_str).unwrap_or("validation");
    let succeeded = validation.get("succeeded").and_then(Value::as_bool).unwrap_or(false);
    let exit_code =
        validation.get("exit_code").and_then(Value::as_i64).unwrap_or(UNKNOWN_VALIDATION_EXIT_CODE);
    Some(format!(
        "validation: {} ({command}, exit_code={exit_code})",
        if succeeded { "passed" } else { "failed" }
    ))
}

fn review_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_event_line(payload),
        TraceEventType::ReviewVoteResolved => payload
            .get(KEY_SUMMARY)
            .and_then(Value::as_str)
            .map(|summary| format!("review_vote: {summary}"))
            .or_else(|| {
                payload.get(KEY_VOTE_RESOLUTION).map(|resolution| {
                    format!(
                        "review_vote: {}",
                        serde_json::to_string(resolution).unwrap_or_default()
                    )
                })
            }),
        TraceEventType::ReviewAdjudicated => {
            reviewer_event_line(payload).map(|line| format!("review_adjudication: {line}"))
        }
        TraceEventType::ReviewTerminalRecorded => payload
            .get(KEY_REVIEW_OUTCOME)
            .and_then(Value::as_str)
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get(KEY_FAILURE_REASON)
                    .and_then(Value::as_str)
                    .map(|reason| format!("review_reason: {reason}"))
            }),
        _ => None,
    }
}

fn governance_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::GovernanceSelected => Some(format!(
            "governance_selected: {} -> {}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("selected_runtime").and_then(Value::as_str).unwrap_or("unknown-runtime")
        )),
        TraceEventType::GovernanceStarted => Some(format!(
            "governance_started: {}{}{}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload
                .get("canon_mode")
                .and_then(Value::as_str)
                .map(|mode| format!(" ({mode})"))
                .unwrap_or_default(),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceDecisionRecorded => payload
            .get("selected_action")
            .and_then(Value::as_str)
            .map(|action| format!("governance_decision: {action}"))
            .or_else(|| {
                payload
                    .get("blocked_reason")
                    .and_then(Value::as_str)
                    .map(|reason| format!("governance_decision_blocked: {reason}"))
            }),
        TraceEventType::GovernanceAwaitingApproval => Some(format!(
            "governance_awaiting_approval: {} ({}){}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(Value::as_str).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}{}",
            payload.get("headline").and_then(Value::as_str).unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(Value::as_str)
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("blocked"),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("packet rejected"),
            governance_packet_provenance_suffix(payload)
        )),
        _ => None,
    }
}

pub(crate) fn governance_packet_provenance_suffix(payload: &Value) -> String {
    governance_packet_provenance_text(
        payload.get("packet_source_stage").and_then(Value::as_str),
        payload.get("packet_binding_reason").and_then(Value::as_str),
    )
    .map(|provenance| format!(" from {provenance}"))
    .unwrap_or_default()
}

fn reviewer_event_line(payload: &Value) -> Option<String> {
    let reviewer_id = payload.get(KEY_REVIEWER_ID).and_then(Value::as_str)?;

    if let Some(finding) = payload.get(KEY_FINDING) {
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        let role = payload.get(KEY_REVIEWER_ROLE).and_then(Value::as_str);
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get(KEY_FAILURE_REASON)
        .and_then(Value::as_str)
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}

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

fn render_compatibility_follow_up_lines(
    follow_up: &CompatibilityFollowUpView,
    routing_label: &str,
    execution_condition_label: &str,
    next_command_label: &str,
) -> Vec<String> {
    let routing_summary =
        follow_up.routing_summary.strip_prefix("routing: ").unwrap_or(&follow_up.routing_summary);

    vec![
        format!("compatibility_follow_up: {}", follow_up.follow_up_mode.as_str()),
        format!("compatibility_trace_ref: {}", follow_up.trace_ref),
        format!("{routing_label}: {routing_summary}"),
        format!("{execution_condition_label}: {}", follow_up.execution_condition),
        format!("compatibility_terminal_status: {}", task_status_text(follow_up.terminal_status)),
        format!("compatibility_terminal_reason: {}", follow_up.terminal_reason),
        format!("{next_command_label}: {}", follow_up.next_command),
    ]
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
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn routing_outcome_for_status_view(view: &SessionStatusView) -> RoutingOutcome {
    match view.execution_path.as_deref() {
        Some("native_goal_plan") => RoutingOutcome {
            mode: RoutingMode::Native,
            source: RoutingSource::GoalPlan,
            reason: "goal plan is ready for native execution".to_string(),
        },
        Some("fixture_compatibility") => RoutingOutcome {
            mode: RoutingMode::Compatibility,
            source: RoutingSource::ExecutionProfile,
            reason: "compatibility execution remains active from the persisted task".to_string(),
        },
        Some("native_goal_plan_pending_plan_confirmation") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalPlan,
            reason: "plan confirmation is still pending before native execution".to_string(),
        },
        Some("native_session_pending_plan") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalCapture,
            reason: "goal captured but a goal plan is not ready yet".to_string(),
        },
        _ => match view.latest_status {
            SessionStatus::Initialized => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "capture a goal before planning or execution can begin".to_string(),
            },
            SessionStatus::GoalCaptured => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::GoalCapture,
                reason: "goal captured but a goal plan is not ready yet".to_string(),
            },
            SessionStatus::Invalid => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "active session state is invalid and must be recreated".to_string(),
            },
            _ => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "session has no goal plan or compatibility task to route".to_string(),
            },
        },
    }
}

fn render_route_config_projection(projection: Vec<String>) -> Option<String> {
    (!projection.is_empty()).then(|| format!("route_config_projection: {}", projection.join(" | ")))
}

fn render_cluster_story_lines(story: &ClusterDeliveryStory) -> Vec<String> {
    let mut lines = vec![
        format!("cluster_id: {}", story.cluster_id),
        format!("cluster_route_owner: {}", cluster_route_owner_text(story)),
        format!("cluster_authoritative_workspace: {}", story.authoritative_workspace_ref),
        format!(
            "cluster_execution_condition: {} - {}",
            cluster_execution_kind_text(story.execution_condition.kind),
            story.execution_condition.summary
        ),
    ];

    if let Some(blocking_workspace_ref) = &story.execution_condition.blocking_workspace_ref {
        lines.push(format!("cluster_blocking_workspace: {blocking_workspace_ref}"));
    }

    if !story.participating_workspaces.is_empty() {
        lines.push(format!(
            "cluster_participating_workspaces: {}",
            story
                .participating_workspaces
                .iter()
                .map(|record| format!(
                    "{} [{}]",
                    record.workspace_ref,
                    participation_kind_text(record.participation_kind)
                ))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    lines
}

fn cluster_execution_kind_text(kind: ClusteredExecutionKind) -> &'static str {
    match kind {
        ClusteredExecutionKind::Success => "success",
        ClusteredExecutionKind::Paused => "paused",
        ClusteredExecutionKind::Blocked => "blocked",
        ClusteredExecutionKind::Failed => "failed",
        ClusteredExecutionKind::Exhausted => "exhausted",
        ClusteredExecutionKind::InspectOnly => "inspect_only",
    }
}

fn participation_kind_text(kind: WorkspaceParticipationKind) -> &'static str {
    match kind {
        WorkspaceParticipationKind::Entry => "entry",
        WorkspaceParticipationKind::ReadOnly => "read_only",
        WorkspaceParticipationKind::Mutated => "mutated",
        WorkspaceParticipationKind::Blocked => "blocked",
        WorkspaceParticipationKind::Skipped => "skipped",
    }
}

fn cluster_route_owner_text(story: &ClusterDeliveryStory) -> &'static str {
    match story.route_owner {
        crate::domain::cluster::ClusterRouteOwner::Native => "native",
        crate::domain::cluster::ClusterRouteOwner::Workflow => "workflow",
        crate::domain::cluster::ClusterRouteOwner::Review => "review",
        crate::domain::cluster::ClusterRouteOwner::Governance => "governance",
        crate::domain::cluster::ClusterRouteOwner::Compatibility => "compatibility",
    }
}

fn session_route_owner(view: &SessionStatusView) -> &'static str {
    if view.latest_governance_state.is_some() || view.latest_governance_stage.is_some() {
        return "governance";
    }

    if view.latest_review_trigger.is_some()
        || view.latest_review_vote.is_some()
        || view.latest_review_outcome.is_some()
        || view.latest_review_council_profile.is_some()
        || view.latest_review_independence_state.is_some()
        || view.latest_review_stop_semantics.is_some()
        || view.latest_review_selection_summary.is_some()
        || view.latest_review_headline.is_some()
    {
        return "review";
    }

    if view.active_workflow.is_some() {
        return "workflow";
    }

    if matches!(view.continuity_authority, Some(ContinuityAuthority::CompatibilityTrace))
        || matches!(view.execution_path.as_deref(), Some("fixture_compatibility"))
    {
        return "compatibility";
    }

    "native"
}

fn trace_route_owner(summary: &TraceSummaryView) -> &'static str {
    if !summary.governance_timeline.is_empty() {
        return "governance";
    }

    if !summary.review_timeline.is_empty() {
        return "review";
    }

    if summary
        .routing_summary
        .as_deref()
        .is_some_and(|routing| routing.starts_with("routing: compatibility"))
    {
        return "compatibility";
    }

    "native"
}

fn route_config_projection_for_status_view(view: &SessionStatusView) -> Vec<String> {
    let mut projection = current_routing_projection(Path::new(&view.workspace_ref));

    if let Some(active_workflow) = &view.active_workflow {
        projection.push(format!("workflow={active_workflow}"));
    }

    if let Some(workflow_phase) = &view.workflow_phase {
        projection.push(format!("workflow_phase={workflow_phase}"));
    }

    if let Some(flow_state) = &view.flow_state {
        projection.push(format!("flow_state={flow_state}"));
    }

    if let Some(requested_governance_runtime) = &view.requested_governance_runtime {
        projection.push(format!("requested_governance_runtime={requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &view.requested_governance_risk {
        projection.push(format!("requested_governance_risk={requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &view.requested_governance_zone {
        projection.push(format!("requested_governance_zone={requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &view.requested_governance_owner {
        projection.push(format!("requested_governance_owner={requested_governance_owner}"));
    }

    projection
}

fn route_config_projection_for_trace_summary(summary: &TraceSummaryView) -> Vec<String> {
    let mut projection = summary.routing_projection.projection_lines();

    if projection.is_empty()
        && let Some(workspace) = workspace_from_trace_ref(Path::new(&summary.trace_ref))
    {
        projection.extend(current_routing_projection(&workspace));
    }

    if let Some(requested_governance_runtime) = &summary.requested_governance_runtime {
        projection.push(format!("requested_governance_runtime={requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &summary.requested_governance_risk {
        projection.push(format!("requested_governance_risk={requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &summary.requested_governance_zone {
        projection.push(format!("requested_governance_zone={requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &summary.requested_governance_owner {
        projection.push(format!("requested_governance_owner={requested_governance_owner}"));
    }

    projection
}

fn route_config_projection_for_run_trace(trace: &ExecutionTrace, trace_ref: &Path) -> Vec<String> {
    let mut projection = trace_routing_projection(trace);

    if projection.is_empty()
        && let Some(workspace) = workspace_from_trace_ref(trace_ref)
    {
        projection.extend(current_routing_projection(&workspace));
    }

    if let Some(input) = trace.events.iter().find_map(|event| {
        (event.event_type == TraceEventType::TaskStarted)
            .then(|| event.payload.get("input"))
            .flatten()
    }) {
        if let Some(requested_governance_runtime) =
            input.get("requested_governance_runtime").and_then(Value::as_str)
        {
            projection.push(format!("requested_governance_runtime={requested_governance_runtime}"));
        }
        if let Some(requested_governance_risk) =
            input.get("requested_governance_risk").and_then(Value::as_str)
        {
            projection.push(format!("requested_governance_risk={requested_governance_risk}"));
        }
        if let Some(requested_governance_zone) =
            input.get("requested_governance_zone").and_then(Value::as_str)
        {
            projection.push(format!("requested_governance_zone={requested_governance_zone}"));
        }
        if let Some(requested_governance_owner) =
            input.get("requested_governance_owner").and_then(Value::as_str)
        {
            projection.push(format!("requested_governance_owner={requested_governance_owner}"));
        }
    }

    projection
}

fn trace_routing_projection(trace: &ExecutionTrace) -> Vec<String> {
    trace
        .events
        .iter()
        .find_map(|event| RoutingDecisionProjection::from_event_payload(&event.payload))
        .map(|projection| projection.projection_lines())
        .unwrap_or_default()
}

fn run_trace_route_owner(trace: &ExecutionTrace) -> &'static str {
    let mut saw_native_routing_signal = false;
    let mut saw_review_signal = false;
    let mut saw_governance_signal = false;

    for event in &trace.events {
        if event.event_type.is_decision_loop_event() {
            saw_native_routing_signal = true;
        }

        match event.event_type {
            TraceEventType::GovernanceSelected
            | TraceEventType::GovernanceStarted
            | TraceEventType::GovernanceDecisionRecorded
            | TraceEventType::GovernanceAwaitingApproval
            | TraceEventType::GovernanceCompleted
            | TraceEventType::GovernanceBlocked
            | TraceEventType::GovernancePacketRejected => saw_governance_signal = true,
            TraceEventType::ReviewStarted
            | TraceEventType::ReviewTriggerIgnored
            | TraceEventType::ReviewerCompleted
            | TraceEventType::ReviewVoteResolved
            | TraceEventType::ReviewAdjudicated
            | TraceEventType::ReviewTerminalRecorded => saw_review_signal = true,
            _ => {}
        }
    }

    if saw_governance_signal {
        "governance"
    } else if saw_review_signal {
        "review"
    } else if saw_native_routing_signal {
        "native"
    } else {
        "compatibility"
    }
}

fn workspace_from_trace_ref(trace_ref: &Path) -> Option<std::path::PathBuf> {
    let traces_dir = trace_ref.parent()?;
    let boundline_dir = traces_dir.parent()?;
    if traces_dir.file_name()? != "traces" || boundline_dir.file_name()? != ".boundline" {
        return None;
    }

    boundline_dir.parent().map(Path::to_path_buf)
}

fn workspace_routing_projection(workspace: &Path) -> Option<String> {
    let routing = FileConfigStore::for_workspace(workspace).local_routing().ok().flatten()?;
    summarize_routing_config("workspace_routing", &routing)
}

fn current_routing_projection(workspace: &Path) -> Vec<String> {
    let workspace_routing =
        FileConfigStore::for_workspace(workspace).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();

    let mut projection = workspace_routing_projection(workspace).into_iter().collect::<Vec<_>>();

    let effective = resolve_effective_routing(
        &RoutingOverrides::default(),
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    let effective_capabilities = resolve_effective_runtime_capabilities(
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    let effective_effort = resolve_effective_slot_effort_policies(
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    projection.extend(
        RoutingDecisionProjection::from_effective_state(
            &effective,
            &effective_capabilities,
            &effective_effort,
        )
        .projection_lines(),
    );

    projection
}

fn summarize_routing_config(label: &str, routing: &RoutingConfig) -> Option<String> {
    let mut configured_routes = Vec::new();

    if let Some(route) = routing.planning.as_ref() {
        configured_routes.push(format!("planning={}", format_model_route(route)));
    }
    if let Some(route) = routing.implementation.as_ref() {
        configured_routes.push(format!("implementation={}", format_model_route(route)));
    }
    if let Some(route) = routing.verification.as_ref() {
        configured_routes.push(format!("verification={}", format_model_route(route)));
    }
    if let Some(route) = routing.review.as_ref() {
        configured_routes.push(format!("review={}", format_model_route(route)));
    }
    if let Some(route) = routing.adjudication.as_ref() {
        configured_routes.push(format!("adjudication={}", format_model_route(route)));
    }

    if configured_routes.is_empty() {
        None
    } else {
        Some(format!("{label}: {}", configured_routes.join(", ")))
    }
}

fn format_model_route(route: &ModelRoute) -> String {
    format!("{}/{}", route.runtime.as_str(), route.model)
}

fn render_session_execution_condition(view: &SessionStatusView) -> String {
    let (kind, reason) = session_execution_condition_parts(view);
    format!("execution_condition: {kind} - {reason}")
}

fn session_execution_condition_parts(view: &SessionStatusView) -> (&'static str, String) {
    if let Some(governance_state) = view.latest_governance_state.as_deref() {
        match governance_state {
            "awaiting_approval" => {
                return (
                    "waiting",
                    "governance approval is still pending before execution can continue"
                        .to_string(),
                );
            }
            "blocked" => {
                return (
                    "blocked",
                    view.latest_governance_blocked_reason.clone().unwrap_or_else(|| {
                        "governance blocked further execution until the blocker is resolved"
                            .to_string()
                    }),
                );
            }
            _ => {}
        }
    }

    if let Some(reason) =
        view.latest_reasoning_profile.as_ref().and_then(reasoning_execution_block_reason)
    {
        return ("blocked", reason);
    }

    if let Some(delegation) = &view.delegation {
        return match delegation.mode {
            crate::domain::session::DelegationContinuityMode::HandoffRequired
            | crate::domain::session::DelegationContinuityMode::EscalationRequired
            | crate::domain::session::DelegationContinuityMode::Stuck => {
                ("blocked", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Resolved => {
                ("waiting", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Exhausted
            | crate::domain::session::DelegationContinuityMode::InspectOnly => {
                ("inspect_only", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::None => {
                ("waiting", delegation.headline.clone())
            }
        };
    }

    if let Some(workflow_phase) = view.workflow_phase.as_deref() {
        match workflow_phase {
            "capture" if view.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() => {
                return (
                    "waiting",
                    "workflow is waiting for a captured goal before it can continue".to_string(),
                );
            }
            "clarify"
                if view.clarification_headline.is_some()
                    || view.clarification_prompt.is_some()
                    || view
                        .clarification_missing_fields
                        .as_ref()
                        .is_some_and(|fields| !fields.is_empty()) =>
            {
                return (
                    "waiting",
                    "clarification is still required before workflow planning can continue"
                        .to_string(),
                );
            }
            "review" => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                if view.latest_review_trigger.is_some() && view.latest_review_outcome.is_none() {
                    return (
                        "waiting",
                        "review outcome is still pending before workflow can continue".to_string(),
                    );
                }

                return (
                    "blocked",
                    "workflow review phase requires review evidence from the active session"
                        .to_string(),
                );
            }
            "govern" if view.latest_governance_state.is_none() => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                return (
                    "blocked",
                    "workflow govern phase requires governance evidence from the active session"
                        .to_string(),
                );
            }
            "govern"
                if matches!(
                    view.latest_governance_state.as_deref(),
                    Some("governed_ready" | "completed")
                ) && !view.latest_status.is_terminal() =>
            {
                return ("waiting", "governance is ready and workflow can resume".to_string());
            }
            _ => {}
        }
    }

    match view.execution_path.as_deref() {
        Some("native_goal_plan_pending_plan_confirmation") => {
            return (
                "blocked",
                "plan confirmation is still pending before native execution".to_string(),
            );
        }
        Some("native_session_pending_plan") => {
            return ("blocked", "goal captured but a goal plan is not ready yet".to_string());
        }
        _ => {}
    }

    match view.latest_status {
        SessionStatus::Initialized => {
            ("blocked", "capture a goal before planning or execution can begin".to_string())
        }
        SessionStatus::GoalCaptured => {
            ("blocked", "goal captured but a goal plan is not ready yet".to_string())
        }
        SessionStatus::Planned => (
            "waiting",
            if view.current_step_id.is_some() {
                "a bounded task is ready for the next execution step".to_string()
            } else {
                "planning is complete and execution can begin".to_string()
            },
        ),
        SessionStatus::Running => ("running", running_condition_reason(view).to_string()),
        SessionStatus::Succeeded => ("terminal", "work completed successfully".to_string()),
        SessionStatus::Failed => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "work stopped after a non-success result".to_string())
        }
        SessionStatus::Exhausted => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "retry or recovery limits were exhausted".to_string())
        }
        SessionStatus::Aborted => ("terminal", "work was aborted before completion".to_string()),
        SessionStatus::Invalid => {
            ("blocked", "active session state is invalid and must be recreated".to_string())
        }
    }
}

fn running_condition_reason(view: &SessionStatusView) -> &'static str {
    match view.latest_decision_status.as_deref() {
        Some("pending") => "a bounded decision is pending dispatch",
        Some("dispatched") => "the latest bounded decision is in flight",
        Some("verified") => "the latest bounded decision was verified and more work may remain",
        Some("failed") => "the latest bounded decision failed and recovery is in progress",
        Some("recovered") => "the latest bounded decision recovered and execution can continue",
        _ if view.latest_review_trigger.is_some() => {
            "review is in progress as part of the active session"
        }
        _ => "bounded execution is in progress",
    }
}

fn render_trace_execution_condition(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("execution_condition: {kind} - {reason}")
}

fn trace_execution_condition_parts(summary: &TraceSummaryView) -> (&'static str, String) {
    if let Some(reason) =
        summary.reasoning_profile.as_ref().and_then(reasoning_execution_block_reason)
    {
        return ("blocked", reason);
    }

    if let Some(delegation) = &summary.delegation {
        return match delegation.mode {
            crate::domain::session::DelegationContinuityMode::HandoffRequired
            | crate::domain::session::DelegationContinuityMode::EscalationRequired
            | crate::domain::session::DelegationContinuityMode::Stuck => {
                ("blocked", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Resolved => {
                ("waiting", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Exhausted
            | crate::domain::session::DelegationContinuityMode::InspectOnly => {
                ("inspect_only", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::None => {
                ("waiting", delegation.headline.clone())
            }
        };
    }

    let governance_waiting =
        summary.governance_timeline.iter().any(|line| line.contains("awaiting_approval"));
    let governance_blocked = summary.governance_timeline.iter().any(|line| {
        line.contains("governance_blocked") || line.contains("governance_packet_rejected")
    });

    if governance_waiting {
        return (
            "waiting",
            "governance approval is still pending before execution can continue".to_string(),
        );
    }

    if governance_blocked {
        return (
            "blocked",
            summary.governance_next_action.clone().unwrap_or_else(|| {
                "governance blocked further execution until the blocker is resolved".to_string()
            }),
        );
    }

    match summary.terminal_status {
        TaskStatus::Failed | TaskStatus::Exhausted => {
            if let Some(reason) = trace_adaptive_exhaustion_reason(summary) {
                ("terminal", reason)
            } else {
                ("terminal", summary.terminal_reason.message.clone())
            }
        }
        TaskStatus::Planned => ("waiting", summary.terminal_reason.message.clone()),
        TaskStatus::Running => ("running", summary.terminal_reason.message.clone()),
        TaskStatus::Succeeded | TaskStatus::Aborted => {
            ("terminal", summary.terminal_reason.message.clone())
        }
    }
}

fn reasoning_execution_block_reason(reasoning_profile: &ProfileActivationRecord) -> Option<String> {
    if !reasoning_profile.status.halts_outer_workflow() {
        return None;
    }

    let detail = reasoning_profile
        .outcome
        .as_ref()
        .and_then(|outcome| outcome.next_action.clone())
        .or_else(|| {
            reasoning_profile
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.disagreement_summary.clone())
        })
        .unwrap_or_else(|| reasoning_profile.activation_reason.clone());

    Some(format!(
        "reasoning profile {} blocked stage {}: {}",
        reasoning_profile.profile_id, reasoning_profile.stage_key, detail
    ))
}

fn trace_adaptive_exhaustion_reason(summary: &TraceSummaryView) -> Option<String> {
    summary
        .adaptive_evidence
        .iter()
        .find_map(|line| line.strip_prefix("adaptive_exhaustion: ").map(str::to_string))
}

fn render_run_execution_condition(response: &TaskRunResponse) -> String {
    let kind = match response.terminal_status {
        TaskStatus::Planned => "waiting",
        TaskStatus::Running => {
            let message = response.terminal_reason.message.to_ascii_lowercase();
            if message.contains("approval")
                || message.contains("wait")
                || message.contains("blocked")
            {
                "waiting"
            } else {
                "running"
            }
        }
        TaskStatus::Succeeded
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "terminal",
    };

    format!("execution_condition: {kind} - {}", response.terminal_reason.message)
}

fn cluster_member_state_text(state: ClusterMemberState) -> &'static str {
    match state {
        ClusterMemberState::Healthy => "healthy",
        ClusterMemberState::MissingSession => "missing-session",
        ClusterMemberState::MissingTrace => "missing-trace",
        ClusterMemberState::Blocked => "blocked",
        ClusterMemberState::Invalid => "invalid",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        command_name, governance_event_line, push_advanced_context_lines, render_diagnostics,
        render_host_command_json, render_run_execution_condition, render_run_trace,
        render_session_status, render_trace_summary, review_event_line, reviewer_event_line,
        session_execution_condition_parts, trace_execution_condition_parts,
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
            semantic_trace_records: vec![SemanticTraceRecord {
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
            }],
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
            (DeveloperCommand::Start { workspace: None, cluster: None }, "start"),
            (
                DeveloperCommand::Capture {
                    workspace: None,
                    cluster: None,
                    goal: Some("goal".to_string()),
                    brief: Vec::new(),
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                },
                "capture",
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
                    flow: None,
                    no_flow: false,
                    confirm: false,
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
                    command: CheckpointSubcommand::List { workspace: None, cluster: None },
                },
                "checkpoint",
            ),
            (DeveloperCommand::Inspect { trace: None, workspace: None, cluster: None }, "inspect"),
            (DeveloperCommand::Status { workspace: None, cluster: None }, "status"),
            (DeveloperCommand::Next { workspace: None, cluster: None }, "next"),
            (DeveloperCommand::Continue { workspace: None, cluster: None }, "continue"),
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
                    workspace: "/tmp/workspace".into(),
                    non_interactive: false,
                    template: None,
                    assistant: Vec::new(),
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
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: Vec::new(),
            context_provenance: Vec::new(),
            context_staleness_reason: None,
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
            review_timeline: Vec::new(),
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
            next_command: None,
            explanation: "session is invalid".to_string(),
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_status: invalid"), "{text}");
        assert!(!text.contains("latest_changed_files:"), "{text}");
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
                            "effective_route": "reviewer_roles.reviewer_primary:claude:sonnet-4.6",
                            "provider_family": "claude",
                            "context_basis": "governance_stage:bug-fix:investigate",
                            "prompting_pattern": "blind_reviewer",
                            "status": "pending",
                            "result_summary": null
                        },
                        {
                            "role_id": "reviewer_secondary",
                            "participant_id": "independent_pair_review-reviewer_secondary",
                            "effective_route": "review:codex:gpt-5-codex",
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
                            "boundline_min": "0.61.0",
                            "boundline_max_exclusive": "0.62.0",
                            "canon_min": "0.57.0",
                            "canon_max_exclusive": "0.58.0",
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
            next_command: Some("boundline step".to_string()),
            explanation: "review is in progress".to_string(),
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
                        effective_route: "reviewer_roles.reviewer_primary:claude:sonnet-4.6"
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
                        effective_route: "review:codex:gpt-5-codex".to_string(),
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
        assert!(text.contains("latest_reasoning_independence_result: failed"), "{text}");
        assert!(text.contains("latest_reasoning_confidence_level: low"), "{text}");
        assert!(
            text.contains("latest_reasoning_next_action: configure distinct reviewer routes"),
            "{text}"
        );
        assert!(text.contains("latest_reasoning_participants: reviewer_primary=reviewer_roles.reviewer_primary:claude:sonnet-4.6"), "{text}");
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
    }

    #[test]
    fn render_trace_summary_includes_review_timeline() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.boundline/traces/task-review-output.json".to_string(),
            goal: "Render trace summary".to_string(),
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
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: Vec::new(),
            context_provenance: Vec::new(),
            context_staleness_reason: None,
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
            review_timeline: vec![
                "review_trigger: pr_ready".to_string(),
                "reviewer safety (Safety) approve: No blockers".to_string(),
                "review_vote: strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"
                    .to_string(),
                "review_outcome: accepted".to_string(),
            ],
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
                        boundline_min: "0.61.0".to_string(),
                        boundline_max_exclusive: "0.62.0".to_string(),
                        canon_min: "0.57.0".to_string(),
                        canon_max_exclusive: "0.58.0".to_string(),
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
        assert!(text.contains("reasoning_independence_result: degraded"), "{text}");
        assert!(
            text.contains("reasoning_posture_contract: governed_reasoning_posture_v1"),
            "{text}"
        );
        assert!(text.contains("reasoning_confidence_level: medium"), "{text}");
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
    }
}
