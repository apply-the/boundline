use boundline::domain::goal_plan::PlannedTask;
use boundline::domain::trace::TraceSummaryView;
use boundline::{
    CapabilityPhase, CapabilityResolutionRecord, GoalPlan, GuardianCapability, GuardianDisposition,
    GuardianFinding, GuardianKind, GuidanceAuthoritySource, GuidanceCapability,
    GuidanceGuardianProjection, GuidancePriority, LoadedCapabilitySource, SkippedCapabilitySource,
    planning_runtime_evidence, resolve_capabilities_for_phase,
};
use serde_json::json;

fn sample_task() -> PlannedTask {
    PlannedTask {
        task_id: "task-1".to_string(),
        description: "Plan the bounded change".to_string(),
        target: "src/lib.rs".to_string(),
        expected_outcome: Some("validated".to_string()),
        decision_type_hint: None,
    }
}

#[test]
fn guidance_and_guardian_manifest_contract_uses_expected_fields() {
    let guidance = GuidanceCapability {
        capability_id: "rust.clean-code".to_string(),
        title: "Rust Clean Code".to_string(),
        applies_to: vec![CapabilityPhase::Planning, CapabilityPhase::Implementation],
        roles: vec!["planner".to_string(), "implementer".to_string()],
        content_ref: "assistant/guidance/rust-clean-code.md".to_string(),
        priority: GuidancePriority::High,
        authority_source: GuidanceAuthoritySource::SharedPack,
        source_ref: "assistant/packs/rust-foundations.toml".to_string(),
        pack_id: Some("rust-foundations".to_string()),
        catalog_pillar: None,
        catalog_strength: None,
        catalog_authority_source: None,
    };
    let guardian = GuardianCapability {
        guardian_id: "rust.clean-code.guardian".to_string(),
        title: "Rust Clean Code Guardian".to_string(),
        kind: GuardianKind::Hybrid,
        applies_to: vec![CapabilityPhase::Verification],
        rules: vec!["no_panics".to_string(), "explicit_errors".to_string()],
        severity_floor: GuardianDisposition::Concern,
        command_ref: Some("scripts/check-rust-clean-code.sh".to_string()),
        instruction_ref: Some("assistant/guardians/rust-clean-code.md".to_string()),
        authority_source: GuidanceAuthoritySource::SharedPack,
        source_ref: "assistant/guardians/rust-clean-code.toml".to_string(),
        pack_id: Some("rust-foundations".to_string()),
        catalog_pillar: None,
        catalog_default_disposition: None,
        catalog_authority_source: None,
    };

    let guidance_document = serde_json::to_value(&guidance).unwrap();
    let guardian_document = serde_json::to_value(&guardian).unwrap();

    assert_eq!(guidance_document["capability_id"], json!("rust.clean-code"));
    assert_eq!(guidance_document["content_ref"], json!("assistant/guidance/rust-clean-code.md"));
    assert_eq!(guidance_document["authority_source"], json!("shared_pack"));
    assert_eq!(guardian_document["guardian_id"], json!("rust.clean-code.guardian"));
    assert_eq!(guardian_document["kind"], json!("hybrid"));
    assert_eq!(guardian_document["severity_floor"], json!("concern"));
    assert_eq!(guardian_document["command_ref"], json!("scripts/check-rust-clean-code.sh"));
    assert_eq!(
        guardian_document["instruction_ref"],
        json!("assistant/guardians/rust-clean-code.md")
    );
}

#[test]
fn capability_resolution_contract_preserves_loaded_and_skipped_sources() {
    let record = CapabilityResolutionRecord {
        target_ref: "workspace:.".to_string(),
        phase: CapabilityPhase::Planning,
        loaded_guidance: vec!["rust.clean-code".to_string()],
        loaded_guardians: vec!["rust.clean-code.guardian".to_string()],
        loaded_sources: vec![LoadedCapabilitySource {
            source_ref: "assistant/packs/rust-foundations.toml".to_string(),
            authority_source: GuidanceAuthoritySource::SharedPack,
        }],
        skipped_sources: vec![SkippedCapabilitySource {
            source_ref: ".boundline/guidance/rust-clean-code.md".to_string(),
            authority_source: GuidanceAuthoritySource::WorkspaceOverride,
            reason: "shadowed by runtime evidence".to_string(),
        }],
        loaded_packs: vec!["assistant/packs/guidance-catalog".to_string()],
        skipped_packs: vec![
            "assistant/packs/legacy-pack (missing catalog-manifest.toml)".to_string(),
        ],
        validation_findings: Vec::new(),
        resolution_notes: vec![
            "workspace override skipped because runtime evidence selected the same capability id"
                .to_string(),
        ],
        summary: "1 guidance source and 1 guardian source resolved".to_string(),
    };

    let document = serde_json::to_value(&record).unwrap();

    assert_eq!(document["target_ref"], json!("workspace:."));
    assert_eq!(document["phase"], json!("planning"));
    assert_eq!(document["loaded_sources"][0]["authority_source"], json!("shared_pack"));
    assert_eq!(document["skipped_sources"][0]["reason"], json!("shadowed by runtime evidence"));
    assert_eq!(document["summary"], json!("1 guidance source and 1 guardian source resolved"));
}

