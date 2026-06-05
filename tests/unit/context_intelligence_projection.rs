use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{
    ContextInclusionMode, ContextOmissionSeverity, HybridOutcome, ImpactFindingKind,
    RelationshipKind, RepositoryMapState, RetrievalBudgets, RetrievalMode, RetrievalState,
    SemanticCapabilityState, SemanticPolicyState, SnapshotCacheState,
};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection,
};
use boundline::orchestrator::goal_planner::{PlanningContextSources, build_context_pack};
use uuid::Uuid;

const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
const SEMANTIC_VECTOR_STATE_READY_VALUE: &str = "ready";
const SEMANTIC_VECTOR_STATE_DEGRADED_VALUE: &str = "degraded";
const SEMANTIC_VECTOR_STATE_CORRUPT_VALUE: &str = "corrupt";

static SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK: Mutex<()> = Mutex::new(());

struct EnvVarGuard {
    name: &'static str,
    previous: Option<OsString>,
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            unsafe {
                std::env::set_var(self.name, previous);
            }
        } else {
            unsafe {
                std::env::remove_var(self.name);
            }
        }
    }
}

fn set_env_var(name: &'static str, value: &str) -> EnvVarGuard {
    let previous = std::env::var_os(name);
    unsafe {
        std::env::set_var(name, value);
    }
    EnvVarGuard { name, previous }
}

/// Creates a temporary workspace root for bounded context-intelligence tests.
fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

#[test]
fn build_context_pack_projects_selected_local_evidence_for_workspace_targets() {
    let workspace = temp_workspace("boundline-context-intelligence-local");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("tests/lib.rs"),
        "#[test]\nfn add_works() { assert_eq!(crate::add(2, 2), 4); }\n",
    )
    .unwrap();

    let context_pack = build_context_pack(
        "Fix the add implementation",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Selected);
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| candidate.source_ref == "src/lib.rs")
    );
    assert!(advanced_context.relationships.iter().any(|relationship| {
        relationship.relationship_kind == RelationshipKind::ExercisesTest
            && relationship.subject_ref == "src/lib.rs"
    }));
}

#[test]
fn build_context_pack_records_missing_test_findings_for_uncovered_targets() {
    let workspace = temp_workspace("boundline-context-intelligence-impact");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/engine.rs"), "pub fn reconcile_plan() -> bool { true }\n")
        .unwrap();

    let context_pack = build_context_pack(
        "Reconcile the planner engine",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert!(advanced_context.relationships.iter().any(|relationship| {
        relationship.relationship_kind == RelationshipKind::RequiresEvidence
            && relationship.subject_ref == "src/engine.rs"
    }));
    assert!(advanced_context.impact_findings.iter().any(|finding| {
        finding.finding_kind == ImpactFindingKind::MissingTest
            && finding.subject_ref == "tests/engine.rs"
    }));
}

#[test]
fn build_context_pack_emits_validation_group_inputs_for_assumptions_and_hidden_impact() {
    let workspace = temp_workspace("boundline-context-intelligence-groups");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/engine.rs"), "pub fn reconcile_plan() -> bool { true }\n")
        .unwrap();

    let context_pack = build_context_pack(
        "Reconcile the planner engine",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert!(advanced_context.relationships.iter().any(|relationship| {
        relationship.relationship_kind == RelationshipKind::RequiresEvidence
            && relationship.subject_ref == "src/engine.rs"
    }));
    assert!(advanced_context.impact_findings.iter().any(|finding| {
        finding.finding_kind == ImpactFindingKind::MissingTest
            && finding.subject_ref == "tests/engine.rs"
    }));
}

#[test]
fn build_context_pack_projects_advanced_context_from_authored_input_without_files() {
    let workspace = temp_workspace("boundline-context-intelligence-insufficient");
    let context_pack = build_context_pack(
        "Plan a bounded change from authored input",
        &workspace,
        &PlanningContextSources {
            authored_input_summary: Some(
                "operator notes: refresh the bounded reconciliation flow".to_string(),
            ),
            ..PlanningContextSources::default()
        },
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Selected);
    assert!(!advanced_context.selected_evidence.is_empty());
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| candidate.source_ref.contains("operator notes:"))
    );
}

