//! Planning and execution brief rendering plus goal-text decomposition helpers
//! reused by session runtime and CLI surfaces.

use serde_json::Value;

use crate::domain::decision::DecisionStatus;

use super::{
    CanonMode, Decision, DecisionType, EXECUTION_BRIEF_MAX_DECISIONS, GoalPlan,
    LATEST_VALIDATION_STATUS_KEY, PLANNING_DEFAULT_TARGET, PLANNING_STAGE_AUTHORED_INPUTS_HEADING,
    PLANNING_STAGE_BRIEF_TITLE, PLANNING_STAGE_CANON_MEMORY_HEADING,
    PLANNING_STAGE_CONTEXT_HEADING, PLANNING_STAGE_OUTPUT_LANGUAGE_HEADING,
    PLANNING_STAGE_OUTPUT_LANGUAGE_INSTRUCTION, PLANNING_STAGE_OVERVIEW_HEADING,
    PLANNING_STAGE_WORKFLOW_HEADING, PLANNING_UNSPECIFIED_FLOW, PlanningContextSources,
    StageCouncilFinding, StageCouncilRequest, TaskContext, execution_governance_read_targets,
};

pub(super) fn render_planning_stage_brief(
    stage_key: &str,
    mode: CanonMode,
    goal_plan: &GoalPlan,
    context_sources: &PlanningContextSources,
) -> String {
    let flow_name = goal_plan
        .flow
        .as_ref()
        .map(|flow| flow.flow_name.as_str())
        .unwrap_or(PLANNING_UNSPECIFIED_FLOW);
    let target_summary = goal_plan
        .context_pack
        .as_ref()
        .filter(|context_pack| !context_pack.selected_targets.is_empty())
        .map(|context_pack| context_pack.selected_targets.join(", "))
        .unwrap_or_else(|| PLANNING_DEFAULT_TARGET.to_string());
    let context_summary = goal_plan
        .context_summary()
        .unwrap_or_else(|| "no bounded context summary recorded".to_string());
    let primary_inputs = goal_plan.context_primary_inputs();
    let primary_inputs =
        if primary_inputs.is_empty() { "none".to_string() } else { primary_inputs.join(", ") };
    let authored_inputs = if context_sources.authored_input_sources.is_empty() {
        "none".to_string()
    } else {
        context_sources.authored_input_sources.join(", ")
    };

    let mut brief = format!(
        concat!(
            "{title}\n\n",
            "{output_lang_heading}\n",
            "- instruction: {output_lang_instruction}\n\n",
            "{overview}\n",
            "- stage_key: {stage_key}\n",
            "- canon_mode: {mode}\n",
            "- flow: {flow_name}\n",
            "- goal: {goal}\n",
            "- targets: {targets}\n\n",
            "{workflow}\n",
            "- planning_rationale: {planning_rationale}\n",
            "- verification_strategy: {verification_strategy}\n\n",
            "{context}\n",
            "- summary: {context_summary}\n",
            "- primary_inputs: {primary_inputs}\n\n",
            "{authored}\n",
            "- authored_input_summary: {authored_input_summary}\n",
            "- authored_input_sources: {authored_inputs}\n"
        ),
        title = PLANNING_STAGE_BRIEF_TITLE,
        output_lang_heading = PLANNING_STAGE_OUTPUT_LANGUAGE_HEADING,
        output_lang_instruction = PLANNING_STAGE_OUTPUT_LANGUAGE_INSTRUCTION,
        overview = PLANNING_STAGE_OVERVIEW_HEADING,
        workflow = PLANNING_STAGE_WORKFLOW_HEADING,
        context = PLANNING_STAGE_CONTEXT_HEADING,
        authored = PLANNING_STAGE_AUTHORED_INPUTS_HEADING,
        stage_key = stage_key,
        mode = mode.as_str(),
        flow_name = flow_name,
        goal = goal_plan.goal_text,
        targets = target_summary,
        planning_rationale = goal_plan.planning_rationale.as_deref().unwrap_or("none"),
        verification_strategy = goal_plan.verification_strategy.as_deref().unwrap_or("none"),
        context_summary = context_summary,
        primary_inputs = primary_inputs,
        authored_input_summary =
            context_sources.authored_input_summary.as_deref().unwrap_or("none"),
        authored_inputs = authored_inputs,
    );

    if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
        brief.push_str("\n\n");
        brief.push_str(PLANNING_STAGE_CANON_MEMORY_HEADING);
        brief.push('\n');
        brief.push_str("- summary: ");
        brief.push_str(&memory.summary_text());
        brief.push('\n');
        brief.push_str("- credibility: ");
        brief.push_str(memory.credibility.as_str());
        brief.push('\n');
    }

    brief.push_str("\n\n## Problem Domain\n");
    brief.push_str("- domain: ");
    brief.push_str(&planning_problem_domain(goal_plan));
    brief.push('\n');

    brief.push_str("\n## Known Facts\n");
    brief.push_str("- goal: ");
    brief.push_str(&goal_plan.goal_text);
    brief.push('\n');
    brief.push_str("- selected_targets: ");
    brief.push_str(&target_summary);
    brief.push('\n');
    brief.push_str("- primary_inputs: ");
    brief.push_str(&primary_inputs);
    brief.push('\n');
    brief.push_str("- authored_inputs: ");
    brief.push_str(&authored_inputs);
    brief.push('\n');

    if let Some(decomposition_section) = render_goal_decomposition_section(&goal_plan.goal_text) {
        brief.push_str(&decomposition_section);
    }

    brief.push_str("\n## Unknowns\n");
    for unknown in planning_unknown_markers(
        &goal_plan.goal_text,
        goal_plan.verification_strategy.as_deref(),
        !context_sources.authored_input_sources.is_empty(),
    ) {
        brief.push_str("- ");
        brief.push_str(&unknown);
        brief.push('\n');
    }

    brief.push_str("\n## Assumptions\n");
    for assumption in planning_assumptions(goal_plan) {
        brief.push_str("- ");
        brief.push_str(&assumption);
        brief.push('\n');
    }

    brief.push_str("\n## Validation Targets\n");
    brief.push_str("- strategy: ");
    brief.push_str(goal_plan.verification_strategy.as_deref().unwrap_or(
        "operator must provide validation command or acceptance evidence before execution",
    ));
    brief.push('\n');

    brief.push_str("\n## Confidence Levels\n");
    brief.push_str("- context_pack: ");
    brief.push_str(
        goal_plan
            .context_pack
            .as_ref()
            .map(|context_pack| context_pack.credibility.as_str())
            .unwrap_or("unavailable"),
    );
    brief.push('\n');
    brief.push_str("- authored_input: ");
    brief.push_str(if context_sources.authored_input_summary.is_some() {
        "operator_authored"
    } else {
        "not_provided"
    });
    brief.push('\n');

    brief.push_str("\n## Discovery Handoff\n");
    brief.push_str("- handoff: use known facts as bounded evidence, preserve unknowns as questions, and reject the packet if discovery cannot convert assumptions into actionable requirements.\n");

    brief
}

