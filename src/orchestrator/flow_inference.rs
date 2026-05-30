//! Flow inference from goal text and bounded planning evidence.

use crate::domain::goal_plan::{ContextInputKind, ContextPack, InferredFlow, WorkspaceSignals};
use crate::domain::workflow::WorkflowProgressState;

/// Keywords that map to the bug-fix flow.
const BUG_FIX_KEYWORDS: &[&str] =
    &["fix", "bug", "broken", "failing", "regression", "crash", "error"];

/// Keywords that map to the change flow.
const CHANGE_KEYWORDS: &[&str] = &["change", "update", "modify", "extend", "prepare", "refactor"];

/// Keywords that map to the delivery flow.
const DELIVERY_KEYWORDS: &[&str] = &["deliver", "release", "ship", "deploy", "complete", "launch"];
const DELIVERY_BUILD_KEYWORDS: &[&str] = &[
    "add",
    "build",
    "implement",
    "feature",
    "new",
    "create",
    "bootstrap",
    "initialize",
    "scaffold",
    "first slice",
];
const DELIVERY_SHAPE_KEYWORDS: &[&str] = &[
    "api",
    "apis",
    "endpoint",
    "endpoints",
    "grpc",
    "microservice",
    "microservizio",
    "service",
    "oauth",
    "authorization",
    "authenticates",
    "crud",
    "rbac",
    "role",
    "roles",
    "user management",
];
const EXISTING_SYSTEM_CHANGE_KEYWORDS: &[&str] =
    &["existing", "change", "update", "modify", "extend", "refactor", "prepare"];

/// Inputs used to infer a delivery flow from bounded planning evidence.
#[derive(Debug, Clone, Copy)]
pub struct FlowInferenceContext<'a> {
    pub goal_text: &'a str,
    pub context_pack: Option<&'a ContextPack>,
    pub workspace_signals: &'a WorkspaceSignals,
    pub workflow_progress: Option<&'a WorkflowProgressState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowCandidate {
    BugFix,
    Change,
    Delivery,
}

impl FlowCandidate {
    const fn flow_name(self) -> &'static str {
        match self {
            Self::BugFix => "bug-fix",
            Self::Change => "change",
            Self::Delivery => "delivery",
        }
    }

    const fn priority(self) -> usize {
        match self {
            Self::BugFix => 0,
            Self::Delivery => 1,
            Self::Change => 2,
        }
    }
}

/// Infers a delivery flow from bounded planning context, workspace signals, and
/// optional workflow progress.
pub fn infer_flow_from_context(context: &FlowInferenceContext<'_>) -> Option<InferredFlow> {
    let bug_fix = score_bug_fix(context);
    let change = score_change(context);
    let delivery = score_delivery(context);

    let mut candidates = vec![
        (FlowCandidate::BugFix, bug_fix.0, bug_fix.1),
        (FlowCandidate::Change, change.0, change.1),
        (FlowCandidate::Delivery, delivery.0, delivery.1),
    ]
    .into_iter()
    .filter(|(_, score, _)| *score > 0)
    .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right.1.cmp(&left.1).then_with(|| left.0.priority().cmp(&right.0.priority()))
    });

    let (candidate, _, reasons) = candidates.into_iter().next()?;
    let confidence_reason = format!(
        "evidence suggests {} because {}",
        candidate.flow_name(),
        reasons.into_iter().take(2).collect::<Vec<_>>().join("; ")
    );

    Some(InferredFlow {
        flow_name: candidate.flow_name().to_string(),
        confidence_reason,
        confirmed: false,
    })
}