#[test]
fn build_context_pack_respects_disabled_advanced_context_policy() {
    let workspace = temp_workspace("boundline-context-intelligence-disabled");
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        "version = 1\n\n[routing.advanced_context]\nretrieval_mode = \"disabled\"\nremote_policy = \"blocked\"\n",
    )
    .unwrap();

    let context_pack = build_context_pack(
        "Fix the add implementation",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Disabled);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Insufficient);
    assert!(advanced_context.selected_evidence.is_empty());
    assert!(
        advanced_context
            .terminal_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("disabled by configuration"))
    );
}

#[test]
fn build_context_pack_surfaces_local_semantic_acceleration_policy() {
    let workspace = temp_workspace("boundline-context-intelligence-semantic-local");
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        concat!(
            "version = 1\n\n",
            "[routing.advanced_context]\n",
            "retrieval_mode = \"local\"\n",
            "remote_policy = \"local_only\"\n\n",
            "[routing.semantic_acceleration]\n",
            "policy = \"local\"\n",
        ),
    )
    .unwrap();

    let context_pack = build_context_pack(
        "Fix the add implementation",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.semantic_policy_state, SemanticPolicyState::Local);
    assert_eq!(advanced_context.hybrid_outcome, HybridOutcome::Skipped);
    assert!(advanced_context.terminal_reason.as_deref().is_some_and(|reason| {
        reason.contains("semantic acceleration")
            || reason.contains("semantic refresh")
            || reason.contains("sqlite-vec")
    }));
}

#[test]
fn build_context_pack_records_semantic_selection_and_rejection_annotations() {
    let _guard =
        SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard =
        set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = temp_workspace("boundline-context-intelligence-semantic-annotations");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        concat!("pub fn planner() -> bool {\n", "    true\n", "}\n",),
    )
    .unwrap();
    fs::write(
        workspace.join("src/semantic.rs"),
        "pub fn reconcileConfigState() -> bool { true }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("src/alternate.rs"),
        "pub fn reconcilePlanningConfiguration() -> bool { true }\n",
    )
    .unwrap();
    let advanced_context = build_advanced_context_projection(
        "planner reconcile configuration state",
        &workspace,
        &[
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "selected bounded implementation surface".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            },
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/semantic.rs".to_string(),
                rationale: "related implementation surface".to_string(),
                source: "workspace_scan".to_string(),
                primary: false,
            },
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/alternate.rs".to_string(),
                rationale: "alternate related implementation surface".to_string(),
                source: "workspace_scan".to_string(),
                primary: false,
            },
        ],
        &[],
        AdvancedContextBuildState {
            credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig {
            budgets: RetrievalBudgets {
                evidence_limit: 2,
                expansion_limit: 4,
                ..RetrievalBudgets::default()
            },
            ..AdvancedContextConfig::default()
        },
    );

    assert_eq!(advanced_context.semantic_policy_state, SemanticPolicyState::Local);
    assert_eq!(advanced_context.hybrid_outcome, HybridOutcome::Expanded);
    assert_eq!(advanced_context.semantic_selected_count(), 1);
    assert_eq!(advanced_context.semantic_rejected_count(), 1);
    assert!(advanced_context.selected_evidence.iter().any(|candidate| {
        candidate.match_origin.as_str() == "semantic_expand"
            && candidate.semantic_score.is_some()
            && candidate.selection_reason.contains("expanded the V1 candidate set")
    }));
    assert!(advanced_context.rejected_candidates.iter().any(|candidate| {
        candidate.match_origin.as_str() == "semantic_expand"
            && candidate.semantic_score.is_some()
            && candidate
                .selection_reason
                .contains("bounded evidence limit kept the V1 set unchanged")
    }));
}

