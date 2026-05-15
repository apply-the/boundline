use std::fs;

use boundline::FileConfigStore;
use boundline::adapters::config_store::ConfigStoreError;
use boundline::domain::configuration::{
    CapabilityState, ConfigFile, ModelRoute, RouteSlot, RoutingConfig, RuntimeCapabilityProfile,
    RuntimeKind,
};
use boundline::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, PlannedTask,
    WorkspaceSignals,
};
use boundline::domain::trace::TraceSummaryView;
use boundline::{
    CapabilityPhase, FindingConfidence, GoalPlan, GuardianCapability, GuardianDisposition,
    GuardianExecutionRequest, GuardianFinding, GuardianKind, GuidanceAuthoritySource,
    execute_guardians_for_phase, guardian_kind_requires_route, order_guardians_for_execution,
    planning_runtime_evidence, resolve_capabilities_for_phase,
    should_short_circuit_semantic_guards,
};
use uuid::Uuid;

fn sample_task() -> PlannedTask {
    PlannedTask {
        task_id: "task-1".to_string(),
        description: "Implement the bounded change".to_string(),
        target: "src/lib.rs".to_string(),
        expected_outcome: Some("compiles".to_string()),
        decision_type_hint: None,
    }
}

fn sample_context_pack(target: &str) -> ContextPack {
    ContextPack {
        pack_id: "context-pack".to_string(),
        summary: "bounded context".to_string(),
        credibility: ContextPackCredibility::Credible,
        inputs: vec![ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: target.to_string(),
            rationale: "primary target".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec![target.to_string()],
        staleness_reason: None,
    }
}

fn guardian(
    guardian_id: &str,
    kind: GuardianKind,
    authority_source: GuidanceAuthoritySource,
) -> GuardianCapability {
    let (command_ref, instruction_ref) = match kind {
        GuardianKind::Deterministic => (Some("scripts/check.sh".to_string()), None),
        GuardianKind::Hybrid => {
            (Some("scripts/check.sh".to_string()), Some("assistant/prompts/check.md".to_string()))
        }
        GuardianKind::Llm => (None, Some("assistant/prompts/check.md".to_string())),
    };

    GuardianCapability {
        guardian_id: guardian_id.to_string(),
        title: guardian_id.to_string(),
        kind,
        applies_to: vec![CapabilityPhase::Verification],
        rules: vec!["bounded-rule".to_string()],
        severity_floor: GuardianDisposition::Concern,
        command_ref,
        instruction_ref,
        authority_source,
        source_ref: format!("assistant/guardians/{guardian_id}.toml"),
        pack_id: None,
        catalog_pillar: None,
        catalog_default_disposition: None,
        catalog_authority_source: None,
    }
}

fn finding(disposition: GuardianDisposition) -> GuardianFinding {
    GuardianFinding {
        finding_id: format!("finding-{}", disposition.as_str()),
        guardian_id: "guardian-1".to_string(),
        rule_id: "bounded-rule".to_string(),
        disposition,
        summary: "summary".to_string(),
        evidence_refs: vec!["src/lib.rs".to_string()],
        confidence: FindingConfidence::High,
        recommended_action: "fix the issue".to_string(),
        authority_source: GuidanceAuthoritySource::BuiltIn,
        source_ref: "assistant/guardians/guardian-1.toml".to_string(),
        phase: CapabilityPhase::Verification,
    }
}

#[test]
fn goal_plan_and_trace_start_with_empty_guidance_projection() {
    let plan = GoalPlan::new("Implement guidance", vec![sample_task()]).unwrap();
    let trace = TraceSummaryView::default();

    assert!(plan.guidance_guardian.is_empty());
    assert!(trace.guidance_guardian.is_empty());
}

#[test]
fn guardian_order_prefers_deterministic_then_authority() {
    let ordered = order_guardians_for_execution(vec![
        guardian("llm-guardian", GuardianKind::Llm, GuidanceAuthoritySource::BuiltIn),
        guardian(
            "built-in-deterministic",
            GuardianKind::Deterministic,
            GuidanceAuthoritySource::BuiltIn,
        ),
        guardian(
            "workspace-deterministic",
            GuardianKind::Deterministic,
            GuidanceAuthoritySource::WorkspaceOverride,
        ),
    ]);

    assert_eq!(ordered[0].guardian_id, "workspace-deterministic");
    assert_eq!(ordered[1].guardian_id, "built-in-deterministic");
    assert_eq!(ordered[2].guardian_id, "llm-guardian");
}

#[test]
fn blocking_findings_short_circuit_semantic_guardians() {
    let findings = vec![finding(GuardianDisposition::Advise), finding(GuardianDisposition::Block)];

    assert!(should_short_circuit_semantic_guards(&findings));
}

#[test]
fn semantic_guardians_require_existing_routes() {
    assert!(!guardian_kind_requires_route(GuardianKind::Deterministic));
    assert!(guardian_kind_requires_route(GuardianKind::Hybrid));
    assert!(guardian_kind_requires_route(GuardianKind::Llm));
}

#[test]
fn guardian_execution_emits_structured_findings_for_changed_rust_files() {
    let workspace = temp_workspace("guidance-runtime-findings");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { Some(left + right).unwrap() }\n",
    )
    .unwrap();

    let outcome = execute_guardians_for_phase(
        &workspace,
        &GuardianExecutionRequest {
            goal_text: "verify the rust change".to_string(),
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Verification,
            evidence_refs: vec!["src/lib.rs".to_string(), "cargo test --quiet".to_string()],
            changed_files: vec!["src/lib.rs".to_string()],
            workspace_signals: rust_workspace_signals(),
        },
    );

    assert!(
        outcome
            .findings
            .iter()
            .any(|finding| finding.summary.contains("unwrap/expect shortcut detected"))
    );
    assert!(
        outcome
            .projection
            .guardian_blocking_outcome
            .as_deref()
            .is_some_and(|summary| summary.contains("blocking deterministic findings"))
    );
}