#[test]
fn goal_plan_and_trace_flatten_guidance_projection_fields() {
    let finding = GuardianFinding {
        finding_id: "finding-1".to_string(),
        guardian_id: "rust.clean-code.guardian".to_string(),
        rule_id: "no_panics".to_string(),
        disposition: GuardianDisposition::Warn,
        summary: "panic-prone flow detected".to_string(),
        evidence_refs: vec!["src/domain/guidance.rs".to_string()],
        confidence: boundline::FindingConfidence::High,
        recommended_action: "replace panic-prone flow with explicit error propagation".to_string(),
        authority_source: GuidanceAuthoritySource::SharedPack,
        source_ref: "assistant/guardians/rust-clean-code.toml".to_string(),
        phase: CapabilityPhase::Verification,
    };
    let projection = GuidanceGuardianProjection {
        capability_resolution_summary: Some(
            "guidance resolved for planning and verification".to_string(),
        ),
        loaded_packs: vec!["assistant/packs/guidance-catalog".to_string()],
        skipped_packs: Vec::new(),
        catalog_validation_findings: Vec::new(),
        loaded_guidance_sources: vec!["assistant/guidance/rust-clean-code.md".to_string()],
        skipped_guidance_sources: vec![".boundline/guidance/legacy.md (shadowed)".to_string()],
        loaded_guardian_sources: vec!["assistant/guardians/rust-clean-code.toml".to_string()],
        skipped_guardian_sources: vec![".boundline/guardians/legacy.toml (disabled)".to_string()],
        guardian_timeline: vec!["deterministic: completed".to_string()],
        guardian_findings_summary: Some("1 warning from shared_pack guardians".to_string()),
        guardian_findings: vec![finding],
        guardian_degradations: vec!["llm guardian skipped: no verification route".to_string()],
        guardian_blocking_outcome: Some("no blocking guardians triggered".to_string()),
    };

    let mut plan = GoalPlan::new("Plan with guidance", vec![sample_task()]).unwrap();
    plan.guidance_guardian = projection.clone();
    let plan_document = serde_json::to_value(&plan).unwrap();

    assert_eq!(
        plan_document["capability_resolution_summary"],
        json!("guidance resolved for planning and verification")
    );
    assert_eq!(
        plan_document["loaded_guidance_sources"][0],
        json!("assistant/guidance/rust-clean-code.md")
    );
    assert_eq!(
        plan_document["guardian_findings_summary"],
        json!("1 warning from shared_pack guardians")
    );

    let summary = TraceSummaryView {
        trace_ref: "trace-1".to_string(),
        goal: "Plan with guidance".to_string(),
        guidance_guardian: projection,
        ..TraceSummaryView::default()
    };
    let trace_document = serde_json::to_value(&summary).unwrap();

    assert_eq!(
        trace_document["guardian_degradations"][0],
        json!("llm guardian skipped: no verification route")
    );
    assert_eq!(
        trace_document["guardian_blocking_outcome"],
        json!("no blocking guardians triggered")
    );
    assert_eq!(trace_document["guardian_findings"][0]["rule_id"], json!("no_panics"));
}