pub(super) fn render_execution_stage_brief(
    mode: CanonMode,
    goal_plan: &GoalPlan,
    decisions: &[Decision],
    native_context: &TaskContext,
    fallback_targets: &[String],
) -> String {
    let changed_files = execution_governance_read_targets(native_context, fallback_targets);
    let validation_status = native_context
        .state
        .get(LATEST_VALIDATION_STATUS_KEY)
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let mut brief = format!(
        concat!(
            "# Execution Governance Brief\n\n",
            "## Overview\n",
            "- canon_mode: {mode}\n",
            "- goal: {goal}\n",
            "- plan_revision: {plan_revision}\n"
        ),
        mode = mode.as_str(),
        goal = goal_plan.goal_text,
        plan_revision = goal_plan.proposal_revision,
    );

    brief.push_str("\n## Changed Files\n");
    if changed_files.is_empty() {
        brief.push_str("- no bounded file targets were recorded\n");
    } else {
        for changed_file in &changed_files {
            brief.push_str("- ");
            brief.push_str(changed_file);
            brief.push('\n');
        }
    }

    brief.push_str("\n## Validation\n");
    brief.push_str("- status: ");
    brief.push_str(validation_status);
    brief.push('\n');

    if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
        brief.push_str("\n## Canon Memory\n");
        brief.push_str("- summary: ");
        brief.push_str(&memory.summary_text());
        brief.push('\n');
        brief.push_str("- credibility: ");
        brief.push_str(memory.credibility.as_str());
        brief.push('\n');
    }

    brief.push_str("\n## Decision Summary\n");
    let mut rendered_any_decision = false;
    for decision in decisions
        .iter()
        .filter(|decision| decision.status.is_terminal())
        .take(EXECUTION_BRIEF_MAX_DECISIONS)
    {
        let decision_type = match decision.decision_type {
            DecisionType::Analyze => "analyze",
            DecisionType::Code => "code",
            DecisionType::Test => "test",
            DecisionType::Fix => "fix",
            DecisionType::Replan => "replan",
        };
        let decision_status = match decision.status {
            DecisionStatus::Pending => "pending",
            DecisionStatus::Dispatched => "dispatched",
            DecisionStatus::Verified => "verified",
            DecisionStatus::Failed => "failed",
            DecisionStatus::Recovered => "recovered",
        };
        brief.push_str("- ");
        brief.push_str(decision_type);
        brief.push_str(": ");
        brief.push_str(&decision.target);
        brief.push_str(" (status: ");
        brief.push_str(decision_status);
        brief.push_str(") -> ");
        brief.push_str(&decision.expected_outcome);
        brief.push('\n');
        rendered_any_decision = true;
    }

    if !rendered_any_decision {
        brief.push_str("- no terminal decisions were recorded\n");
    }

    brief
}

fn planning_problem_domain(goal_plan: &GoalPlan) -> String {
    let lower = goal_plan.goal_text.to_ascii_lowercase();
    if lower.contains("user") || lower.contains("oauth") || lower.contains("auth") {
        "user management and authentication".to_string()
    } else if lower.contains("api") || lower.contains("grpc") || lower.contains("service") {
        "service/API delivery".to_string()
    } else {
        "bounded delivery target from captured goal".to_string()
    }
}

