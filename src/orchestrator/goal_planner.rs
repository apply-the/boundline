//! Goal-derived planning from workspace state (feature 013).

use std::fs;
use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, GoalPlanError,
    PlannedTask, WorkspaceSignals,
};

/// Maximum directory traversal depth for workspace signal collection.
const MAX_SCAN_DEPTH: usize = 4;
const MAX_CONTEXT_FILES: usize = 5;
const MAX_SYMBOL_HINTS: usize = 3;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanningContextSources {
    pub authored_input_summary: Option<String>,
    pub authored_input_sources: Vec<String>,
    pub negotiation_goal_summary: Option<String>,
    pub negotiation_resolution: Option<String>,
    pub negotiation_acceptance_boundary: Option<String>,
    pub latest_trace_ref: Option<String>,
}

/// Collect workspace signals from the given workspace root.
pub fn collect_workspace_signals(workspace_ref: &Path) -> WorkspaceSignals {
    let mut signals = WorkspaceSignals::default();

    // Detect language from manifest files
    if workspace_ref.join("Cargo.toml").exists() {
        signals.language = Some("rust".to_string());
    } else if workspace_ref.join("package.json").exists() {
        signals.language = Some("javascript".to_string());
    } else if workspace_ref.join("pyproject.toml").exists()
        || workspace_ref.join("setup.py").exists()
    {
        signals.language = Some("python".to_string());
    } else if workspace_ref.join("go.mod").exists() {
        signals.language = Some("go".to_string());
    }

    // Count files (bounded depth)
    signals.file_count = count_files(workspace_ref, 0);

    // Check for synod config
    signals.has_config = workspace_ref.join(".synod").join("config.toml").exists();

    // Check for Canon artifacts
    signals.has_canon = workspace_ref.join(".canon").is_dir();

    // Check for test directories
    signals.has_tests = workspace_ref.join("tests").is_dir()
        || workspace_ref.join("test").is_dir()
        || workspace_ref.join("spec").is_dir();

    signals
}

fn count_files(dir: &Path, depth: usize) -> usize {
    if depth >= MAX_SCAN_DEPTH {
        return 0;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden dirs and target/node_modules
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                continue;
            }
            count += count_files(&path, depth + 1);
        } else {
            count += 1;
        }
    }
    count
}

fn collect_workspace_files(
    workspace_root: &Path,
    dir: &Path,
    depth: usize,
    files: &mut Vec<String>,
) {
    if depth >= MAX_SCAN_DEPTH {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                continue;
            }
            collect_workspace_files(workspace_root, &path, depth + 1, files);
        } else if let Ok(relative) = path.strip_prefix(workspace_root) {
            files.push(relative.to_string_lossy().to_string());
        }
    }
}

fn goal_keywords(goal_text: &str) -> Vec<String> {
    let mut keywords = goal_text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|part| part.len() >= 3)
        .map(|part| part.to_lowercase())
        .collect::<Vec<_>>();
    keywords.sort();
    keywords.dedup();
    keywords
}

fn file_relevance_score(path: &str, keywords: &[String]) -> usize {
    let lower = path.to_lowercase();
    let mut score = 0;
    for keyword in keywords {
        if lower.contains(keyword) {
            score += 3;
        }
    }
    if lower.starts_with("src/") {
        score += 2;
    }
    if lower.ends_with(".rs") {
        score += 1;
    }
    score
}