#[test]
fn workspace_override_precedence_is_disclosed_in_resolution_projection() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-guidance-contract-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(workspace.join(".boundline/guidance")).unwrap();
    std::fs::write(
        workspace.join(".boundline/guidance/clean-code.md"),
        "# Workspace Clean Code\nPrefer the workspace-specific rule set.\n",
    )
    .unwrap();

    let context_pack = boundline::domain::goal_plan::ContextPack {
        pack_id: "context-pack".to_string(),
        summary: "bounded guidance context".to_string(),
        credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
        inputs: vec![boundline::domain::goal_plan::ContextInput {
            kind: boundline::domain::goal_plan::ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "bounded target".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/lib.rs".to_string()],
        advanced_context: None,
        staleness_reason: None,
    };
    let signals = boundline::domain::goal_plan::WorkspaceSignals {
        language: Some("rust".to_string()),
        file_count: 1,
        has_config: true,
        has_canon: false,
        has_tests: true,
    };

    let resolution = resolve_capabilities_for_phase(
        &workspace,
        CapabilityPhase::Planning,
        &planning_runtime_evidence(
            "Use the workspace clean-code override",
            &context_pack,
            &signals,
        ),
    );

    assert!(
        resolution
            .projection
            .loaded_guidance_sources
            .iter()
            .any(|source| source == ".boundline/guidance/clean-code.md")
    );
    assert!(
        resolution
            .projection
            .skipped_guidance_sources
            .iter()
            .any(|source| source.contains("assistant/packs/engineering-foundations.toml")
                && source.contains("shadowed"))
    );
}

#[test]
fn invalid_workspace_guardian_override_is_reported_as_skipped_source() {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-guidance-invalid-guardian-contract-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
    std::fs::write(
        workspace.join(".boundline/guardians/invalid.toml"),
        "[guardians.invalid\nkind = \"deterministic\"\n",
    )
    .unwrap();

    let context_pack = boundline::domain::goal_plan::ContextPack {
        pack_id: "context-pack".to_string(),
        summary: "bounded verification context".to_string(),
        credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
        inputs: vec![boundline::domain::goal_plan::ContextInput {
            kind: boundline::domain::goal_plan::ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "bounded target".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/lib.rs".to_string()],
        advanced_context: None,
        staleness_reason: None,
    };
    let signals = boundline::domain::goal_plan::WorkspaceSignals {
        language: Some("rust".to_string()),
        file_count: 1,
        has_config: true,
        has_canon: false,
        has_tests: true,
    };

    let resolution = resolve_capabilities_for_phase(
        &workspace,
        CapabilityPhase::Verification,
        &planning_runtime_evidence("verify the rust change", &context_pack, &signals),
    );

    assert!(
        resolution
            .projection
            .skipped_guardian_sources
            .iter()
            .any(|source| source.contains(".boundline/guardians/invalid.toml")
                && source.contains("failed to parse workspace guardian override"))
    );
}

#[test]
fn bundled_capability_packs_cover_multiple_technology_clusters() {
    let packs_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assistant/packs");
    let mut entries = std::fs::read_dir(packs_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    entries.sort();

    for expected in [
        "guidance-catalog",
        "engineering-foundations.toml",
        "rust-delivery.toml",
        "javascript-typescript-delivery.toml",
        "python-delivery.toml",
        "jvm-delivery.toml",
        "dotnet-delivery.toml",
        "go-delivery.toml",
        "php-ruby-delivery.toml",
        "mobile-delivery.toml",
        "systems-delivery.toml",
        "shell-automation-delivery.toml",
    ] {
        assert!(
            entries.iter().any(|entry| entry == expected),
            "missing pack {expected}; found {entries:?}"
        );
    }
}

#[test]
fn javascript_workspace_prefers_javascript_delivery_pack() {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-guidance-javascript-contract-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("package.json"),
        "{\n  \"name\": \"bounded-js\",\n  \"private\": true\n}\n",
    )
    .unwrap();
    std::fs::write(
        workspace.join("src/app.tsx"),
        "export function App() {\n  return <main>Hello</main>;\n}\n",
    )
    .unwrap();

    let context_pack = boundline::domain::goal_plan::ContextPack {
        pack_id: "context-pack".to_string(),
        summary: "bounded javascript context".to_string(),
        credibility: boundline::domain::goal_plan::ContextPackCredibility::Credible,
        inputs: vec![boundline::domain::goal_plan::ContextInput {
            kind: boundline::domain::goal_plan::ContextInputKind::WorkspaceFile,
            reference: "src/app.tsx".to_string(),
            rationale: "explicit frontend target".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/app.tsx".to_string()],
        advanced_context: None,
        staleness_reason: None,
    };
    let signals = boundline::domain::goal_plan::WorkspaceSignals {
        language: Some("javascript".to_string()),
        file_count: 2,
        has_config: false,
        has_canon: false,
        has_tests: true,
    };

    let resolution = resolve_capabilities_for_phase(
        &workspace,
        CapabilityPhase::Planning,
        &planning_runtime_evidence(
            "Improve the React page and keep testing confidence high",
            &context_pack,
            &signals,
        ),
    );

    assert!(
        resolution
            .projection
            .loaded_guidance_sources
            .iter()
            .any(|source| source == "assistant/packs/javascript-typescript-delivery.toml")
    );
    assert!(
        !resolution
            .projection
            .loaded_guidance_sources
            .iter()
            .any(|source| source == "assistant/packs/rust-delivery.toml")
    );
}