#[test]
fn guardian_execution_records_failed_deterministic_guardians_explicitly() {
    let workspace = temp_workspace("guidance-runtime-failure");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/guardians/custom.toml"),
        "[guardians.custom-failure]\ntitle = \"Custom Failure\"\nkind = \"deterministic\"\napplies_to = [\"verification\"]\nrules = [\"custom_failure\"]\nseverity_floor = \"error\"\ncommand = \"builtin:missing\"\n",
    )
    .unwrap();

    let outcome = execute_guardians_for_phase(
        &workspace,
        &GuardianExecutionRequest {
            goal_text: "verify the rust change".to_string(),
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Verification,
            evidence_refs: vec!["src/lib.rs".to_string()],
            changed_files: vec!["src/lib.rs".to_string()],
            workspace_signals: rust_workspace_signals(),
        },
    );

    assert!(outcome.findings.iter().any(|finding| finding.rule_id == "guardian_failure"));
    assert!(
        outcome
            .projection
            .guardian_timeline
            .iter()
            .any(|line| line.contains("custom-failure: failed"))
    );
}

#[test]
fn guardian_execution_degrades_semantic_guards_when_validation_support_is_unavailable() {
    let workspace = temp_workspace("guidance-runtime-degraded");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            verification: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "guardian-reviewer".to_string(),
            }),
            runtime_capabilities: std::iter::once((
                RuntimeKind::Claude,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Unsupported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("semantic validation is intentionally unavailable".to_string()),
                },
            ))
            .collect(),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    save_local_config(&workspace, &config).unwrap();

    let outcome = execute_guardians_for_phase(
        &workspace,
        &GuardianExecutionRequest {
            goal_text: "verify the rust change".to_string(),
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Verification,
            evidence_refs: vec!["src/lib.rs".to_string()],
            changed_files: vec!["src/lib.rs".to_string()],
            workspace_signals: rust_workspace_signals(),
        },
    );

    assert!(
        outcome
            .projection
            .guardian_degradations
            .iter()
            .any(|line| line.contains("validation support is unavailable"))
    );
    assert!(outcome.projection.guardian_timeline.iter().any(|line| {
        line.contains(": degraded (") && line.contains("validation support is unavailable")
    }));
}

#[test]
fn guardian_execution_stages_hybrid_guardians_on_verification_routes() {
    let workspace = temp_workspace("guidance-runtime-hybrid");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/guardians/custom.toml"),
        "[guardians.custom-hybrid]\ntitle = \"Custom Hybrid\"\nkind = \"hybrid\"\napplies_to = [\"verification\"]\nrules = [\"custom_hybrid\"]\nseverity_floor = \"warn\"\ncommand = \"builtin:validation-evidence\"\ninstruction = \"assistant/guardians/testing-evidence.md\"\n",
    )
    .unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            verification: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "guardian-reviewer".to_string(),
            }),
            runtime_capabilities: std::iter::once((
                RuntimeKind::Copilot,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Supported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: None,
                },
            ))
            .collect(),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    save_local_config(&workspace, &config).unwrap();

    let outcome = execute_guardians_for_phase(
        &workspace,
        &GuardianExecutionRequest {
            goal_text: "verify the rust change".to_string(),
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Verification,
            evidence_refs: vec!["src/lib.rs".to_string()],
            changed_files: vec!["src/lib.rs".to_string()],
            workspace_signals: rust_workspace_signals(),
        },
    );

    let execution = outcome
        .executions
        .iter()
        .find(|execution| execution.guardian_id == "custom-hybrid")
        .expect("custom hybrid guardian should execute");
    assert_eq!(execution.route_slot, Some(RouteSlot::Verification));
    assert!(!execution.finding_ids.is_empty());
    assert!(outcome.projection.guardian_timeline.iter().any(|line| {
        line.contains("custom-hybrid: completed (semantic review staged on verification")
    }));
}

#[test]
fn resolve_capabilities_filters_verification_only_guardians_by_phase() {
    let workspace = temp_workspace("guidance-runtime-phase-filter");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();

    let evidence = planning_runtime_evidence(
        "verify the rust change",
        &sample_context_pack("src/lib.rs"),
        &rust_workspace_signals(),
    );
    let planning = resolve_capabilities_for_phase(&workspace, CapabilityPhase::Planning, &evidence);
    let verification =
        resolve_capabilities_for_phase(&workspace, CapabilityPhase::Verification, &evidence);

    assert!(!planning.guardians.iter().any(|guardian| guardian.guardian_id == "testing-evidence"));
    assert!(
        !planning.guardians.iter().any(|guardian| guardian.guardian_id == "rust-language-safety")
    );
    assert!(
        verification.guardians.iter().any(|guardian| guardian.guardian_id == "testing-evidence")
    );
    assert!(
        verification
            .guardians
            .iter()
            .any(|guardian| guardian.guardian_id == "rust-language-safety")
    );
}

fn rust_workspace_signals() -> WorkspaceSignals {
    WorkspaceSignals {
        language: Some("rust".to_string()),
        file_count: 1,
        has_config: false,
        has_canon: false,
        has_tests: true,
    }
}

fn temp_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn save_local_config(
    workspace: &std::path::Path,
    config: &ConfigFile,
) -> Result<(), ConfigStoreError> {
    FileConfigStore::for_workspace(workspace).save_local(config).map(|_| ())
}