fn select_relevant_workspace_files(workspace_ref: &Path, goal_text: &str) -> Vec<String> {
    let keywords = goal_keywords(goal_text);
    let mut files = Vec::new();
    collect_workspace_files(workspace_ref, workspace_ref, 0, &mut files);

    let mut scored = files
        .into_iter()
        .map(|path| {
            let score = file_relevance_score(&path, &keywords);
            (path, score)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let mut selected = scored
        .into_iter()
        .filter(|(_, score)| *score > 0)
        .map(|(path, _)| path)
        .take(MAX_CONTEXT_FILES)
        .collect::<Vec<_>>();

    if selected.is_empty() {
        let primary = select_primary_target(workspace_ref);
        if !primary.is_empty() && workspace_ref.join(&primary).is_file() {
            selected.push(primary);
        }
    }

    selected
}

fn extract_symbol_hints(
    workspace_ref: &Path,
    file_refs: &[String],
    goal_text: &str,
) -> Vec<String> {
    let keywords = goal_keywords(goal_text);
    let mut hints = Vec::new();

    for file_ref in file_refs {
        if hints.len() >= MAX_SYMBOL_HINTS {
            break;
        }
        if !file_ref.ends_with(".rs") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(workspace_ref.join(file_ref)) else {
            continue;
        };
        for line in contents.lines() {
            let trimmed = line.trim_start();
            let symbol = if let Some(rest) = trimmed.strip_prefix("pub fn ") {
                rest.split('(').next()
            } else if let Some(rest) = trimmed.strip_prefix("fn ") {
                rest.split('(').next()
            } else if let Some(rest) = trimmed.strip_prefix("pub struct ") {
                rest.split_whitespace().next()
            } else if let Some(rest) = trimmed.strip_prefix("struct ") {
                rest.split_whitespace().next()
            } else if let Some(rest) = trimmed.strip_prefix("pub enum ") {
                rest.split_whitespace().next()
            } else if let Some(rest) = trimmed.strip_prefix("enum ") {
                rest.split_whitespace().next()
            } else {
                None
            };

            let Some(symbol) = symbol else {
                continue;
            };
            let normalized = symbol.trim().trim_matches('{').trim_matches('(').to_string();
            if normalized.is_empty() {
                continue;
            }
            let lower = normalized.to_lowercase();
            if !keywords.is_empty() && !keywords.iter().any(|keyword| lower.contains(keyword)) {
                continue;
            }
            hints.push(format!("{file_ref}::{normalized}"));
            if hints.len() >= MAX_SYMBOL_HINTS {
                break;
            }
        }
    }

    hints
}

fn selected_canon_artifacts(workspace_ref: &Path, goal_text: &str) -> Vec<String> {
    let keywords = goal_keywords(goal_text);
    let evidence = scan_canon_artifacts(workspace_ref);
    let mut selected = evidence
        .into_iter()
        .filter(|item| {
            if keywords.is_empty() {
                return true;
            }
            let lower = item.reference.to_lowercase();
            keywords.iter().any(|keyword| lower.contains(keyword))
        })
        .map(|item| item.reference)
        .take(MAX_CONTEXT_FILES)
        .collect::<Vec<_>>();

    if selected.is_empty() && workspace_ref.join(".canon").is_dir() {
        selected = scan_canon_artifacts(workspace_ref)
            .into_iter()
            .map(|item| item.reference)
            .take(1)
            .collect();
    }

    selected
}

pub fn build_context_pack(
    goal_text: &str,
    workspace_ref: &Path,
    context_sources: &PlanningContextSources,
) -> ContextPack {
    let relevant_files = select_relevant_workspace_files(workspace_ref, goal_text);
    let symbol_hints = extract_symbol_hints(workspace_ref, &relevant_files, goal_text);
    let canon_artifacts = selected_canon_artifacts(workspace_ref, goal_text);

    let mut inputs = Vec::new();

    for file_ref in &relevant_files {
        inputs.push(ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: file_ref.clone(),
            rationale: "selected as a bounded workspace target for the current goal".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        });
    }

    for symbol_hint in symbol_hints {
        inputs.push(ContextInput {
            kind: ContextInputKind::SymbolHint,
            reference: symbol_hint,
            rationale: "matched a bounded symbol hint inside the selected workspace files"
                .to_string(),
            source: "symbol_scan".to_string(),
            primary: false,
        });
    }

    if let Some(summary) = context_sources.authored_input_summary.as_ref() {
        inputs.push(ContextInput {
            kind: ContextInputKind::AuthoredBrief,
            reference: summary.clone(),
            rationale: "captures the operator-authored task framing".to_string(),
            source: "authored_input_summary".to_string(),
            primary: relevant_files.is_empty(),
        });
    }

    for source_label in &context_sources.authored_input_sources {
        inputs.push(ContextInput {
            kind: ContextInputKind::AuthoredBrief,
            reference: source_label.clone(),
            rationale: "records which authored source contributed to the bounded context"
                .to_string(),
            source: "authored_input_source".to_string(),
            primary: false,
        });
    }

    if let Some(goal_summary) = context_sources.negotiation_goal_summary.as_ref() {
        inputs.push(ContextInput {
            kind: ContextInputKind::Negotiation,
            reference: goal_summary.clone(),
            rationale: "keeps the negotiated delivery target visible during planning".to_string(),
            source: "negotiation_goal_summary".to_string(),
            primary: false,
        });
    }

    if let Some(trace_ref) = context_sources.latest_trace_ref.as_ref() {
        inputs.push(ContextInput {
            kind: ContextInputKind::RecentTrace,
            reference: trace_ref.clone(),
            rationale: "reuses the latest persisted trace as bounded historical evidence"
                .to_string(),
            source: "latest_trace_ref".to_string(),
            primary: false,
        });
    }

    for artifact_ref in &canon_artifacts {
        inputs.push(ContextInput {
            kind: ContextInputKind::CanonArtifact,
            reference: artifact_ref.clone(),
            rationale: "reuses a bounded governed artifact as planning input".to_string(),
            source: "canon_artifact_scan".to_string(),
            primary: relevant_files.is_empty(),
        });
    }

    let has_credible_context = !relevant_files.is_empty()
        || context_sources.authored_input_summary.is_some()
        || !canon_artifacts.is_empty()
        || context_sources.latest_trace_ref.is_some();
    let credibility = if has_credible_context {
        ContextPackCredibility::Credible
    } else {
        ContextPackCredibility::Insufficient
    };
    let summary = if has_credible_context {
        format!(
            "bounded context from {} primary input(s)",
            usize::max(relevant_files.len(), canon_artifacts.len()).max(1)
        )
    } else {
        format!("no credible bounded context found for planning `{}`", goal_text.trim())
    };

    ContextPack {
        pack_id: Uuid::new_v4().to_string(),
        summary,
        credibility,
        inputs,
        selected_targets: if !relevant_files.is_empty() { relevant_files } else { canon_artifacts },
        staleness_reason: None,
    }
}

fn select_primary_target(workspace_ref: &Path) -> String {
    for candidate in ["src/lib.rs", "src/main.rs", "Cargo.toml", "README.md"] {
        if workspace_ref.join(candidate).is_file() {
            return candidate.to_string();
        }
    }

    first_workspace_file(workspace_ref, workspace_ref, 0)
        .unwrap_or_else(|| workspace_ref.to_string_lossy().to_string())
}

fn first_workspace_file(workspace_root: &Path, dir: &Path, depth: usize) -> Option<String> {
    if depth >= MAX_SCAN_DEPTH {
        return None;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                continue;
            }

            if let Some(found) = first_workspace_file(workspace_root, &path, depth + 1) {
                return Some(found);
            }
        } else if let Ok(relative) = path.strip_prefix(workspace_root) {
            return Some(relative.to_string_lossy().to_string());
        }
    }

    None
}