/// Structured decomposition of a goal string into the semantic sections that
/// Canon governance templates expect as authored body content.
///
/// # Why this exists
///
/// Canon's requirements template generates multiple artifacts (prd.md,
/// tradeoffs.md, constraints.md, etc.) by reading structured sections from the
/// planning brief. When the brief contains only a flat goal string under
/// `## Known Facts`, Canon cannot locate a `## Problem`, `## Outcome`, or
/// `## Constraints` section and emits "NOT CAPTURED" placeholder stubs for
/// every missing section in every output artifact.
///
/// `decompose_goal_text` performs best-effort deterministic parsing of the
/// goal string to extract these sections. The decomposition is keyword-based
/// and does NOT invoke an external LLM; it splits on well-known structural
/// markers ("Persistence:", "Auth:", "Intended outcome:", entity/operation
/// patterns) to produce content Canon can reference as authored body.
///
/// # Degradation contract
///
/// If a section cannot be extracted, its field is `None` (or empty vec).
/// The brief renderer writes only the sections that have content, so an
/// empty decomposition produces output identical to the previous behavior.
/// This guarantees backward compatibility with goals that don't follow
/// recognizable patterns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GoalDecomposition {
    /// The core problem statement: what is being built and why.
    /// Extracted from text preceding structural markers like "Persistence:"
    /// or "Intended outcome:".
    pub problem: Option<String>,

    /// The desired deliverable outcome.
    /// Extracted from text following "Intended outcome:" or "Desired outcome:".
    pub outcome: Option<String>,

    /// Technical constraints binding the implementation.
    /// Each entry is a single constraint (e.g. "Persistence: in-memory",
    /// "Auth: OAuth2 JWT at service level").
    pub constraints: Vec<String>,

    /// Domain entities with their attributes.
    /// Extracted from "Users: first name, last name, ..." or similar patterns.
    pub entities: Vec<String>,

    /// API operations, endpoints, or RPC methods in scope.
    /// Extracted from comma-separated lists of operation names or CRUD
    /// expansion patterns.
    pub operations: Vec<String>,

    /// The validation strategy or acceptance criteria.
    /// Extracted from "Validation:" markers or test command references.
    pub validation: Option<String>,
}

impl GoalDecomposition {
    /// Returns `true` when the decomposition extracted at least one
    /// substantive section that Canon templates can consume.
    pub fn has_content(&self) -> bool {
        self.problem.is_some()
            || self.outcome.is_some()
            || !self.constraints.is_empty()
            || !self.entities.is_empty()
            || !self.operations.is_empty()
            || self.validation.is_some()
    }
}