/// Infer a flow from goal text using keyword matching.
///
/// Returns `Some(InferredFlow)` if a match is found, `None` if no keywords
/// match (caller should default to no-flow mode).
pub fn infer_flow(goal_text: &str) -> Option<InferredFlow> {
    // Check bug-fix first (highest priority for safety)
    for keyword in BUG_FIX_KEYWORDS {
        if contains_goal_cue(&goal_text.to_lowercase(), keyword) {
            return Some(InferredFlow {
                flow_name: "bug-fix".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    let lower = goal_text.to_lowercase();
    let change_is_explicit = matched_keywords(goal_text, EXISTING_SYSTEM_CHANGE_KEYWORDS);
    let delivery_shapes = matched_keywords(goal_text, DELIVERY_SHAPE_KEYWORDS);
    let delivery_build = matched_keywords(goal_text, DELIVERY_BUILD_KEYWORDS);

    if !delivery_shapes.is_empty()
        && (!delivery_build.is_empty()
            || !matched_keywords(goal_text, DELIVERY_KEYWORDS).is_empty())
        && change_is_explicit.is_empty()
    {
        return Some(InferredFlow {
            flow_name: "delivery".to_string(),
            confidence_reason: format!(
                "goal describes a concrete delivery surface via {}",
                delivery_shapes.join(", ")
            ),
            confirmed: false,
        });
    }

    // Check delivery before change (broader scope takes precedence)
    for keyword in DELIVERY_KEYWORDS {
        if contains_goal_cue(&lower, keyword) {
            return Some(InferredFlow {
                flow_name: "delivery".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    // Check change last
    for keyword in CHANGE_KEYWORDS {
        if contains_goal_cue(&lower, keyword) {
            return Some(InferredFlow {
                flow_name: "change".to_string(),
                confidence_reason: format!("goal contains keyword '{keyword}'"),
                confirmed: false,
            });
        }
    }

    None
}

fn score_bug_fix(context: &FlowInferenceContext<'_>) -> (usize, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();

    let matched_keywords = matched_keywords(context.goal_text, BUG_FIX_KEYWORDS);
    if !matched_keywords.is_empty() {
        score += 3 * matched_keywords.len();
        reasons.push(format!(
            "goal language mentions {}",
            matched_keywords
                .iter()
                .map(|keyword| format!("`{keyword}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if has_recent_trace(context) {
        score += 3;
        reasons.push("recent trace evidence is available".to_string());
    }
    if has_source_target(context) && has_test_target(context) {
        score += 3;
        reasons.push("selected targets span existing tests and source files".to_string());
    }
    if context.workspace_signals.has_tests {
        score += 1;
        reasons.push("workspace already contains tests".to_string());
    }

    (score, reasons)
}

fn score_change(context: &FlowInferenceContext<'_>) -> (usize, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();
    let bug_fix_keywords = matched_keywords(context.goal_text, BUG_FIX_KEYWORDS);

    let matched_keywords = matched_keywords(context.goal_text, CHANGE_KEYWORDS);
    if !matched_keywords.is_empty() {
        score += 3 * matched_keywords.len();
        reasons.push(format!(
            "goal language mentions {}",
            matched_keywords
                .iter()
                .map(|keyword| format!("`{keyword}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if has_source_target(context) {
        score += 2;
        reasons.push("selected targets focus on implementation files".to_string());
    }
    if symbol_hint_count(context) > 0 {
        score += 1;
        reasons.push("symbol hints identify a bounded implementation surface".to_string());
    }
    if bug_fix_keywords.is_empty() && !has_test_target(context) && !has_recent_trace(context) {
        score += 1;
        reasons.push("workspace evidence does not point to an existing failing test".to_string());
    }

    (score, reasons)
}

fn score_delivery(context: &FlowInferenceContext<'_>) -> (usize, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();

    let delivery_keywords = matched_keywords(context.goal_text, DELIVERY_KEYWORDS);
    if !delivery_keywords.is_empty() {
        score += 4 * delivery_keywords.len();
        reasons.push(format!(
            "goal language mentions {}",
            delivery_keywords
                .iter()
                .map(|keyword| format!("`{keyword}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let delivery_shapes = matched_keywords(context.goal_text, DELIVERY_SHAPE_KEYWORDS);
    let delivery_build = matched_keywords(context.goal_text, DELIVERY_BUILD_KEYWORDS);
    let change_is_explicit = matched_keywords(context.goal_text, EXISTING_SYSTEM_CHANGE_KEYWORDS);
    if !delivery_shapes.is_empty()
        && (!delivery_build.is_empty() || !delivery_keywords.is_empty())
        && change_is_explicit.is_empty()
    {
        score += 6 + delivery_shapes.len();
        reasons.push(format!(
            "goal describes a concrete delivery surface via {}",
            delivery_shapes
                .iter()
                .map(|keyword| format!("`{keyword}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if workflow_delivery_hint(context) {
        score += 2;
        reasons.push("workflow guidance points at delivery or release follow-through".to_string());
    }
    if negotiation_input_count(context) > 0 && doc_target_count(context) > 0 {
        score += 1;
        reasons.push(
            "delivery context includes negotiated goals and release-facing targets".to_string(),
        );
    }

    (score, reasons)
}

fn matched_keywords(goal_text: &str, keywords: &[&str]) -> Vec<String> {
    let lower = goal_text.to_lowercase();
    keywords
        .iter()
        .filter(|keyword| contains_goal_cue(&lower, keyword))
        .map(|keyword| (*keyword).to_string())
        .collect()
}

fn contains_goal_cue(lower: &str, cue: &str) -> bool {
    if cue.contains(' ') {
        return lower.contains(cue);
    }

    lower
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .any(|word| word == cue)
}

fn has_recent_trace(context: &FlowInferenceContext<'_>) -> bool {
    context.context_pack.is_some_and(|pack| {
        pack.inputs.iter().any(|input| input.kind == ContextInputKind::RecentTrace)
    })
}

fn has_source_target(context: &FlowInferenceContext<'_>) -> bool {
    context
        .context_pack
        .is_some_and(|pack| pack.selected_targets.iter().any(|target| is_source_target(target)))
}

fn has_test_target(context: &FlowInferenceContext<'_>) -> bool {
    context
        .context_pack
        .is_some_and(|pack| pack.selected_targets.iter().any(|target| is_test_target(target)))
}

fn symbol_hint_count(context: &FlowInferenceContext<'_>) -> usize {
    context.context_pack.map_or(0, |pack| {
        pack.inputs.iter().filter(|input| input.kind == ContextInputKind::SymbolHint).count()
    })
}

fn negotiation_input_count(context: &FlowInferenceContext<'_>) -> usize {
    context.context_pack.map_or(0, |pack| {
        pack.inputs.iter().filter(|input| input.kind == ContextInputKind::Negotiation).count()
    })
}

fn doc_target_count(context: &FlowInferenceContext<'_>) -> usize {
    context.context_pack.map_or(0, |pack| {
        pack.selected_targets.iter().filter(|target| is_doc_target(target)).count()
    })
}

fn workflow_delivery_hint(context: &FlowInferenceContext<'_>) -> bool {
    let Some(workflow_progress) = context.workflow_progress else {
        return false;
    };

    let mut workflow_text = workflow_progress.current_phase_text().unwrap_or_default();
    if let Some(next_action) = workflow_progress.next_action_text() {
        if !workflow_text.is_empty() {
            workflow_text.push(' ');
        }
        workflow_text.push_str(&next_action);
    }

    let lower = workflow_text.to_lowercase();
    DELIVERY_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
}

fn is_source_target(target: &str) -> bool {
    let lower = target.to_lowercase();
    (lower.starts_with("src/") || lower.ends_with(".rs")) && !is_test_target(target)
}

fn is_test_target(target: &str) -> bool {
    let lower = target.to_lowercase();
    lower.starts_with("tests/")
        || lower.starts_with("test/")
        || lower.starts_with("spec/")
        || lower.contains("_test")
        || lower.contains("test") && lower.ends_with(".rs")
}

fn is_doc_target(target: &str) -> bool {
    let lower = target.to_lowercase();
    lower.ends_with(".md") || lower.contains("readme") || lower.contains("changelog")
}

#[cfg(test)]
mod tests {
    use super::{FlowInferenceContext, infer_flow, infer_flow_from_context};
    use crate::domain::goal_plan::WorkspaceSignals;

    #[test]
    fn infer_flow_prefers_delivery_for_concrete_greenfield_feature_goals() {
        let inferred = infer_flow(
            "Implement the first slice of a Rust user-management microservice with REST endpoints, gRPC methods, and OAuth2 authorization",
        )
        .expect("concrete feature goal should infer a flow");

        assert_eq!(inferred.flow_name, "delivery");
    }

    #[test]
    fn infer_flow_from_context_prefers_delivery_for_concrete_service_surfaces() {
        let inferred = infer_flow_from_context(&FlowInferenceContext {
            goal_text:
                "Implement the first slice of a Rust user-management microservice with REST endpoints, gRPC methods, and OAuth2 authorization",
            context_pack: None,
            workspace_signals: &WorkspaceSignals::default(),
            workflow_progress: None,
        })
        .expect("concrete feature goal should infer a flow");

        assert_eq!(inferred.flow_name, "delivery");
    }

    #[test]
    fn infer_flow_keeps_existing_system_updates_on_change() {
        let inferred = infer_flow("Update the existing onboarding flow to add audit logging")
            .expect("existing-system change should infer a flow");

        assert_eq!(inferred.flow_name, "change");
    }
}
