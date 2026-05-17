use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{
    RetrievalCompatibilityState, RetrievalSourceKind, SemanticCapabilityState,
    SemanticTraceEventKind,
};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use boundline::domain::governance::{
    CanonSemanticEligibilityState, CanonSemanticProvenanceBoundary,
    SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE,
};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection,
};
use serde_json::json;
use uuid::Uuid;

const SEMANTIC_VECTOR_STATE_OVERRIDE_ENV: &str = "BOUNDLINE_SEMANTIC_VECTOR_STATE_OVERRIDE";
const SEMANTIC_VECTOR_STATE_READY_VALUE: &str = "ready";
const CANON_INDEX_CONTRACT_LINE_V1: &str = "v1";

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

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn write_workspace_file(workspace: &Path, relative_path: &str, contents: &str) {
    let path = workspace.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn write_canon_semantic_artifact(
    workspace: &Path,
    relative_path: &str,
    contents: &str,
    semantic_eligibility: CanonSemanticEligibilityState,
    semantic_exclusion_reason: Option<&str>,
) {
    let artifact_path = workspace.join(relative_path);
    if let Some(parent) = artifact_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&artifact_path, contents).unwrap();

    let file_stem = artifact_path.file_stem().and_then(|value| value.to_str()).unwrap();
    let sidecar_path = artifact_path.with_file_name(format!("{file_stem}.packet-metadata.json"));
    let provenance_ref = format!("{relative_path}#section:overview");
    let sidecar = json!({
        "lineage": {
            "contract_version": CANON_INDEX_CONTRACT_LINE_V1,
            "source_run": "canon:packet:planner-guidance",
            "promotion_state": "auto"
        },
        "publication_target_class": "stable",
        "semantic_descriptor": {
            "semantic_contract_line": SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE,
            "semantic_eligibility": semantic_eligibility,
            "semantic_provenance_boundary": CanonSemanticProvenanceBoundary::Section,
            "semantic_provenance_ref": provenance_ref,
            "semantic_labels": ["planner", "guided-boundary"],
            "semantic_exclusion_reason": semantic_exclusion_reason
        }
    });
    fs::write(sidecar_path, serde_json::to_string_pretty(&sidecar).unwrap()).unwrap();
}

#[test]
fn build_projection_accepts_compatible_canon_artifacts_and_surfaces_explicit_skip_reasons() {
    let _guard =
        SEMANTIC_VECTOR_STATE_OVERRIDE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard =
        set_env_var(SEMANTIC_VECTOR_STATE_OVERRIDE_ENV, SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = temp_workspace("boundline-context-intelligence-canon-semantic-flow");

    write_workspace_file(
        &workspace,
        "src/lib.rs",
        concat!("pub fn planner() -> bool {\n", "    true\n", "}\n"),
    );
    write_canon_semantic_artifact(
        &workspace,
        ".canon/planner-guidance.md",
        "# Planner Guidance\n\nPrefer the guided boundary for planner reconciliation.\n",
        CanonSemanticEligibilityState::Eligible,
        None,
    );
    write_canon_semantic_artifact(
        &workspace,
        ".canon/excluded-guidance.md",
        "# Excluded Guidance\n\nLegacy planner guidance should stay out of semantic retrieval.\n",
        CanonSemanticEligibilityState::Excluded,
        Some("excluded by Canon semantic policy"),
    );

    let projection = build_advanced_context_projection(
        "planner guided boundary reconciliation",
        &workspace,
        &[
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "selected implementation surface".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            },
            ContextInput {
                kind: ContextInputKind::CanonArtifact,
                reference: ".canon/planner-guidance.md".to_string(),
                rationale: "compatible Canon semantic guidance".to_string(),
                source: "canon_scan".to_string(),
                primary: false,
            },
            ContextInput {
                kind: ContextInputKind::CanonArtifact,
                reference: ".canon/excluded-guidance.md".to_string(),
                rationale: "excluded Canon semantic guidance".to_string(),
                source: "canon_scan".to_string(),
                primary: false,
            },
        ],
        &["src/lib.rs".to_string(), ".canon/planner-guidance.md".to_string()],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    assert_eq!(projection.semantic_capability_state, SemanticCapabilityState::Ready);
    assert!(projection.selected_evidence.iter().any(|candidate| {
        candidate.source_kind == RetrievalSourceKind::CanonArtifact
            && candidate.source_ref == ".canon/planner-guidance.md"
            && candidate.canon_semantic_contract_line.as_deref() == Some("v1")
            && candidate.canon_semantic_provenance_ref.as_deref()
                == Some(".canon/planner-guidance.md#section:overview")
    }));
    assert!(
        !projection
            .selected_evidence
            .iter()
            .any(|candidate| { candidate.source_ref == ".canon/excluded-guidance.md" })
    );

    let skipped_candidate = projection
        .rejected_candidates
        .iter()
        .find(|candidate| candidate.source_ref == ".canon/excluded-guidance.md")
        .expect("excluded Canon artifact rejected");
    assert_eq!(skipped_candidate.compatibility_state, RetrievalCompatibilityState::PolicyBlocked);
    assert!(skipped_candidate.selection_reason.contains("excluded by Canon semantic policy"));

    let skipped_trace = projection
        .semantic_trace_records
        .iter()
        .find(|record| {
            record.event_kind == SemanticTraceEventKind::CanonArtifactSkipped
                && record.candidate_ref.as_deref() == Some(".canon/excluded-guidance.md")
        })
        .expect("excluded Canon artifact trace record");
    assert_eq!(skipped_trace.canon_artifact_class.as_deref(), Some("stable"));
    assert_eq!(skipped_trace.canon_semantic_contract_line.as_deref(), Some("v1"));
    assert_eq!(
        skipped_trace.canon_semantic_provenance_boundary,
        Some(CanonSemanticProvenanceBoundary::Section)
    );
    assert_eq!(
        skipped_trace.canon_semantic_provenance_ref.as_deref(),
        Some(".canon/excluded-guidance.md#section:overview")
    );
    assert!(skipped_trace.reason.contains("excluded by Canon semantic policy"));
}