/// Performs best-effort structured decomposition of a goal string.
///
/// Parses the goal text for recognizable patterns and extracts semantic
/// sections that align with Canon template expectations. The extraction is
/// deterministic and keyword-driven; no external LLM call is made.
///
/// # Recognized patterns
///
/// | Pattern | Extracted as |
/// |---------|-------------|
/// | Text before first structural marker | `problem` |
/// | "Intended outcome:" / "Desired outcome:" | `outcome` |
/// | "Persistence:" clause | constraint |
/// | "Auth:" / "OAuth2" clause | constraint |
/// | "edition YYYY" / framework mentions | constraint |
/// | "Users:" or entity-attribute lists | entity |
/// | Comma-separated PascalCase names | operations |
/// | "CRUD" keyword expansion | operations |
/// | "Validation:" / test command | `validation` |
///
/// # Examples
///
/// ```text
/// goal = "Rust microservice (edition 2024), Axum + gRPC, user management
///         service. Users: first name, last name, email, role (Admin | User).
///         Persistence: in-memory. Auth: OAuth2 JWT. gRPC operations:
///         CreateUser, GetUser, ListUsers, UpdateUser, DeleteUser.
///         Intended outcome: a complete Cargo workspace with unit tests.
///         Validation: shell script with curl/grpcurl smoke tests."
///
/// result.problem = Some("Rust microservice (edition 2024), Axum + gRPC, user management service")
/// result.outcome = Some("a complete Cargo workspace with unit tests")
/// result.constraints = ["Persistence: in-memory store with no external database...",
///                       "Auth: OAuth2 JWT validated at service level",
///                       "Rust edition 2024", "Axum HTTP framework", "gRPC RPC surface"]
/// result.entities = ["Users: first name, last name, email, role (Admin | User)"]
/// result.operations = ["CreateUser", "GetUser", "ListUsers", "UpdateUser", "DeleteUser"]
/// result.validation = Some("shell script with curl/grpcurl smoke tests against running server")
/// ```
pub fn decompose_goal_text(goal: &str) -> GoalDecomposition {
    let mut decomposition = GoalDecomposition::default();
    let goal_trimmed = goal.trim();
    if goal_trimmed.is_empty() {
        return decomposition;
    }

    let outcome_markers = ["intended outcome:", "desired outcome:"];
    let lower = goal_trimmed.to_ascii_lowercase();
    for marker in &outcome_markers {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let outcome_text =
                after.split_once('.').map(|(sentence, _)| sentence.trim()).unwrap_or(after.trim());
            if !outcome_text.is_empty() {
                decomposition.outcome = Some(outcome_text.to_string());
            }
            break;
        }
    }

    let validation_markers = ["validation:", "acceptance:"];
    for marker in &validation_markers {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let validation_text =
                after.split_once('.').map(|(sentence, _)| sentence.trim()).unwrap_or(after.trim());
            if !validation_text.is_empty() {
                decomposition.validation = Some(validation_text.to_string());
            }
            break;
        }
    }
    if decomposition.validation.is_none() {
        let test_commands = ["cargo test", "npm test", "pytest", "go test"];
        for cmd in &test_commands {
            if lower.contains(cmd) {
                decomposition.validation = Some(format!("{cmd} (detected from goal text)"));
                break;
            }
        }
    }

    if let Some(pos) = lower.find("persistence:") {
        let after = &goal_trimmed[pos + "persistence:".len()..];
        let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
        if !clause.is_empty() {
            decomposition.constraints.push(format!("Persistence: {clause}"));
        }
    }
    if let Some(pos) = lower.find("auth:") {
        let after = &goal_trimmed[pos + "auth:".len()..];
        let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
        if !clause.is_empty() {
            decomposition.constraints.push(format!("Auth: {clause}"));
        }
    }
    if lower.contains("edition 2024") || lower.contains("edition 2021") {
        let edition = if lower.contains("edition 2024") { "2024" } else { "2021" };
        decomposition.constraints.push(format!("Rust edition {edition}"));
    }
    if lower.contains("axum") {
        decomposition.constraints.push("Axum HTTP framework".to_string());
    }
    if lower.contains("grpc") {
        decomposition.constraints.push("gRPC RPC surface".to_string());
    }
    if lower.contains("actix") {
        decomposition.constraints.push("Actix-web HTTP framework".to_string());
    }
    if lower.contains("tonic") {
        decomposition.constraints.push("Tonic gRPC framework".to_string());
    }

    let entity_markers = ["users:", "user:", "entities:", "entity:"];
    for marker in &entity_markers {
        if let Some(pos) = lower.find(marker) {
            let capitalized_marker = &goal_trimmed[pos..pos + marker.len()];
            let after = &goal_trimmed[pos + marker.len()..];
            let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
            if !clause.is_empty() {
                decomposition.entities.push(format!("{capitalized_marker} {clause}"));
            }
        }
    }

    let operation_patterns =
        ["operations:", "operations in scope:", "rpcs:", "endpoints:", "methods:"];
    for marker in &operation_patterns {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
            for op in clause.split(',') {
                let op = op.trim();
                if !op.is_empty() && op.len() < 60 {
                    decomposition.operations.push(op.to_string());
                }
            }
            break;
        }
    }
    if decomposition.operations.is_empty() {
        let pascal_ops: Vec<&str> = goal_trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| {
                s.len() > 3
                    && s.len() < 40
                    && s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && s.chars().any(|c| c.is_ascii_lowercase())
                    && s.chars().all(|c| c.is_alphanumeric())
            })
            .collect();
        if pascal_ops.len() >= 3 {
            decomposition.operations = pascal_ops.into_iter().map(|s| s.to_string()).collect();
        }
    }

    let structural_markers = [
        "persistence:",
        "auth:",
        "intended outcome:",
        "desired outcome:",
        "validation:",
        "acceptance:",
    ];
    let first_marker_pos = structural_markers.iter().filter_map(|marker| lower.find(marker)).min();
    if let Some(pos) = first_marker_pos {
        let problem_text = goal_trimmed[..pos].trim().trim_end_matches('.');
        if !problem_text.is_empty() {
            decomposition.problem = Some(problem_text.to_string());
        }
    } else if decomposition.outcome.is_none() {
        let first_sentence =
            goal_trimmed.split_once('.').map(|(s, _)| s.trim()).unwrap_or(goal_trimmed);
        if !first_sentence.is_empty() {
            decomposition.problem = Some(first_sentence.to_string());
        }
    }

    decomposition
}

fn render_goal_decomposition_section(goal_text: &str) -> Option<String> {
    let decomposition = decompose_goal_text(goal_text);
    if !decomposition.has_content() {
        return None;
    }

    let mut section = String::from("\n## Structured Goal Decomposition\n");

    if let Some(problem) = &decomposition.problem {
        section.push_str("### Problem\n");
        section.push_str(problem);
        section.push_str("\n\n");
    }

    if let Some(outcome) = &decomposition.outcome {
        section.push_str("### Desired Outcome\n");
        section.push_str(outcome);
        section.push_str("\n\n");
    }

    if !decomposition.constraints.is_empty() {
        section.push_str("### Constraints\n");
        for constraint in &decomposition.constraints {
            section.push_str("- ");
            section.push_str(constraint);
            section.push('\n');
        }
        section.push('\n');
    }

    if !decomposition.entities.is_empty() {
        section.push_str("### Domain Entities\n");
        for entity in &decomposition.entities {
            section.push_str("- ");
            section.push_str(entity);
            section.push('\n');
        }
        section.push('\n');
    }

    if !decomposition.operations.is_empty() {
        section.push_str("### Operations In Scope\n");
        for operation in &decomposition.operations {
            section.push_str("- ");
            section.push_str(operation);
            section.push('\n');
        }
        section.push('\n');
    }

    if let Some(validation) = &decomposition.validation {
        section.push_str("### Validation Criteria\n");
        section.push_str(validation);
        section.push('\n');
    }

    Some(section)
}