/// Scan Canon artifacts directory and return evidence references.
pub fn scan_canon_artifacts(workspace_ref: &Path) -> Vec<EvidenceRef> {
    let canon_dir = workspace_ref.join(".canon");
    if !canon_dir.is_dir() {
        return Vec::new();
    }
    let mut evidence = Vec::new();
    if let Ok(entries) = fs::read_dir(&canon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let rel = path.strip_prefix(workspace_ref).unwrap_or(&path);
                evidence.push(EvidenceRef::canon(rel.to_string_lossy()));
            }
        }
    }
    evidence
}

/// Derive a bounded task list from goal text and workspace signals.
pub fn derive_tasks(
    goal_text: &str,
    workspace_ref: &Path,
    signals: &WorkspaceSignals,
) -> Vec<PlannedTask> {
    let mut tasks = Vec::new();
    let goal_lower = goal_text.to_lowercase();
    let primary_target = select_primary_target(workspace_ref);

    // Always start with an analysis task
    tasks.push(PlannedTask {
        task_id: Uuid::new_v4().to_string(),
        description: format!("Analyze workspace for: {goal_text}"),
        target: primary_target.clone(),
        expected_outcome: Some("understanding of current state and required changes".to_string()),
        decision_type_hint: Some(DecisionType::Analyze),
    });

    // Derive implementation tasks from goal keywords
    if goal_lower.contains("fix") || goal_lower.contains("bug") || goal_lower.contains("broken") {
        tasks.push(PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: format!("Fix: {goal_text}"),
            target: primary_target.clone(),
            expected_outcome: Some("issue resolved".to_string()),
            decision_type_hint: Some(DecisionType::Fix),
        });
    } else {
        tasks.push(PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: format!("Implement: {goal_text}"),
            target: primary_target,
            expected_outcome: Some("changes applied".to_string()),
            decision_type_hint: Some(DecisionType::Code),
        });
    }

    // Always end with verification
    if signals.has_tests {
        tasks.push(PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: "Run tests to verify changes".to_string(),
            target: "test suite".to_string(),
            expected_outcome: Some("all tests pass".to_string()),
            decision_type_hint: Some(DecisionType::Test),
        });
    }

    tasks
}

