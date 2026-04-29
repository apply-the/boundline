//! Goal-derived planning from workspace state (feature 013).

use std::fs;
use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::goal_plan::{GoalPlan, GoalPlanError, PlannedTask, WorkspaceSignals};

/// Maximum directory traversal depth for workspace signal collection.
const MAX_SCAN_DEPTH: usize = 4;

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

/// Build a complete goal plan from goal text and workspace.
pub fn build_goal_plan(
    goal_text: &str,
    workspace_ref: &Path,
) -> Result<GoalPlan, GoalPlannerError> {
    if goal_text.trim().is_empty() {
        return Err(GoalPlannerError::MissingGoal);
    }

    let signals = collect_workspace_signals(workspace_ref);
    let tasks = derive_tasks(goal_text, workspace_ref, &signals);
    let canon_evidence = scan_canon_artifacts(workspace_ref);

    let plan = GoalPlan::new(goal_text, tasks)
        .map_err(GoalPlannerError::PlanCreation)?
        .with_signals(signals)
        .with_evidence(canon_evidence);

    Ok(plan)
}

#[derive(Debug, Error)]
pub enum GoalPlannerError {
    #[error("no goal text provided — run `synod capture` first")]
    MissingGoal,
    #[error("failed to create goal plan: {0}")]
    PlanCreation(#[from] GoalPlanError),
}