pub(super) fn plain_goal_requires_planning_clarification(
    goal: &str,
    context_sources: &PlanningContextSources,
) -> bool {
    if !context_sources.authored_input_sources.is_empty()
        || !context_sources.execution_profile_read_targets.is_empty()
        || context_sources.latest_trace_ref.is_some()
        || !context_sources.latest_changed_files.is_empty()
        || context_sources.compacted_canon_memory.is_some()
    {
        return false;
    }

    let lower = goal.to_ascii_lowercase();
    let broad_delivery = lower.contains("build ")
        || lower.contains("deliver ")
        || lower.contains("capability")
        || lower.contains("microservice")
        || lower.contains("microservizio")
        || lower.contains("service")
        || lower.contains("api");
    let has_validation = lower.contains("cargo test")
        || lower.contains("validation")
        || lower.contains("acceptance")
        || lower.contains("verify");

    broad_delivery && !has_validation
}

pub(super) fn plain_goal_planning_clarification_prompt() -> String {
    "Answer these planning questions before Boundline can continue planning: What exact outcome should Boundline deliver? Which domain entities and relationships are in scope? Which API operations, endpoints, or RPC methods are in scope? What persistence and OAuth/security assumptions are binding? Which validation command or acceptance evidence should prove the slice?".to_string()
}

pub fn planning_unknown_markers(
    goal_text: &str,
    verification_strategy: Option<&str>,
    has_authored_inputs: bool,
) -> Vec<String> {
    let lower = goal_text.to_ascii_lowercase();
    let mut unknowns = Vec::new();
    if !lower.contains("validation")
        && !lower.contains("cargo test")
        && !lower.contains("acceptance")
        && verification_strategy.unwrap_or("none") == "none"
    {
        unknowns.push("validation_target requires operator confirmation".to_string());
    }
    if !lower.contains("database")
        && !lower.contains("postgres")
        && !lower.contains("sqlite")
        && !lower.contains("persist")
    {
        unknowns.push("persistence assumptions require operator confirmation".to_string());
    }
    if !has_authored_inputs {
        unknowns.push("authored source provenance is unavailable".to_string());
    }

    let decomposition = decompose_goal_text(goal_text);
    if decomposition.outcome.is_none() {
        unknowns.push("desired outcome could not be extracted from goal text and requires operator confirmation".to_string());
    }
    if decomposition.operations.is_empty() && lower.contains("service") {
        unknowns.push("API operations or endpoints in scope could not be identified and require operator specification".to_string());
    }
    if decomposition.entities.is_empty() && (lower.contains("user") || lower.contains("entity")) {
        unknowns.push(
            "domain entities and their attributes could not be parsed from goal text".to_string(),
        );
    }

    if unknowns.is_empty() {
        unknowns
            .push("no explicit unknown markers were detected from the captured brief".to_string());
    }
    unknowns
}

fn planning_assumptions(goal_plan: &GoalPlan) -> Vec<String> {
    let lower = goal_plan.goal_text.to_ascii_lowercase();
    let mut assumptions = Vec::new();
    if lower.contains("rust") {
        assumptions.push("language/runtime: Rust".to_string());
    }
    if lower.contains("axum") {
        assumptions.push("HTTP framework: Axum".to_string());
    }
    if lower.contains("grpc") {
        assumptions.push("RPC surface: gRPC".to_string());
    }
    if lower.contains("oauth") {
        assumptions.push("security: OAuth2 protected surface".to_string());
    }

    if assumptions.is_empty() {
        assumptions.push("no concrete technical assumptions were captured".to_string());
    }
    assumptions
}

pub(super) fn render_stage_council_blocked_markdown(
    request: &StageCouncilRequest,
    findings: &[StageCouncilFinding],
    accepted_findings: &[String],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Discovery Stage Council Blocked\n\n");
    markdown.push_str(&format!("- stage: {}\n", request.stage_key));
    markdown.push_str("- outcome: blocked\n");
    if findings.is_empty() {
        markdown.push_str("- findings: no provider-backed reviewer findings were recorded\n");
    } else {
        markdown.push_str("\n## Findings\n\n");
        for finding in findings {
            let accepted = if accepted_findings.contains(&finding.reviewer_id) {
                "accepted"
            } else {
                "rejected"
            };
            markdown.push_str(&format!(
                "- {} [{}] {}: {}\n",
                finding.reviewer_id, finding.effective_route, accepted, finding.summary
            ));
        }
    }
    markdown.push_str(
        "\nRepair the discovery inputs or reviewer routing, then rerun `boundline plan`.\n",
    );
    markdown
}