#[test]
fn build_context_pack_persists_derived_index_manifest_sidecar() {
    let workspace = temp_workspace("boundline-context-intelligence-manifest");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();

    let advanced_context = build_advanced_context_projection(
        "Fix the add implementation",
        &workspace,
        &[ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "selected bounded implementation surface".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        &[],
        AdvancedContextBuildState {
            credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    assert_eq!(advanced_context.retrieval_state, RetrievalState::Selected);
    let manifest_path = workspace.join(".boundline/context-intelligence/manifest.json");
    assert!(manifest_path.is_file());
    let manifest = fs::read_to_string(manifest_path).unwrap();
    assert!(manifest.contains("\"schema_version\": \"retrieval-index-v3\""));
    assert!(manifest.contains("\"workspace_fingerprint\""));
}

#[test]
fn build_context_pack_surfaces_degraded_and_corrupt_semantic_capability_states() {
    for (override_value, expected_state, expected_reason) in [
        (SEMANTIC_VECTOR_STATE_DEGRADED_VALUE, SemanticCapabilityState::Degraded, "degraded"),
        (SEMANTIC_VECTOR_STATE_CORRUPT_VALUE, SemanticCapabilityState::Corrupt, "corrupt"),
    ] {
        let _guard = SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env_guard = set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, override_value);
        let workspace = temp_workspace("boundline-context-intelligence-semantic-fallback");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();

        let advanced_context = build_advanced_context_projection(
            "Fix the add implementation",
            &workspace,
            &[ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "selected bounded implementation surface".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            &[],
            AdvancedContextBuildState {
                credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
                staleness_reason: None,
                semantic_policy: SemanticAccelerationPolicyState::Local,
            },
            &AdvancedContextConfig::default(),
        );

        assert_eq!(advanced_context.semantic_policy_state, SemanticPolicyState::Local);
        assert_eq!(advanced_context.semantic_capability_state, expected_state);
        assert!(
            advanced_context
                .terminal_reason
                .as_deref()
                .is_some_and(|reason| reason.contains(expected_reason))
        );
    }
}

#[test]
fn build_context_pack_handles_empty_fts_query_for_short_goal_and_path_tokens() {
    let _guard =
        SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard =
        set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = temp_workspace("boundline-context-intelligence-empty-query");
    fs::create_dir_all(workspace.join("aa")).unwrap();
    fs::write(workspace.join("aa/bb.c"), "int ok(void) { return 1; }\n").unwrap();

    let advanced_context = build_advanced_context_projection(
        "go",
        &workspace,
        &[ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "aa/bb.c".to_string(),
            rationale: "short-path empty-query fixture".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        &[],
        AdvancedContextBuildState {
            credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_ne!(advanced_context.retrieval_state, RetrievalState::Unavailable);
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| candidate.source_ref == "aa/bb.c")
    );
}

#[test]
fn build_advanced_context_projection_surfaces_digest_compaction_and_patch_safe_guards() {
    let workspace = temp_workspace("boundline-context-intelligence-substrate");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("logs")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        format!("pub fn add() -> i32 {{\n{}\n0\n}}\n", "    let x = 1;\n".repeat(1200)),
    )
    .unwrap();
    fs::write(
        workspace.join("logs/failed-run.log"),
        format!("{}\n", "validation failure".repeat(1500)),
    )
    .unwrap();

    let advanced_context = build_advanced_context_projection(
        "fix add validation log",
        &workspace,
        &[
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "bounded implementation target".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            },
            ContextInput {
                kind: ContextInputKind::RecentTrace,
                reference: "logs/failed-run.log".to_string(),
                rationale: "latest failing validation evidence".to_string(),
                source: "latest_trace".to_string(),
                primary: false,
            },
        ],
        &["src/lib.rs".to_string(), "logs/failed-run.log".to_string()],
        AdvancedContextBuildState {
            credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Disabled,
        },
        &AdvancedContextConfig {
            budgets: RetrievalBudgets { evidence_limit: 6, ..RetrievalBudgets::default() },
            ..AdvancedContextConfig::default()
        },
    );

    assert_eq!(advanced_context.repository_map_state, Some(RepositoryMapState::Ready));
    assert_eq!(advanced_context.snapshot_cache_state, Some(SnapshotCacheState::Ready));
    assert!(advanced_context.context_pack_entries.iter().any(|entry| {
        entry.source_ref == "src/lib.rs"
            && entry.inclusion_mode == ContextInclusionMode::Excerpt
            && entry.required_for_admission
    }));
    assert!(advanced_context.context_pack_entries.iter().any(|entry| {
        entry.source_ref == "logs/failed-run.log"
            && entry.inclusion_mode == ContextInclusionMode::Digest
    }));
    assert!(advanced_context.omission_findings.iter().any(|finding| {
        finding.candidate_ref == "logs/failed-run.log"
            && finding.severity == ContextOmissionSeverity::Blocking
    }));
    assert!(
        advanced_context.patch_safe_edit_attempts.iter().any(|attempt| {
            attempt.target_ref == "src/lib.rs" && !attempt.anchor_refs.is_empty()
        })
    );
}