fn derive_tasks_from_context(
    goal_text: &str,
    context_pack: &ContextPack,
    workspace_ref: &Path,
    signals: &WorkspaceSignals,
) -> Vec<PlannedTask> {
    let mut tasks = derive_tasks(goal_text, workspace_ref, signals);
    let goal_lower = goal_text.to_lowercase();
    let primary_target = if goal_lower.contains("fix")
        || goal_lower.contains("implement")
        || goal_lower.contains("build")
        || goal_lower.contains("change")
        || goal_lower.contains("feature")
    {
        context_pack
            .selected_targets
            .iter()
            .find(|target| target.starts_with("src/"))
            .cloned()
            .or_else(|| {
                context_pack
                    .selected_targets
                    .iter()
                    .find(|target| !target.starts_with("tests/"))
                    .cloned()
            })
    } else {
        None
    }
    .or_else(|| {
        context_pack.selected_targets.iter().find(|target| !target.trim().is_empty()).cloned()
    })
    .unwrap_or_else(|| select_primary_target(workspace_ref));

    for task in &mut tasks {
        if task.target == "test suite" {
            continue;
        }
        task.target = primary_target.clone();
        if let Some(expected_outcome) = &task.expected_outcome {
            task.expected_outcome =
                Some(format!("{expected_outcome}; context: {}", context_pack.summary));
        }
    }

    tasks
}

pub fn build_goal_plan_with_sources(
    goal_text: &str,
    workspace_ref: &Path,
    context_sources: &PlanningContextSources,
) -> Result<GoalPlan, GoalPlannerError> {
    if goal_text.trim().is_empty() {
        return Err(GoalPlannerError::MissingGoal);
    }

    let signals = collect_workspace_signals(workspace_ref);
    let context_pack = build_context_pack(goal_text, workspace_ref, context_sources);
    let tasks = derive_tasks_from_context(goal_text, &context_pack, workspace_ref, &signals);
    let canon_evidence = scan_canon_artifacts(workspace_ref);

    let plan = GoalPlan::new(goal_text, tasks)
        .map_err(GoalPlannerError::PlanCreation)?
        .with_context_pack(context_pack)
        .with_signals(signals)
        .with_evidence(canon_evidence);

    if plan.context_pack.as_ref().map(|pack| pack.credibility)
        != Some(ContextPackCredibility::Credible)
    {
        let summary = plan.context_summary().unwrap_or_else(|| {
            "goal planning stopped because the bounded context is not credible".to_string()
        });
        return Err(GoalPlannerError::InsufficientContext { summary, goal_plan: Box::new(plan) });
    }

    Ok(plan)
}

/// Build a complete goal plan from goal text and workspace.
pub fn build_goal_plan(
    goal_text: &str,
    workspace_ref: &Path,
) -> Result<GoalPlan, GoalPlannerError> {
    build_goal_plan_with_sources(goal_text, workspace_ref, &PlanningContextSources::default())
}

#[derive(Debug, Error)]
pub enum GoalPlannerError {
    #[error("no goal text provided — run `synod capture` first")]
    MissingGoal,
    #[error("goal planning stopped because the bounded context is not credible: {summary}")]
    InsufficientContext { summary: String, goal_plan: Box<GoalPlan> },
    #[error("failed to create goal plan: {0}")]
    PlanCreation(#[from] GoalPlanError),
}