pub(super) fn render_stage_council_blocked_note(reason: &str) -> String {
    format!(
        "# Discovery Stage Council Blocked\n\n- reason: {reason}\n\nRerun `boundline plan` after restoring independent provider-backed council execution.\n"
    )
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use serde_json::{Map, Value, json};

    use crate::domain::decision::{Decision, DecisionStatus, DecisionType};
    use crate::domain::goal_plan::{
        ContextPack, ContextPackCredibility, GoalPlan, InferredFlow, PlannedTask,
    };
    use crate::domain::governance::{CanonMode, CompactedCanonMemory, MemoryCredibilityState};
    use crate::domain::limits::RunLimits;
    use crate::domain::stage_council::{
        StageCouncilFinding, StageCouncilFindingDisposition, StageCouncilRequest,
    };
    use crate::domain::task_context::TaskContext;
    use crate::orchestrator::goal_planner::PlanningContextSources;

    use super::{
        decompose_goal_text, plain_goal_planning_clarification_prompt,
        plain_goal_requires_planning_clarification, planning_assumptions, planning_problem_domain,
        planning_unknown_markers, render_execution_stage_brief, render_goal_decomposition_section,
        render_planning_stage_brief, render_stage_council_blocked_markdown,
        render_stage_council_blocked_note,
    };

    const RICH_GOAL: &str = "Rust microservice (edition 2024) with Axum and tonic gRPC service for user management. Users: first name, last name, email, role. Persistence: sqlite. Auth: OAuth2 JWT. Operations: CreateUser, GetUser, ListUsers. Intended outcome: a complete Cargo workspace with unit tests. Validation: cargo test.";
    const ACTIX_GOAL: &str = "Actix service for order intake. cargo test";
    const BROAD_GOAL: &str = "Build a user management microservice API";
    const DEFAULT_GOAL: &str = "Refine release checklist";
    const SESSION_ID: &str = "session-briefs";
    const WORKSPACE_REF: &str = "workspace-briefs";

    #[test]
    fn brief_helpers_cover_goal_decomposition_and_clarification_paths() {
        let decomposition = decompose_goal_text(RICH_GOAL);
        assert_eq!(
            decomposition.problem.as_deref(),
            Some(
                "Rust microservice (edition 2024) with Axum and tonic gRPC service for user management. Users: first name, last name, email, role"
            )
        );
        assert_eq!(
            decomposition.outcome.as_deref(),
            Some("a complete Cargo workspace with unit tests")
        );
        assert_eq!(decomposition.validation.as_deref(), Some("cargo test"));
        assert!(decomposition.constraints.contains(&"Persistence: sqlite".to_string()));
        assert!(decomposition.constraints.contains(&"Auth: OAuth2 JWT".to_string()));
        assert!(decomposition.constraints.contains(&"Rust edition 2024".to_string()));
        assert!(decomposition.constraints.contains(&"Axum HTTP framework".to_string()));
        assert!(decomposition.constraints.contains(&"gRPC RPC surface".to_string()));
        assert!(decomposition.constraints.contains(&"Tonic gRPC framework".to_string()));
        assert_eq!(
            decomposition.entities,
            vec!["Users: first name, last name, email, role".to_string()]
        );
        assert_eq!(
            decomposition.operations,
            vec!["CreateUser".to_string(), "GetUser".to_string(), "ListUsers".to_string(),]
        );

        let actix_decomposition = decompose_goal_text(ACTIX_GOAL);
        assert_eq!(
            actix_decomposition.validation.as_deref(),
            Some("cargo test (detected from goal text)")
        );
        assert!(actix_decomposition.constraints.contains(&"Actix-web HTTP framework".to_string()));

        assert!(render_goal_decomposition_section("").is_none());
        let rendered_section = render_goal_decomposition_section(RICH_GOAL);
        assert!(rendered_section.as_deref().is_some_and(|section| {
            section.contains("## Structured Goal Decomposition")
                && section.contains("### Constraints")
                && section.contains("### Domain Entities")
                && section.contains("### Operations In Scope")
                && section.contains("### Validation Criteria")
        }));

        assert!(plain_goal_requires_planning_clarification(
            BROAD_GOAL,
            &PlanningContextSources::default(),
        ));
        assert!(!plain_goal_requires_planning_clarification(
            BROAD_GOAL,
            &PlanningContextSources {
                authored_input_sources: vec!["brief.md".to_string()],
                ..PlanningContextSources::default()
            },
        ));
        assert!(!plain_goal_requires_planning_clarification(
            "Build a user management microservice API. Validation: cargo test.",
            &PlanningContextSources::default(),
        ));
        assert!(
            plain_goal_planning_clarification_prompt()
                .contains("What exact outcome should Boundline deliver?")
        );

        let unknowns = planning_unknown_markers(BROAD_GOAL, None, false);
        assert!(unknowns.contains(&"validation_target requires operator confirmation".to_string()));
        assert!(
            unknowns.contains(&"persistence assumptions require operator confirmation".to_string())
        );
        assert!(unknowns.contains(&"authored source provenance is unavailable".to_string()));
        assert!(unknowns.contains(&"desired outcome could not be extracted from goal text and requires operator confirmation".to_string()));
        assert!(unknowns.contains(&"API operations or endpoints in scope could not be identified and require operator specification".to_string()));
        assert!(unknowns.contains(
            &"domain entities and their attributes could not be parsed from goal text".to_string()
        ));

        let no_unknowns = planning_unknown_markers(RICH_GOAL, Some("cargo test"), true);
        assert_eq!(
            no_unknowns,
            vec!["no explicit unknown markers were detected from the captured brief".to_string()]
        );
    }

    #[test]
    fn brief_helpers_cover_planning_brief_and_assumption_rendering() -> Result<(), Box<dyn Error>> {
        let rich_goal_plan = rich_goal_plan()?;
        let rich_sources = PlanningContextSources {
            authored_input_summary: Some("brief.md plus research.md".to_string()),
            authored_input_sources: vec!["brief.md".to_string(), "research.md".to_string()],
            ..PlanningContextSources::default()
        };
        let rich_brief = render_planning_stage_brief(
            "plan:discovery",
            CanonMode::Discovery,
            &rich_goal_plan,
            &rich_sources,
        );
        assert!(rich_brief.contains(super::PLANNING_STAGE_BRIEF_TITLE));
        assert!(rich_brief.contains("- canon_mode: discovery"));
        assert!(rich_brief.contains("- flow: delivery"));
        assert!(rich_brief.contains("- targets: src/api.rs, src/auth.rs"));
        assert!(rich_brief.contains("- authored_input_summary: brief.md plus research.md"));
        assert!(rich_brief.contains("- authored_input_sources: brief.md, research.md"));
        assert!(rich_brief.contains(super::PLANNING_STAGE_CANON_MEMORY_HEADING));
        assert!(rich_brief.contains("governed discovery packet [credible]"));
        assert!(rich_brief.contains("## Structured Goal Decomposition"));
        assert!(rich_brief.contains(
            "## Unknowns\n- no explicit unknown markers were detected from the captured brief"
        ));
        assert!(rich_brief.contains("language/runtime: Rust"));
        assert!(rich_brief.contains("HTTP framework: Axum"));
        assert!(rich_brief.contains("RPC surface: gRPC"));
        assert!(rich_brief.contains("security: OAuth2 protected surface"));
        assert_eq!(planning_problem_domain(&rich_goal_plan), "user management and authentication");
        assert_eq!(
            planning_assumptions(&rich_goal_plan),
            vec![
                "language/runtime: Rust".to_string(),
                "HTTP framework: Axum".to_string(),
                "RPC surface: gRPC".to_string(),
                "security: OAuth2 protected surface".to_string(),
            ]
        );

        let service_goal_plan = sample_goal_plan("Implement a bounded gRPC API service")?;
        assert_eq!(planning_problem_domain(&service_goal_plan), "service/API delivery");

        let default_goal_plan = sample_goal_plan(DEFAULT_GOAL)?;
        let default_brief = render_planning_stage_brief(
            "plan:discovery",
            CanonMode::Discovery,
            &default_goal_plan,
            &PlanningContextSources::default(),
        );
        assert!(default_brief.contains("- flow: unspecified"));
        assert!(default_brief.contains(&format!("- targets: {}", super::PLANNING_DEFAULT_TARGET)));
        assert!(default_brief.contains("- primary_inputs: none"));
        assert!(default_brief.contains("- authored_input_summary: none"));
        assert!(default_brief.contains("- authored_input_sources: none"));
        assert!(!default_brief.contains(super::PLANNING_STAGE_CANON_MEMORY_HEADING));
        assert_eq!(
            planning_assumptions(&default_goal_plan),
            vec!["no concrete technical assumptions were captured".to_string()]
        );
        assert_eq!(
            planning_problem_domain(&default_goal_plan),
            "bounded delivery target from captured goal"
        );
        Ok(())
    }

    #[test]
    fn brief_helpers_cover_execution_brief_and_stage_council_rendering()
    -> Result<(), Box<dyn Error>> {
        let goal_plan = rich_goal_plan()?;
        let decisions = vec![
            sample_decision("src/api.rs", DecisionType::Code, DecisionStatus::Verified),
            sample_decision("src/auth.rs", DecisionType::Fix, DecisionStatus::Recovered),
            sample_decision("tests/api.rs", DecisionType::Test, DecisionStatus::Failed),
            sample_decision("README.md", DecisionType::Analyze, DecisionStatus::Pending),
        ];
        let changed_files_context = TaskContext::new(
            SESSION_ID,
            WORKSPACE_REF,
            RunLimits::default(),
            Map::from_iter([
                (
                    super::super::LATEST_CHANGED_FILES_KEY.to_string(),
                    json!(["src/api.rs", "src/auth.rs"]),
                ),
                (
                    super::LATEST_VALIDATION_STATUS_KEY.to_string(),
                    Value::String("passed".to_string()),
                ),
            ]),
        );
        let execution_brief = render_execution_stage_brief(
            CanonMode::Verification,
            &goal_plan,
            &decisions,
            &changed_files_context,
            &["fallback.rs".to_string()],
        );
        assert!(execution_brief.contains("# Execution Governance Brief"));
        assert!(execution_brief.contains("- canon_mode: verification"));
        assert!(execution_brief.contains("- src/api.rs"));
        assert!(execution_brief.contains("- src/auth.rs"));
        assert!(!execution_brief.contains("fallback.rs"));
        assert!(execution_brief.contains("- status: passed"));
        assert!(execution_brief.contains("## Canon Memory"));
        assert!(
            execution_brief
                .contains("code: src/api.rs (status: verified) -> coverage should improve")
        );
        assert!(
            execution_brief
                .contains("fix: src/auth.rs (status: recovered) -> coverage should improve")
        );
        assert!(
            execution_brief
                .contains("test: tests/api.rs (status: failed) -> coverage should improve")
        );
        assert!(!execution_brief.contains("README.md"));

        let fallback_context =
            TaskContext::new(SESSION_ID, WORKSPACE_REF, RunLimits::default(), Map::new());
        let fallback_brief = render_execution_stage_brief(
            CanonMode::Verification,
            &goal_plan,
            &[],
            &fallback_context,
            &["fallback.rs".to_string()],
        );
        assert!(fallback_brief.contains("- fallback.rs"));
        assert!(fallback_brief.contains("- no terminal decisions were recorded"));

        let no_targets_brief = render_execution_stage_brief(
            CanonMode::Verification,
            &goal_plan,
            &[],
            &fallback_context,
            &[],
        );
        assert!(no_targets_brief.contains("- no bounded file targets were recorded"));

        let request = StageCouncilRequest {
            stage_key: "plan:discovery".to_string(),
            phase: "planning-discovery".to_string(),
            producer_slot: "planning".to_string(),
            target_refs: vec!["brief.md".to_string()],
            current_artifact_ref: Some("brief.md".to_string()),
            goal: "Clarify the bounded scope".to_string(),
            constraints: vec!["preserve delivery defaults".to_string()],
        };
        let findings = vec![
            StageCouncilFinding {
                reviewer_id: "reviewer-a".to_string(),
                effective_route: "copilot/gpt-5.4".to_string(),
                disposition: StageCouncilFindingDisposition::Concern,
                summary: "needs a narrower target".to_string(),
                accepted: false,
            },
            StageCouncilFinding {
                reviewer_id: "reviewer-b".to_string(),
                effective_route: "gemini/gemini-2.5-pro".to_string(),
                disposition: StageCouncilFindingDisposition::Block,
                summary: "independence collapsed".to_string(),
                accepted: false,
            },
        ];
        let blocked_markdown =
            render_stage_council_blocked_markdown(&request, &findings, &["reviewer-a".to_string()]);
        assert!(
            blocked_markdown
                .contains("- reviewer-a [copilot/gpt-5.4] accepted: needs a narrower target")
        );
        assert!(
            blocked_markdown
                .contains("- reviewer-b [gemini/gemini-2.5-pro] rejected: independence collapsed")
        );

        let blocked_without_findings = render_stage_council_blocked_markdown(&request, &[], &[]);
        assert!(
            blocked_without_findings
                .contains("- findings: no provider-backed reviewer findings were recorded")
        );

        let blocked_note = render_stage_council_blocked_note("review routes converged");
        assert!(blocked_note.contains("- reason: review routes converged"));
        Ok(())
    }

    fn rich_goal_plan() -> Result<GoalPlan, Box<dyn Error>> {
        let mut goal_plan = sample_goal_plan(RICH_GOAL)?
            .with_flow(InferredFlow {
                flow_name: "delivery".to_string(),
                confidence_reason: "goal text maps to the delivery lifecycle".to_string(),
                confirmed: true,
            })
            .with_context_pack(ContextPack {
                pack_id: "context-pack-1".to_string(),
                summary: "bounded workspace evidence narrowed the delivery target".to_string(),
                credibility: ContextPackCredibility::Credible,
                inputs: Vec::new(),
                selected_targets: vec!["src/api.rs".to_string(), "src/auth.rs".to_string()],
                advanced_context: None,
                staleness_reason: None,
            })
            .with_compacted_canon_memory(CompactedCanonMemory {
                headline: "governed discovery packet".to_string(),
                credibility: MemoryCredibilityState::Credible,
                stage_key: Some("plan:discovery".to_string()),
                run_ref: Some("run-123".to_string()),
                packet_ref: Some("packet-123".to_string()),
                reason_code: Some("governed-default".to_string()),
                artifact_refs: vec!["brief.md".to_string(), "research.md".to_string()],
                mode_summary: None,
                possible_actions: Vec::new(),
                recommended_next_action: None,
                evidence_summary: None,
                authority_provenance_lines: Vec::new(),
                adaptive_provenance_lines: Vec::new(),
                semantic_provenance_lines: Vec::new(),
            });
        goal_plan.planning_rationale =
            Some("reuse canonical planning context before generating the next slice".to_string());
        goal_plan.verification_strategy = Some("cargo test".to_string());
        Ok(goal_plan)
    }

    fn sample_goal_plan(goal: &str) -> Result<GoalPlan, Box<dyn Error>> {
        GoalPlan::new(
            goal,
            vec![PlannedTask {
                task_id: "task-1".to_string(),
                description: "Implement the bounded slice".to_string(),
                target: "src/orchestrator/session_runtime_briefs.rs".to_string(),
                expected_outcome: Some("coverage should improve".to_string()),
                decision_type_hint: None,
            }],
        )
        .map_err(Into::into)
    }

    fn sample_decision(target: &str, kind: DecisionType, status: DecisionStatus) -> Decision {
        let mut decision = Decision::new(
            kind,
            target,
            "exercise the execution brief renderer",
            "coverage should improve",
            Vec::new(),
        );
        decision.status = status;
        decision
    }
}
