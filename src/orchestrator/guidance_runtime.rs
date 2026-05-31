//! Capability discovery, precedence resolution, and bounded guardian execution.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::adapters::config_store::FileConfigStore;
use crate::domain::configuration::{
    RouteSlot, RoutingOverrides, resolve_effective_routing, resolve_effective_runtime_capabilities,
};
use crate::domain::goal_plan::{ContextPack, WorkspaceSignals};
use crate::domain::guidance::{
    CapabilityPhase, CapabilityResolutionRecord, FindingConfidence, GuardianCapability,
    GuardianDisposition, GuardianExecutionRecord, GuardianExecutionState, GuardianFinding,
    GuardianKind, GuidanceAuthoritySource, GuidanceCapability, GuidanceGuardianProjection,
    GuidancePriority, LoadedCapabilitySource, SkippedCapabilitySource,
};
use crate::orchestrator::guidance_catalog_runtime::{CatalogPackDiscovery, discover_catalog_packs};
use serde::Deserialize;

const BUNDLED_ASSISTANT_DIR: &str = "assistant";
const BUNDLED_PACKS_DIR: &str = "assistant/packs";
const ASSISTANT_ROOT_OVERRIDE_ENV: &str = "BOUNDLINE_ASSISTANT_ROOT";
const WORKSPACE_GUIDANCE_DIR: &str = ".boundline/guidance";
const WORKSPACE_GUARDIANS_DIR: &str = ".boundline/guardians";
const CANON_GUIDANCE_DIR: &str = ".canon/boundline/guidance";
const DEFAULT_WORKSPACE_GUIDANCE_ROLES: &[&str] =
    &["planner", "implementer", "verifier", "reviewer"];
const MAX_GUARDIANS_PER_PHASE: usize = 4;
const MAX_SEMANTIC_GUARDIANS_PER_PHASE: usize = 2;
const GUARDIAN_TIMEOUT: Duration = Duration::from_secs(30);

/// Runtime hints derived from the active goal, selected targets, and workspace
/// signals to rank guidance candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuidanceRuntimeEvidence {
    pub goal_text: String,
    pub language: Option<String>,
    pub selected_targets: Vec<String>,
    pub primary_inputs: Vec<String>,
    pub has_tests: bool,
}

/// Resolved capabilities plus the persisted record and flattened projection
/// reused by `plan`, `status`, and `inspect` surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityResolution {
    pub guidance: Vec<GuidanceCapability>,
    pub guardians: Vec<GuardianCapability>,
    pub record: CapabilityResolutionRecord,
    pub projection: GuidanceGuardianProjection,
}

/// Inputs required to execute guardians for one target in one lifecycle phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardianExecutionRequest {
    pub goal_text: String,
    pub target_ref: String,
    pub phase: CapabilityPhase,
    pub evidence_refs: Vec<String>,
    pub changed_files: Vec<String>,
    pub workspace_signals: WorkspaceSignals,
}

/// Full guardian result, including execution records, findings, and the
/// read-side projection consumed by operator surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardianExecutionOutcome {
    pub executions: Vec<GuardianExecutionRecord>,
    pub findings: Vec<GuardianFinding>,
    pub projection: GuidanceGuardianProjection,
}

#[derive(Debug, Clone, Deserialize)]
struct CapabilityPackManifest {
    #[serde(default)]
    guidance: BTreeMap<String, GuidanceManifestEntry>,
    #[serde(default)]
    guardians: BTreeMap<String, GuardianManifestEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct GuidanceManifestEntry {
    title: String,
    applies_to: Vec<CapabilityPhase>,
    #[serde(default)]
    roles: Vec<String>,
    path: String,
    priority: GuidancePriority,
}

#[derive(Debug, Clone, Deserialize)]
struct GuardianManifestEntry {
    title: String,
    kind: GuardianKind,
    applies_to: Vec<CapabilityPhase>,
    rules: Vec<String>,
    severity_floor: GuardianDisposition,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    instruction: Option<String>,
}

/// Builds the evidence bundle used to rank planning-phase guidance from the
/// current context pack and workspace signals.
pub fn planning_runtime_evidence(
    goal_text: &str,
    context_pack: &ContextPack,
    signals: &WorkspaceSignals,
) -> GuidanceRuntimeEvidence {
    GuidanceRuntimeEvidence {
        goal_text: goal_text.to_string(),
        language: signals.language.clone(),
        selected_targets: context_pack.selected_targets.clone(),
        primary_inputs: context_pack
            .inputs
            .iter()
            .filter(|input| input.primary)
            .map(|input| input.reference.clone())
            .collect(),
        has_tests: signals.has_tests,
    }
}

/// Resolves every candidate source for one lifecycle phase, then collapses
/// duplicate capability ids by authority first and runtime-evidence score second
/// so the persisted record can explain both loaded and skipped sources.
pub fn resolve_capabilities_for_phase(
    workspace_ref: &Path,
    phase: CapabilityPhase,
    evidence: &GuidanceRuntimeEvidence,
) -> CapabilityResolution {
    let mut skipped_sources = Vec::new();
    let mut skipped_guidance_sources = Vec::new();
    let mut skipped_guardian_sources = Vec::new();

    let pack_discovery = discover_pack_capabilities(phase);
    let pack_guidance = pack_discovery.guidance.clone();
    let pack_guardians = pack_discovery.guardians.clone();
    let mut pack_skips = pack_discovery.skipped_sources.clone();
    skipped_guidance_sources.extend(pack_skips.clone());
    skipped_sources.append(&mut pack_skips);

    let (workspace_guidance, mut workspace_guidance_skips) =
        discover_workspace_guidance(workspace_ref, phase);
    skipped_guidance_sources.extend(workspace_guidance_skips.clone());
    skipped_sources.append(&mut workspace_guidance_skips);

    let (workspace_guardians, mut workspace_guardian_skips) =
        discover_workspace_guardians(workspace_ref, phase);
    skipped_guardian_sources.extend(workspace_guardian_skips.clone());
    skipped_sources.append(&mut workspace_guardian_skips);

    let (canon_guidance, mut canon_skips) =
        discover_optional_canon_guidance(workspace_ref, phase, &pack_guidance);
    skipped_guidance_sources.extend(canon_skips.clone());
    skipped_sources.append(&mut canon_skips);

    let (guidance, mut precedence_skips) = resolve_guidance_candidates(
        workspace_guidance
            .into_iter()
            .chain(canon_guidance)
            .chain(pack_guidance)
            .collect::<Vec<_>>(),
        evidence,
    );
    skipped_guidance_sources.extend(precedence_skips.clone());
    skipped_sources.append(&mut precedence_skips);

    let guardians = order_guardians_for_execution(
        workspace_guardians.into_iter().chain(pack_guardians).collect::<Vec<_>>(),
    );

    let loaded_guidance =
        guidance.iter().map(|capability| capability.capability_id.clone()).collect::<Vec<_>>();
    let loaded_guardians =
        guardians.iter().map(|guardian| guardian.guardian_id.clone()).collect::<Vec<_>>();
    let loaded_guidance_sources = sort_sources_by_authority(unique_sources(
        guidance.iter().map(|capability| capability.source_ref.as_str()).collect::<Vec<_>>(),
    ));
    let loaded_guardian_sources = sort_sources_by_authority(unique_sources(
        guardians.iter().map(|guardian| guardian.source_ref.as_str()).collect::<Vec<_>>(),
    ));

    let summary = match (loaded_guidance.len(), loaded_guidance_sources.len()) {
        (0, _) => format!("no guidance sources resolved for {}", phase.as_str()),
        (guidance_count, source_count) => format!(
            "resolved {guidance_count} guidance capability entries from {source_count} source(s) for {}",
            phase.as_str()
        ),
    };

    let mut resolution_notes = pack_discovery.resolution_notes.clone();
    if !pack_discovery.validation_findings.is_empty() {
        resolution_notes.push(format!(
            "{} catalog validation finding(s) were recorded during pack discovery",
            pack_discovery.validation_findings.len()
        ));
    }
    if guidance
        .iter()
        .any(|capability| capability.authority_source == GuidanceAuthoritySource::CanonGoverned)
    {
        resolution_notes.push(
            "canon-governed guidance superseded lower-authority catalog-pack guidance where matching capability ids were present"
                .to_string(),
        );
    }

    let record = CapabilityResolutionRecord {
        target_ref: evidence
            .selected_targets
            .first()
            .cloned()
            .unwrap_or_else(|| "workspace:.".to_string()),
        phase,
        loaded_guidance,
        loaded_guardians,
        loaded_sources: loaded_guidance_sources
            .iter()
            .chain(loaded_guardian_sources.iter())
            .map(|source_ref| LoadedCapabilitySource {
                source_ref: source_ref.clone(),
                authority_source: authority_for_source(source_ref),
            })
            .collect(),
        skipped_sources,
        loaded_packs: pack_discovery.loaded_packs.clone(),
        skipped_packs: pack_discovery.skipped_packs.clone(),
        validation_findings: pack_discovery.validation_findings.clone(),
        resolution_notes,
        summary: summary.clone(),
    };

    let projection = GuidanceGuardianProjection {
        capability_resolution_summary: Some(summary),
        loaded_packs: pack_discovery.loaded_packs.clone(),
        skipped_packs: pack_discovery.skipped_packs.clone(),
        catalog_validation_findings: pack_discovery
            .validation_findings
            .iter()
            .map(|finding| finding.display_line())
            .collect(),
        loaded_guidance_sources,
        skipped_guidance_sources: skipped_guidance_sources
            .iter()
            .map(skipped_source_line)
            .collect(),
        loaded_guardian_sources,
        skipped_guardian_sources: skipped_guardian_sources
            .iter()
            .map(skipped_source_line)
            .collect(),
        guardian_timeline: guardians
            .iter()
            .map(|guardian| {
                format!(
                    "{}: planned {} guardian from {}",
                    guardian.guardian_id,
                    guardian.kind.as_str(),
                    guardian.source_ref
                )
            })
            .collect(),
        guardian_findings_summary: None,
        guardian_findings: Vec::new(),
        guardian_degradations: Vec::new(),
        guardian_blocking_outcome: None,
    };

    CapabilityResolution { guidance, guardians, record, projection }
}

/// Executes guardians for one phase using deterministic-first ordering,
/// explicit semantic degradation when routing is unavailable, and a flattened
/// projection suitable for persisted status and inspect output.
pub fn execute_guardians_for_phase(
    workspace_ref: &Path,
    request: &GuardianExecutionRequest,
) -> GuardianExecutionOutcome {
    let evidence = GuidanceRuntimeEvidence {
        goal_text: request.goal_text.clone(),
        language: request.workspace_signals.language.clone(),
        selected_targets: vec![request.target_ref.clone()],
        primary_inputs: request.evidence_refs.clone(),
        has_tests: request.workspace_signals.has_tests,
    };
    let resolution = resolve_capabilities_for_phase(workspace_ref, request.phase, &evidence);
    let mut projection = resolution.projection.clone();
    let mut executions = Vec::new();
    let mut findings = Vec::new();
    let mut guardian_timeline = Vec::new();
    let mut guardian_degradations = Vec::new();
    let mut semantic_guardians_seen = 0usize;

    for (index, guardian) in resolution.guardians.iter().enumerate() {
        if index >= MAX_GUARDIANS_PER_PHASE {
            executions.push(GuardianExecutionRecord {
                guardian_id: guardian.guardian_id.clone(),
                phase: request.phase,
                execution_state: GuardianExecutionState::Skipped,
                route_slot: route_slot_for_phase(request.phase),
                evidence_refs: request.evidence_refs.clone(),
                finding_ids: Vec::new(),
                degradation_reason: None,
            });
            guardian_timeline.push(format!(
                "{}: skipped because the per-phase guardian limit ({MAX_GUARDIANS_PER_PHASE}) was reached",
                guardian.guardian_id
            ));
            continue;
        }

        if guardian_kind_requires_route(guardian.kind) {
            if should_short_circuit_semantic_guards(&findings) {
                executions.push(GuardianExecutionRecord {
                    guardian_id: guardian.guardian_id.clone(),
                    phase: request.phase,
                    execution_state: GuardianExecutionState::Skipped,
                    route_slot: route_slot_for_phase(request.phase),
                    evidence_refs: request.evidence_refs.clone(),
                    finding_ids: Vec::new(),
                    degradation_reason: None,
                });
                guardian_timeline.push(format!(
                    "{}: skipped after blocking deterministic findings",
                    guardian.guardian_id
                ));
                continue;
            }

            semantic_guardians_seen += 1;
            if semantic_guardians_seen > MAX_SEMANTIC_GUARDIANS_PER_PHASE {
                executions.push(GuardianExecutionRecord {
                    guardian_id: guardian.guardian_id.clone(),
                    phase: request.phase,
                    execution_state: GuardianExecutionState::Skipped,
                    route_slot: route_slot_for_phase(request.phase),
                    evidence_refs: request.evidence_refs.clone(),
                    finding_ids: Vec::new(),
                    degradation_reason: None,
                });
                guardian_timeline.push(format!(
                    "{}: skipped because the semantic guardian limit ({MAX_SEMANTIC_GUARDIANS_PER_PHASE}) was reached",
                    guardian.guardian_id
                ));
                continue;
            }
        }

        let started_at = Instant::now();
        let route_slot = route_slot_for_phase(request.phase);
        let evaluation = if guardian_kind_requires_route(guardian.kind) {
            match semantic_route_availability(workspace_ref, request.phase) {
                SemanticRouteAvailability::Available(slot) => {
                    evaluate_semantic_guardian(guardian, slot)
                }
                SemanticRouteAvailability::Unavailable { slot, reason } => {
                    GuardianEvaluation::Degraded { route_slot: slot, reason }
                }
            }
        } else {
            evaluate_deterministic_guardian(workspace_ref, guardian, request)
        };

        if started_at.elapsed() > GUARDIAN_TIMEOUT {
            let reason = format!(
                "guardian {} exceeded the bounded timeout of {}s",
                guardian.guardian_id,
                GUARDIAN_TIMEOUT.as_secs()
            );
            guardian_degradations.push(reason.clone());
            guardian_timeline.push(format!("{}: degraded ({reason})", guardian.guardian_id));
            executions.push(GuardianExecutionRecord {
                guardian_id: guardian.guardian_id.clone(),
                phase: request.phase,
                execution_state: GuardianExecutionState::Degraded,
                route_slot,
                evidence_refs: request.evidence_refs.clone(),
                finding_ids: Vec::new(),
                degradation_reason: Some(reason),
            });
            continue;
        }

        match evaluation {
            GuardianEvaluation::Completed { route_slot, new_findings, summary } => {
                let finding_ids = new_findings
                    .iter()
                    .map(|finding| finding.finding_id.clone())
                    .collect::<Vec<_>>();
                findings.extend(new_findings.clone());
                executions.push(GuardianExecutionRecord {
                    guardian_id: guardian.guardian_id.clone(),
                    phase: request.phase,
                    execution_state: GuardianExecutionState::Completed,
                    route_slot,
                    evidence_refs: request.evidence_refs.clone(),
                    finding_ids,
                    degradation_reason: None,
                });
                guardian_timeline.push(format!("{}: completed ({summary})", guardian.guardian_id));
            }
            GuardianEvaluation::Failed { route_slot, finding, reason } => {
                executions.push(GuardianExecutionRecord {
                    guardian_id: guardian.guardian_id.clone(),
                    phase: request.phase,
                    execution_state: GuardianExecutionState::Failed,
                    route_slot,
                    evidence_refs: request.evidence_refs.clone(),
                    finding_ids: vec![finding.finding_id.clone()],
                    degradation_reason: Some(reason.clone()),
                });
                guardian_timeline.push(format!("{}: failed ({reason})", guardian.guardian_id));
                findings.push(finding);
            }
            GuardianEvaluation::Degraded { route_slot, reason } => {
                guardian_degradations.push(reason.clone());
                executions.push(GuardianExecutionRecord {
                    guardian_id: guardian.guardian_id.clone(),
                    phase: request.phase,
                    execution_state: GuardianExecutionState::Degraded,
                    route_slot: Some(route_slot),
                    evidence_refs: request.evidence_refs.clone(),
                    finding_ids: Vec::new(),
                    degradation_reason: Some(reason.clone()),
                });
                guardian_timeline.push(format!("{}: degraded ({reason})", guardian.guardian_id));
            }
        }
    }

    let blocking_outcome = blocking_outcome_text(&findings);
    projection.guardian_timeline = guardian_timeline;
    projection.guardian_findings = findings.clone();
    projection.guardian_findings_summary = guardian_findings_summary(&findings);
    projection.guardian_degradations = guardian_degradations;
    projection.guardian_blocking_outcome = blocking_outcome;

    GuardianExecutionOutcome { executions, findings, projection }
}

/// Compares authority precedence. `Ordering::Less` means the left source wins.
pub fn compare_authority_precedence(
    left: GuidanceAuthoritySource,
    right: GuidanceAuthoritySource,
) -> Ordering {
    left.precedence_rank().cmp(&right.precedence_rank())
}

/// Orders guardians for execution with deterministic checks first.
/// Within the same execution kind, higher-authority sources run first so
/// overrides can replace shared packs without changing the bounded model.
pub fn order_guardians_for_execution(
    guardians: impl IntoIterator<Item = GuardianCapability>,
) -> Vec<GuardianCapability> {
    let mut ordered: Vec<GuardianCapability> = guardians.into_iter().collect();
    ordered.sort_by(|left, right| {
        left.kind
            .execution_rank()
            .cmp(&right.kind.execution_rank())
            .then(compare_authority_precedence(left.authority_source, right.authority_source))
            .then(left.guardian_id.cmp(&right.guardian_id))
    });
    ordered
}

/// Returns true when any finding should block follow-through work.
pub fn has_blocking_findings(findings: &[GuardianFinding]) -> bool {
    findings.iter().any(|finding| finding.disposition.is_blocking())
}

/// Returns true when semantic guardians should be skipped because deterministic
/// checks already produced blocking findings.
pub fn should_short_circuit_semantic_guards(findings: &[GuardianFinding]) -> bool {
    findings.iter().any(|finding| {
        matches!(finding.disposition, GuardianDisposition::Error | GuardianDisposition::Block)
    })
}

/// Returns true when a guardian needs an LLM-backed route slot to run.
pub fn guardian_kind_requires_route(kind: GuardianKind) -> bool {
    matches!(kind, GuardianKind::Hybrid | GuardianKind::Llm)
}

fn resolve_guidance_candidates(
    candidates: Vec<GuidanceCapability>,
    evidence: &GuidanceRuntimeEvidence,
) -> (Vec<GuidanceCapability>, Vec<SkippedCapabilitySource>) {
    let mut grouped = BTreeMap::<String, Vec<(GuidanceCapability, usize)>>::new();
    for capability in candidates {
        let score = guidance_relevance_score(&capability, evidence);
        grouped.entry(capability.capability_id.clone()).or_default().push((capability, score));
    }

    let mut resolved = Vec::new();
    let mut skipped = Vec::new();

    for (capability_id, mut candidates) in grouped {
        candidates.sort_by(|(left, left_score), (right, right_score)| {
            compare_authority_precedence(left.authority_source, right.authority_source)
                .then(right_score.cmp(left_score))
                .then(right.priority.cmp(&left.priority))
                .then(left.source_ref.cmp(&right.source_ref))
        });

        let Some((winner, winner_score)) = candidates.first().cloned() else {
            continue;
        };

        for (candidate, _) in candidates.into_iter().skip(1) {
            skipped.push(SkippedCapabilitySource {
                source_ref: candidate.source_ref.clone(),
                authority_source: candidate.authority_source,
                reason: format!(
                    "shadowed by {} from {} for {} (runtime-evidence score {})",
                    winner.capability_id, winner.source_ref, capability_id, winner_score
                ),
            });
        }

        resolved.push((winner, winner_score));
    }

    resolved.sort_by(|(left, left_score), (right, right_score)| {
        right_score
            .cmp(left_score)
            .then(right.priority.cmp(&left.priority))
            .then(compare_authority_precedence(left.authority_source, right.authority_source))
            .then(left.capability_id.cmp(&right.capability_id))
    });

    (resolved.into_iter().map(|(capability, _)| capability).collect(), skipped)
}

fn discover_pack_capabilities(phase: CapabilityPhase) -> CatalogPackDiscovery {
    let bundled_root = bundled_assistant_root();
    let packs_dir = bundled_root.join(BUNDLED_PACKS_DIR);
    let mut discovery = discover_catalog_packs(
        &packs_dir,
        &bundled_root,
        GuidanceAuthoritySource::SharedPack,
        phase,
    );

    let Ok(entries) = fs::read_dir(&packs_dir) else {
        discovery.skipped_sources.push(SkippedCapabilitySource {
            source_ref: BUNDLED_PACKS_DIR.to_string(),
            authority_source: GuidanceAuthoritySource::SharedPack,
            reason: "no bundled capability packs were available".to_string(),
        });
        return discovery;
    };

    let mut manifest_paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("toml"))
        .collect::<Vec<_>>();
    manifest_paths.sort();

    for manifest_path in manifest_paths {
        let source_ref = display_relative_path(&bundled_root, &manifest_path);
        let pack_id =
            manifest_path.file_stem().and_then(|value| value.to_str()).map(ToOwned::to_owned);

        let Ok(contents) = fs::read_to_string(&manifest_path) else {
            discovery.skipped_sources.push(SkippedCapabilitySource {
                source_ref,
                authority_source: GuidanceAuthoritySource::SharedPack,
                reason: "failed to read bundled capability manifest".to_string(),
            });
            continue;
        };

        let Ok(manifest) = toml::from_str::<CapabilityPackManifest>(&contents) else {
            discovery.skipped_sources.push(SkippedCapabilitySource {
                source_ref,
                authority_source: GuidanceAuthoritySource::SharedPack,
                reason: "failed to parse bundled capability manifest".to_string(),
            });
            continue;
        };

        let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

        for (capability_id, entry) in manifest.guidance {
            if !entry.applies_to.contains(&phase) {
                continue;
            }
            let content_ref = display_relative_path(&bundled_root, &manifest_dir.join(&entry.path));
            discovery.guidance.push(GuidanceCapability {
                capability_id,
                title: entry.title,
                applies_to: entry.applies_to,
                roles: entry.roles,
                content_ref,
                priority: entry.priority,
                authority_source: GuidanceAuthoritySource::SharedPack,
                source_ref: source_ref.clone(),
                pack_id: pack_id.clone(),
                catalog_pillar: None,
                catalog_strength: None,
                catalog_authority_source: None,
            });
        }

        for (guardian_id, entry) in manifest.guardians {
            if !entry.applies_to.contains(&phase) {
                continue;
            }
            discovery.guardians.push(GuardianCapability {
                guardian_id,
                title: entry.title,
                kind: entry.kind,
                applies_to: entry.applies_to,
                rules: entry.rules,
                severity_floor: entry.severity_floor,
                command_ref: entry.command,
                instruction_ref: entry.instruction,
                authority_source: GuidanceAuthoritySource::SharedPack,
                source_ref: source_ref.clone(),
                pack_id: pack_id.clone(),
                catalog_pillar: None,
                catalog_default_disposition: None,
                catalog_authority_source: None,
            });
        }
    }

    discovery
}

fn discover_workspace_guidance(
    workspace_ref: &Path,
    phase: CapabilityPhase,
) -> (Vec<GuidanceCapability>, Vec<SkippedCapabilitySource>) {
    let guidance_dir = workspace_ref.join(WORKSPACE_GUIDANCE_DIR);
    let mut discovered = Vec::new();
    let mut skipped = Vec::new();

    let Ok(entries) = fs::read_dir(&guidance_dir) else {
        return (discovered, skipped);
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let source_ref = display_relative_path(workspace_ref, &path);
        let Ok(contents) = fs::read_to_string(&path) else {
            skipped.push(SkippedCapabilitySource {
                source_ref,
                authority_source: GuidanceAuthoritySource::WorkspaceOverride,
                reason: "failed to read workspace guidance override".to_string(),
            });
            continue;
        };

        let capability_id = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| "workspace-guidance".to_string());
        let title =
            markdown_title(&contents).unwrap_or_else(|| title_from_identifier(&capability_id));
        let applies_to = all_capability_phases();

        if !applies_to.contains(&phase) {
            continue;
        }

        discovered.push(GuidanceCapability {
            capability_id,
            title,
            applies_to,
            roles: DEFAULT_WORKSPACE_GUIDANCE_ROLES
                .iter()
                .map(|role| (*role).to_string())
                .collect(),
            content_ref: source_ref.clone(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::WorkspaceOverride,
            source_ref,
            pack_id: None,
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        });
    }

    (discovered, skipped)
}

fn discover_workspace_guardians(
    workspace_ref: &Path,
    phase: CapabilityPhase,
) -> (Vec<GuardianCapability>, Vec<SkippedCapabilitySource>) {
    let guardians_dir = workspace_ref.join(WORKSPACE_GUARDIANS_DIR);
    let mut discovered = Vec::new();
    let mut skipped = Vec::new();

    let Ok(entries) = fs::read_dir(&guardians_dir) else {
        return (discovered, skipped);
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("toml"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let source_ref = display_relative_path(workspace_ref, &path);
        let Ok(contents) = fs::read_to_string(&path) else {
            skipped.push(SkippedCapabilitySource {
                source_ref,
                authority_source: GuidanceAuthoritySource::WorkspaceOverride,
                reason: "failed to read workspace guardian override".to_string(),
            });
            continue;
        };

        let Ok(manifest) = toml::from_str::<CapabilityPackManifest>(&contents) else {
            skipped.push(SkippedCapabilitySource {
                source_ref,
                authority_source: GuidanceAuthoritySource::WorkspaceOverride,
                reason: "failed to parse workspace guardian override".to_string(),
            });
            continue;
        };

        for (guardian_id, entry) in manifest.guardians {
            if !entry.applies_to.contains(&phase) {
                continue;
            }
            discovered.push(GuardianCapability {
                guardian_id,
                title: entry.title,
                kind: entry.kind,
                applies_to: entry.applies_to,
                rules: entry.rules,
                severity_floor: entry.severity_floor,
                command_ref: entry.command,
                instruction_ref: entry.instruction,
                authority_source: GuidanceAuthoritySource::WorkspaceOverride,
                source_ref: source_ref.clone(),
                pack_id: None,
                catalog_pillar: None,
                catalog_default_disposition: None,
                catalog_authority_source: None,
            });
        }
    }

    (discovered, skipped)
}

fn discover_optional_canon_guidance(
    workspace_ref: &Path,
    phase: CapabilityPhase,
    pack_guidance: &[GuidanceCapability],
) -> (Vec<GuidanceCapability>, Vec<SkippedCapabilitySource>) {
    let canon_guidance_dir = workspace_ref.join(CANON_GUIDANCE_DIR);
    if canon_guidance_dir.is_dir() {
        let mut discovered = Vec::new();
        let mut skipped = Vec::new();
        let metadata = canon_guidance_metadata(pack_guidance);

        let Ok(entries) = fs::read_dir(&canon_guidance_dir) else {
            skipped.push(SkippedCapabilitySource {
                source_ref: CANON_GUIDANCE_DIR.to_string(),
                authority_source: GuidanceAuthoritySource::CanonGoverned,
                reason: "failed to read governed guidance directory".to_string(),
            });
            return (discovered, skipped);
        };

        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        paths.sort();

        for path in paths {
            let source_ref = display_relative_path(workspace_ref, &path);
            let Ok(contents) = fs::read_to_string(&path) else {
                skipped.push(SkippedCapabilitySource {
                    source_ref,
                    authority_source: GuidanceAuthoritySource::CanonGoverned,
                    reason: "failed to read governed guidance markdown".to_string(),
                });
                continue;
            };

            let key = normalized_identifier(
                path.file_stem().and_then(|value| value.to_str()).unwrap_or("canon-guidance"),
            );
            let template = metadata.get(&key);
            let capability_id = template
                .map(|capability| capability.capability_id.clone())
                .unwrap_or_else(|| key.replace('-', "_"));
            let applies_to = template
                .map(|capability| capability.applies_to.clone())
                .unwrap_or_else(all_capability_phases);
            if !applies_to.contains(&phase) {
                continue;
            }

            discovered.push(GuidanceCapability {
                capability_id,
                title: markdown_title(&contents).unwrap_or_else(|| title_from_identifier(&key)),
                applies_to,
                roles: template.map(|capability| capability.roles.clone()).unwrap_or_else(|| {
                    DEFAULT_WORKSPACE_GUIDANCE_ROLES
                        .iter()
                        .map(|role| (*role).to_string())
                        .collect()
                }),
                content_ref: source_ref.clone(),
                priority: template
                    .map(|capability| capability.priority)
                    .unwrap_or(GuidancePriority::High),
                authority_source: GuidanceAuthoritySource::CanonGoverned,
                source_ref,
                pack_id: template.and_then(|capability| capability.pack_id.clone()),
                catalog_pillar: template.and_then(|capability| capability.catalog_pillar),
                catalog_strength: template.and_then(|capability| capability.catalog_strength),
                catalog_authority_source: template
                    .and_then(|capability| capability.catalog_authority_source),
            });
        }

        return (discovered, skipped);
    }

    (
        Vec::new(),
        vec![SkippedCapabilitySource {
            source_ref: CANON_GUIDANCE_DIR.to_string(),
            authority_source: GuidanceAuthoritySource::CanonGoverned,
            reason: "no governed guidance discovered; continuing with local capability sources"
                .to_string(),
        }],
    )
}

fn canon_guidance_metadata(
    pack_guidance: &[GuidanceCapability],
) -> BTreeMap<String, GuidanceCapability> {
    let mut metadata = BTreeMap::new();

    for capability in pack_guidance {
        metadata
            .entry(normalized_identifier(&capability.capability_id))
            .or_insert_with(|| capability.clone());

        if let Some(stem) =
            Path::new(&capability.content_ref).file_stem().and_then(|value| value.to_str())
        {
            metadata.entry(normalized_identifier(stem)).or_insert_with(|| capability.clone());
        }
    }

    metadata
}

fn normalized_identifier(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn bundled_assistant_root() -> PathBuf {
    if let Some(root) = env::var_os(ASSISTANT_ROOT_OVERRIDE_ENV).map(PathBuf::from)
        && root.join(BUNDLED_ASSISTANT_DIR).is_dir()
    {
        return root;
    }

    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_root.join("../..");
    if repo_root.join(BUNDLED_ASSISTANT_DIR).is_dir() {
        repo_root
    } else {
        crate_root.to_path_buf()
    }
}

fn display_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn markdown_title(contents: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(|title| title.trim().to_string()))
        .filter(|title| !title.is_empty())
}

fn title_from_identifier(identifier: &str) -> String {
    identifier
        .split(['-', '_', '.'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn all_capability_phases() -> Vec<CapabilityPhase> {
    vec![
        CapabilityPhase::Planning,
        CapabilityPhase::Architecture,
        CapabilityPhase::Implementation,
        CapabilityPhase::Testing,
        CapabilityPhase::Verification,
        CapabilityPhase::Review,
    ]
}

fn guidance_relevance_score(
    capability: &GuidanceCapability,
    evidence: &GuidanceRuntimeEvidence,
) -> usize {
    let mut score = match capability.priority {
        GuidancePriority::High => 30,
        GuidancePriority::Medium => 20,
        GuidancePriority::Low => 10,
    };
    let haystack = format!(
        "{} {} {} {}",
        capability.capability_id, capability.title, capability.content_ref, capability.source_ref
    )
    .to_ascii_lowercase();

    for term in evidence_terms(evidence) {
        if haystack.contains(&term) {
            score += 15;
        }
    }

    if capability.roles.iter().any(|role| role == "planner") {
        score += 10;
    }

    score
}

fn evidence_terms(evidence: &GuidanceRuntimeEvidence) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();

    if let Some(language) = evidence.language.as_ref() {
        terms.insert(language.to_ascii_lowercase());
    }
    if evidence.has_tests {
        terms.insert("test".to_string());
        terms.insert("testing".to_string());
    }
    for text in std::iter::once(evidence.goal_text.as_str())
        .chain(evidence.selected_targets.iter().map(String::as_str))
        .chain(evidence.primary_inputs.iter().map(String::as_str))
    {
        for token in text
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .map(|token| token.to_ascii_lowercase())
            .filter(|token| token.len() > 2)
        {
            terms.insert(token);
        }
    }

    terms
}

fn unique_sources(sources: Vec<&str>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for source in sources {
        if seen.insert(source.to_string()) {
            unique.push(source.to_string());
        }
    }
    unique
}

fn sort_sources_by_authority(mut sources: Vec<String>) -> Vec<String> {
    sources.sort_by(|left, right| {
        compare_authority_precedence(authority_for_source(left), authority_for_source(right))
            .then(left.cmp(right))
    });
    sources
}

fn authority_for_source(source_ref: &str) -> GuidanceAuthoritySource {
    if source_ref.starts_with(".boundline/") {
        GuidanceAuthoritySource::WorkspaceOverride
    } else if source_ref.starts_with(".canon/") {
        GuidanceAuthoritySource::CanonGoverned
    } else if source_ref.starts_with("assistant/packs/") {
        GuidanceAuthoritySource::SharedPack
    } else {
        GuidanceAuthoritySource::BuiltIn
    }
}

fn skipped_source_line(source: &SkippedCapabilitySource) -> String {
    format!("{} ({})", source.source_ref, source.reason)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GuardianEvaluation {
    Completed { route_slot: Option<RouteSlot>, new_findings: Vec<GuardianFinding>, summary: String },
    Failed { route_slot: Option<RouteSlot>, finding: GuardianFinding, reason: String },
    Degraded { route_slot: RouteSlot, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SemanticRouteAvailability {
    Available(RouteSlot),
    Unavailable { slot: RouteSlot, reason: String },
}

fn evaluate_deterministic_guardian(
    workspace_ref: &Path,
    guardian: &GuardianCapability,
    request: &GuardianExecutionRequest,
) -> GuardianEvaluation {
    match guardian.command_ref.as_deref() {
        Some("builtin:no-panic-flow") => scan_changed_files(
            workspace_ref,
            guardian,
            request,
            &["panic!", "todo!", "unimplemented!", "unreachable!"],
            "panic-prone control flow detected",
        ),
        Some("builtin:no-unwrap-expect") => scan_changed_files(
            workspace_ref,
            guardian,
            request,
            &[".unwrap(", ".expect("],
            "unwrap/expect shortcut detected",
        ),
        Some("builtin:validation-evidence") => validation_evidence_guardian(guardian, request),
        Some(other) => GuardianEvaluation::Failed {
            route_slot: None,
            finding: guardian_failure_finding(
                guardian,
                request.phase,
                format!("unsupported deterministic guardian command {other}"),
            ),
            reason: format!("unsupported deterministic guardian command {other}"),
        },
        None => GuardianEvaluation::Failed {
            route_slot: None,
            finding: guardian_failure_finding(
                guardian,
                request.phase,
                "missing deterministic guardian command reference".to_string(),
            ),
            reason: "missing deterministic guardian command reference".to_string(),
        },
    }
}

fn evaluate_semantic_guardian(
    guardian: &GuardianCapability,
    route_slot: RouteSlot,
) -> GuardianEvaluation {
    GuardianEvaluation::Degraded {
        route_slot,
        reason: format!(
            "semantic guardian {} requires real provider execution on route {}; placeholder semantic review output is disabled",
            guardian.guardian_id,
            route_slot.as_str()
        ),
    }
}

fn scan_changed_files(
    workspace_ref: &Path,
    guardian: &GuardianCapability,
    request: &GuardianExecutionRequest,
    patterns: &[&str],
    summary_text: &str,
) -> GuardianEvaluation {
    let changed_files = guardian_changed_files(request);
    let mut findings = Vec::new();

    for file in &changed_files {
        let path = workspace_ref.join(file);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) => {
                return GuardianEvaluation::Failed {
                    route_slot: None,
                    finding: guardian_failure_finding(
                        guardian,
                        request.phase,
                        format!("failed to read {file}: {error}"),
                    ),
                    reason: format!("failed to read {file}: {error}"),
                };
            }
        };

        if patterns.iter().any(|pattern| contents.contains(pattern)) {
            findings.push(GuardianFinding {
                finding_id: format!("{}-{}", guardian.guardian_id, normalize_finding_suffix(file)),
                guardian_id: guardian.guardian_id.clone(),
                rule_id: guardian
                    .rules
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "deterministic_rule".to_string()),
                disposition: guardian.severity_floor,
                summary: format!("{summary_text} in {file}"),
                evidence_refs: vec![file.clone()],
                confidence: FindingConfidence::High,
                recommended_action: format!(
                    "replace the flagged control flow in {file} with explicit bounded error handling"
                ),
                authority_source: guardian.authority_source,
                source_ref: guardian.source_ref.clone(),
                phase: request.phase,
            });
        }
    }

    let summary = if findings.is_empty() {
        format!("{} found no issues", guardian.guardian_id)
    } else {
        format!("{} emitted {} finding(s)", guardian.guardian_id, findings.len())
    };

    GuardianEvaluation::Completed { route_slot: None, new_findings: findings, summary }
}

fn validation_evidence_guardian(
    guardian: &GuardianCapability,
    request: &GuardianExecutionRequest,
) -> GuardianEvaluation {
    let has_validation_evidence = request.evidence_refs.iter().any(|reference| {
        let lower = reference.to_ascii_lowercase();
        lower.contains("test") || lower.contains("verify") || lower.contains("cargo")
    });

    let findings = if has_validation_evidence {
        Vec::new()
    } else {
        vec![GuardianFinding {
            finding_id: format!("{}-verification-evidence", guardian.guardian_id),
            guardian_id: guardian.guardian_id.clone(),
            rule_id: guardian
                .rules
                .first()
                .cloned()
                .unwrap_or_else(|| "verification_evidence".to_string()),
            disposition: guardian.severity_floor,
            summary: "verification evidence was not explicit for the bounded change".to_string(),
            evidence_refs: request.evidence_refs.clone(),
            confidence: FindingConfidence::Medium,
            recommended_action: "record an explicit verification command or evidence ref before finalizing the bounded run".to_string(),
            authority_source: guardian.authority_source,
            source_ref: guardian.source_ref.clone(),
            phase: request.phase,
        }]
    };

    GuardianEvaluation::Completed {
        route_slot: None,
        summary: if findings.is_empty() {
            format!("{} confirmed explicit verification evidence", guardian.guardian_id)
        } else {
            format!("{} flagged missing verification evidence", guardian.guardian_id)
        },
        new_findings: findings,
    }
}

fn guardian_changed_files(request: &GuardianExecutionRequest) -> Vec<String> {
    if !request.changed_files.is_empty() {
        return request.changed_files.clone();
    }
    vec![request.target_ref.clone()]
}

fn guardian_failure_finding(
    guardian: &GuardianCapability,
    phase: CapabilityPhase,
    reason: String,
) -> GuardianFinding {
    GuardianFinding {
        finding_id: format!("{}-failure", guardian.guardian_id),
        guardian_id: guardian.guardian_id.clone(),
        rule_id: "guardian_failure".to_string(),
        disposition: GuardianDisposition::Error,
        summary: reason.clone(),
        evidence_refs: Vec::new(),
        confidence: FindingConfidence::High,
        recommended_action: "inspect the guardian configuration or the bounded file evidence"
            .to_string(),
        authority_source: guardian.authority_source,
        source_ref: guardian.source_ref.clone(),
        phase,
    }
}

fn normalize_finding_suffix(input: &str) -> String {
    input.chars().map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' }).collect()
}

fn route_slot_for_phase(phase: CapabilityPhase) -> Option<RouteSlot> {
    Some(match phase {
        CapabilityPhase::Planning | CapabilityPhase::Architecture => RouteSlot::Planning,
        CapabilityPhase::Implementation => RouteSlot::Implementation,
        CapabilityPhase::Testing | CapabilityPhase::Verification => RouteSlot::Verification,
        CapabilityPhase::Review => RouteSlot::Review,
    })
}

fn semantic_route_availability(
    workspace_ref: &Path,
    phase: CapabilityPhase,
) -> SemanticRouteAvailability {
    let Some(slot) = route_slot_for_phase(phase) else {
        return SemanticRouteAvailability::Unavailable {
            slot: RouteSlot::Verification,
            reason: "no compatible route slot is available for this guardian phase".to_string(),
        };
    };

    let workspace_routing =
        FileConfigStore::for_workspace(workspace_ref).local_routing().ok().flatten();
    let global_routing = FileConfigStore::global_routing().ok().flatten();
    let effective_routing = resolve_effective_routing(
        &RoutingOverrides::default(),
        workspace_routing.as_ref(),
        None,
        global_routing.as_ref(),
    );
    let runtime_capabilities = resolve_effective_runtime_capabilities(
        workspace_routing.as_ref(),
        None,
        global_routing.as_ref(),
    );
    let route = match slot {
        RouteSlot::Planning => effective_routing.planning.route,
        RouteSlot::Implementation => effective_routing.implementation.route,
        RouteSlot::Verification => effective_routing.verification.route,
        RouteSlot::Review => effective_routing.review.route,
    };

    if runtime_capabilities
        .get(&route.runtime)
        .is_some_and(|profile| !profile.profile.validation.is_supported())
    {
        return SemanticRouteAvailability::Unavailable {
            slot,
            reason: format!(
                "route {} is configured on {} but validation support is unavailable",
                slot.as_str(),
                route.runtime.as_str()
            ),
        };
    }

    SemanticRouteAvailability::Available(slot)
}

fn guardian_findings_summary(findings: &[GuardianFinding]) -> Option<String> {
    (!findings.is_empty()).then(|| {
        format!(
            "{} guardian finding(s); blocking={}",
            findings.len(),
            findings.iter().any(|finding| finding.disposition.is_blocking())
        )
    })
}

fn blocking_outcome_text(findings: &[GuardianFinding]) -> Option<String> {
    if findings.iter().any(|finding| finding.disposition.is_blocking()) {
        Some("blocking deterministic findings stop redundant semantic guardians".to_string())
    } else if findings.is_empty() {
        None
    } else {
        Some("guardian findings recorded without a blocking outcome".to_string())
    }
}

/// Maximum number of guidance documents to load content from per phase to
/// keep provider system prompts bounded.
const MAX_GUIDANCE_CONTENT_ENTRIES: usize = 4;

/// Maximum character length for a single guidance document loaded into the
/// system prompt. Longer documents are truncated at this boundary.
const MAX_GUIDANCE_CONTENT_CHARS: usize = 3000;

/// Loads the text content of a resolved guidance capability from its
/// `content_ref` path. Resolves relative paths against the bundled assistant
/// root first, falling back to the workspace root. Returns `None` when the
/// file is missing, unreadable, or empty.
pub fn load_guidance_content(
    workspace_ref: &Path,
    capability: &GuidanceCapability,
) -> Option<String> {
    let assistant_root = bundled_assistant_root();
    let candidate_paths =
        [assistant_root.join(&capability.content_ref), workspace_ref.join(&capability.content_ref)];

    for path in &candidate_paths {
        if let Ok(content) = fs::read_to_string(path) {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.chars().count() <= MAX_GUIDANCE_CONTENT_CHARS {
                return Some(trimmed.to_string());
            }
            return Some(truncate_guidance_content(trimmed));
        }
    }
    None
}

fn truncate_guidance_content(content: &str) -> String {
    let Some((end_index, _)) = content.char_indices().nth(MAX_GUIDANCE_CONTENT_CHARS) else {
        return content.to_string();
    };
    let mut truncated = content[..end_index].to_string();
    truncated.push_str("\n\n[truncated]");
    truncated
}

/// Resolves and loads guidance content for a lifecycle phase, returning up to
/// `MAX_GUIDANCE_CONTENT_ENTRIES` documents suitable for injection into a
/// provider system prompt.
pub fn load_guidance_for_phase(
    workspace_ref: &Path,
    phase: CapabilityPhase,
    evidence: &GuidanceRuntimeEvidence,
) -> Vec<String> {
    let resolution = resolve_capabilities_for_phase(workspace_ref, phase, evidence);
    resolution
        .guidance
        .iter()
        .take(MAX_GUIDANCE_CONTENT_ENTRIES)
        .filter_map(|capability| load_guidance_content(workspace_ref, capability))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::configuration::{
        CapabilityState, ConfigFile, ModelRoute, RouteSlot, RoutingConfig,
        RuntimeCapabilityProfile, RuntimeKind,
    };
    use crate::domain::goal_plan::{ContextInput, ContextInputKind, ContextPack, WorkspaceSignals};
    use crate::domain::guidance::{
        CapabilityPhase, FindingConfidence, GuardianCapability, GuardianDisposition,
        GuardianFinding, GuardianKind, GuidanceAuthoritySource, GuidanceCapability,
        GuidancePriority, SkippedCapabilitySource,
    };

    use uuid::Uuid;

    use super::{
        GuardianEvaluation, MAX_GUIDANCE_CONTENT_CHARS, SemanticRouteAvailability,
        blocking_outcome_text, compare_authority_precedence, discover_optional_canon_guidance,
        discover_workspace_guardians, discover_workspace_guidance, display_relative_path,
        evaluate_deterministic_guardian, evaluate_semantic_guardian, guardian_changed_files,
        guardian_findings_summary, guardian_kind_requires_route, guidance_relevance_score,
        has_blocking_findings, load_guidance_content, markdown_title, normalize_finding_suffix,
        normalized_identifier, order_guardians_for_execution, planning_runtime_evidence,
        resolve_capabilities_for_phase, resolve_guidance_candidates, route_slot_for_phase,
        semantic_route_availability, should_short_circuit_semantic_guards, skipped_source_line,
        title_from_identifier, validation_evidence_guardian,
    };

    fn guardian(
        guardian_id: &str,
        kind: GuardianKind,
        authority_source: GuidanceAuthoritySource,
    ) -> GuardianCapability {
        GuardianCapability {
            guardian_id: guardian_id.to_string(),
            title: guardian_id.to_string(),
            kind,
            applies_to: vec![CapabilityPhase::Verification],
            rules: vec!["rule".to_string()],
            severity_floor: GuardianDisposition::Concern,
            command_ref: Some("scripts/check.sh".to_string()),
            instruction_ref: Some("assistant/prompts/check.md".to_string()),
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
            guardian_id: "guardian".to_string(),
            rule_id: "rule".to_string(),
            disposition,
            summary: "summary".to_string(),
            evidence_refs: Vec::new(),
            confidence: FindingConfidence::High,
            recommended_action: "fix it".to_string(),
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guardians/check.toml".to_string(),
            phase: CapabilityPhase::Verification,
        }
    }

    #[test]
    fn authority_precedence_prefers_workspace_over_built_in() {
        assert_eq!(
            compare_authority_precedence(
                GuidanceAuthoritySource::WorkspaceOverride,
                GuidanceAuthoritySource::BuiltIn,
            ),
            std::cmp::Ordering::Less,
        );
    }

    #[test]
    fn guardian_order_keeps_deterministic_before_semantic() {
        let ordered = order_guardians_for_execution(vec![
            guardian("llm_guard", GuardianKind::Llm, GuidanceAuthoritySource::BuiltIn),
            guardian("det_guard", GuardianKind::Deterministic, GuidanceAuthoritySource::SharedPack),
            guardian("hybrid_guard", GuardianKind::Hybrid, GuidanceAuthoritySource::CanonGoverned),
        ]);

        assert_eq!(ordered[0].guardian_id, "det_guard");
        assert_eq!(ordered[1].guardian_id, "hybrid_guard");
        assert_eq!(ordered[2].guardian_id, "llm_guard");
    }

    #[test]
    fn blocking_findings_short_circuit_semantic_guards() {
        let findings =
            vec![finding(GuardianDisposition::Advise), finding(GuardianDisposition::Block)];

        assert!(has_blocking_findings(&findings));
        assert!(should_short_circuit_semantic_guards(&findings));
    }

    #[test]
    fn only_semantic_guardians_require_routes() {
        assert!(!guardian_kind_requires_route(GuardianKind::Deterministic));
        assert!(guardian_kind_requires_route(GuardianKind::Hybrid));
        assert!(guardian_kind_requires_route(GuardianKind::Llm));
    }

    #[test]
    fn resolve_guidance_candidates_records_shadowed_sources() {
        let workspace_capability = GuidanceCapability {
            capability_id: "clean-code".to_string(),
            title: "Workspace Clean Code".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: ".boundline/guidance/clean-code.md".to_string(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::WorkspaceOverride,
            source_ref: ".boundline/guidance/clean-code.md".to_string(),
            pack_id: None,
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };
        let pack_capability = GuidanceCapability {
            capability_id: "clean-code".to_string(),
            title: "Shared Clean Code".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: "assistant/packs/shared/guidance/clean-code.md".to_string(),
            priority: GuidancePriority::Medium,
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/shared/pack.toml".to_string(),
            pack_id: Some("shared".to_string()),
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };
        let evidence = super::GuidanceRuntimeEvidence {
            goal_text: "Tighten clean code guidance".to_string(),
            language: Some("rust".to_string()),
            selected_targets: vec!["src/lib.rs".to_string()],
            primary_inputs: vec!["src/lib.rs".to_string()],
            has_tests: true,
        };

        let (resolved, skipped) = resolve_guidance_candidates(
            vec![pack_capability, workspace_capability.clone()],
            &evidence,
        );

        assert_eq!(resolved, vec![workspace_capability]);
        assert_eq!(skipped.len(), 1);
        assert!(
            skipped[0]
                .reason
                .contains("shadowed by clean-code from .boundline/guidance/clean-code.md")
        );
    }

    #[test]
    fn workspace_override_discovery_reports_read_and_parse_failures() {
        let workspace = temp_workspace("guidance-runtime-workspace-errors");
        fs::create_dir_all(workspace.join(".boundline/guidance/bad.md")).unwrap();
        fs::create_dir_all(workspace.join(".boundline/guardians/unreadable.toml")).unwrap();
        fs::write(workspace.join(".boundline/guardians/invalid.toml"), "invalid = [toml").unwrap();

        let (guidance, guidance_skips) =
            discover_workspace_guidance(&workspace, CapabilityPhase::Planning);
        let (guardians, guardian_skips) =
            discover_workspace_guardians(&workspace, CapabilityPhase::Verification);

        assert!(guidance.is_empty());
        assert!(guardians.is_empty());
        assert!(guidance_skips.iter().any(|skip| {
            skip.reason == "failed to read workspace guidance override"
                && skip.source_ref == ".boundline/guidance/bad.md"
        }));
        assert!(guardian_skips.iter().any(|skip| {
            skip.reason == "failed to read workspace guardian override"
                && skip.source_ref == ".boundline/guardians/unreadable.toml"
        }));
        assert!(guardian_skips.iter().any(|skip| {
            skip.reason == "failed to parse workspace guardian override"
                && skip.source_ref == ".boundline/guardians/invalid.toml"
        }));
    }

    #[test]
    fn deterministic_guardian_helpers_cover_failure_and_evidence_paths() {
        let workspace = temp_workspace("guidance-runtime-deterministic-helpers");
        let request = super::GuardianExecutionRequest {
            goal_text: "Verify the bounded change".to_string(),
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Verification,
            evidence_refs: Vec::new(),
            changed_files: Vec::new(),
            workspace_signals: WorkspaceSignals {
                language: Some("rust".to_string()),
                file_count: 1,
                has_config: true,
                has_canon: false,
                has_tests: true,
            },
        };

        let mut unsupported = guardian(
            "unsupported_guard",
            GuardianKind::Deterministic,
            GuidanceAuthoritySource::BuiltIn,
        );
        unsupported.command_ref = Some("builtin:unsupported".to_string());
        match evaluate_deterministic_guardian(&workspace, &unsupported, &request) {
            GuardianEvaluation::Failed { reason, finding, .. } => {
                assert!(
                    reason
                        .contains("unsupported deterministic guardian command builtin:unsupported")
                );
                assert!(
                    finding
                        .summary
                        .contains("unsupported deterministic guardian command builtin:unsupported")
                );
            }
            other => panic!("expected failure, got {other:?}"),
        }

        let mut missing = guardian(
            "missing_guard",
            GuardianKind::Deterministic,
            GuidanceAuthoritySource::BuiltIn,
        );
        missing.command_ref = None;
        match evaluate_deterministic_guardian(&workspace, &missing, &request) {
            GuardianEvaluation::Failed { reason, .. } => {
                assert_eq!(reason, "missing deterministic guardian command reference");
            }
            other => panic!("expected failure, got {other:?}"),
        }

        let validation_guardian = guardian(
            "validation_guard",
            GuardianKind::Deterministic,
            GuidanceAuthoritySource::BuiltIn,
        );
        match validation_evidence_guardian(&validation_guardian, &request) {
            GuardianEvaluation::Completed { new_findings, summary, .. } => {
                assert_eq!(new_findings.len(), 1);
                assert!(summary.contains("flagged missing verification evidence"));
            }
            other => panic!("expected completion, got {other:?}"),
        }

        let request_with_evidence = super::GuardianExecutionRequest {
            evidence_refs: vec!["cargo test --quiet".to_string()],
            ..request.clone()
        };
        match validation_evidence_guardian(&validation_guardian, &request_with_evidence) {
            GuardianEvaluation::Completed { new_findings, summary, .. } => {
                assert!(new_findings.is_empty());
                assert!(summary.contains("confirmed explicit verification evidence"));
            }
            other => panic!("expected completion, got {other:?}"),
        }

        assert_eq!(guardian_changed_files(&request), vec!["src/lib.rs".to_string()]);
        assert_eq!(normalize_finding_suffix("src/lib.rs"), "src-lib-rs");
        assert_eq!(route_slot_for_phase(CapabilityPhase::Architecture), Some(RouteSlot::Planning));
        assert_eq!(route_slot_for_phase(CapabilityPhase::Review), Some(RouteSlot::Review));
    }

    #[test]
    fn semantic_guardian_requires_real_provider_execution_or_degrades() {
        let semantic_guardian =
            guardian("semantic_guard", GuardianKind::Llm, GuidanceAuthoritySource::BuiltIn);

        match evaluate_semantic_guardian(&semantic_guardian, RouteSlot::Verification) {
            GuardianEvaluation::Degraded { route_slot, reason } => {
                assert_eq!(route_slot, RouteSlot::Verification);
                assert!(reason.contains("real provider execution"));
            }
            other => panic!("expected semantic guardian degradation, got {other:?}"),
        }
    }

    #[test]
    fn planning_resolution_loads_bundled_guidance_and_discloses_missing_canon() {
        let workspace = temp_workspace("guidance-runtime-bundled");
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"guided\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();

        let context_pack = ContextPack {
            pack_id: "context-pack".to_string(),
            summary: "bounded rust context".to_string(),
            credibility: crate::domain::goal_plan::ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "matches the active goal".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        };
        let signals = WorkspaceSignals {
            language: Some("rust".to_string()),
            file_count: 3,
            has_config: false,
            has_canon: false,
            has_tests: true,
        };

        let resolution = resolve_capabilities_for_phase(
            &workspace,
            CapabilityPhase::Planning,
            &planning_runtime_evidence("Fix the failing Rust tests", &context_pack, &signals),
        );

        assert!(!resolution.guidance.is_empty());
        assert!(
            resolution
                .projection
                .loaded_guidance_sources
                .iter()
                .any(|source| source.starts_with("assistant/packs/"))
        );
        assert!(
            resolution
                .projection
                .skipped_guidance_sources
                .iter()
                .any(|source| source.contains(".canon/boundline/guidance"))
        );
    }

    #[test]
    fn workspace_guidance_override_shadows_shared_pack_entry() {
        let workspace = temp_workspace("guidance-runtime-override");
        fs::create_dir_all(workspace.join(".boundline/guidance")).unwrap();
        fs::write(
            workspace.join(".boundline/guidance/clean-code.md"),
            "# Workspace Clean Code\nPrefer smaller change sets.\n",
        )
        .unwrap();

        let context_pack = ContextPack {
            pack_id: "context-pack".to_string(),
            summary: "bounded planning context".to_string(),
            credibility: crate::domain::goal_plan::ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "bounded target".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        };
        let signals = WorkspaceSignals {
            language: Some("rust".to_string()),
            file_count: 1,
            has_config: true,
            has_canon: false,
            has_tests: true,
        };

        let resolution = resolve_capabilities_for_phase(
            &workspace,
            CapabilityPhase::Planning,
            &planning_runtime_evidence("Tighten clean code guidance", &context_pack, &signals),
        );

        assert!(resolution.guidance.iter().any(|capability| {
            capability.capability_id == "clean-code"
                && capability.authority_source == GuidanceAuthoritySource::WorkspaceOverride
        }));
        assert!(
            resolution.projection.skipped_guidance_sources.iter().any(|source| {
                source.contains("assistant/packs/") && source.contains("shadowed")
            })
        );
    }

    fn temp_workspace(prefix: &str) -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn save_local_routing(workspace: &std::path::Path, routing: RoutingConfig) {
        FileConfigStore::for_workspace(workspace)
            .save_local(&ConfigFile { version: 1, routing, canon: None, adapter: None })
            .unwrap();
    }

    #[test]
    fn helper_titles_and_markdown_headings_are_stable() {
        assert_eq!(
            markdown_title("# Workspace Guard\ncontent"),
            Some("Workspace Guard".to_string())
        );
        assert_eq!(markdown_title("heading without markdown"), None);
        assert_eq!(title_from_identifier("rust.testing_guardians"), "Rust Testing Guardians");
    }

    #[test]
    fn guidance_relevance_score_rewards_language_and_test_signals() {
        let strongly_relevant = GuidanceCapability {
            capability_id: "rust-testing".to_string(),
            title: "Rust testing discipline".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: "assistant/packs/rust/guidance/testing.md".to_string(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/rust/pack.toml".to_string(),
            pack_id: Some("rust".to_string()),
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };
        let loosely_relevant = GuidanceCapability {
            capability_id: "review-handoff".to_string(),
            title: "Review handoff guidance".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["reviewer".to_string()],
            content_ref: "assistant/packs/shared/guidance/review.md".to_string(),
            priority: GuidancePriority::Low,
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/shared/pack.toml".to_string(),
            pack_id: Some("shared".to_string()),
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };
        let evidence = super::GuidanceRuntimeEvidence {
            goal_text: "Fix the failing Rust tests".to_string(),
            language: Some("rust".to_string()),
            selected_targets: vec!["src/lib.rs".to_string()],
            primary_inputs: vec!["tests/red_to_green.rs".to_string()],
            has_tests: true,
        };

        assert!(
            guidance_relevance_score(&strongly_relevant, &evidence)
                > guidance_relevance_score(&loosely_relevant, &evidence)
        );
    }

    #[test]
    fn optional_canon_guidance_is_only_skipped_when_directory_is_missing() {
        let workspace = temp_workspace("guidance-runtime-canon-guidance");
        let seed = GuidanceCapability {
            capability_id: "clean_code".to_string(),
            title: "Clean Code".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: "assistant/packs/guidance-catalog/guidance/clean-code.md".to_string(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/guidance-catalog".to_string(),
            pack_id: Some("boundline-guidance-catalog".to_string()),
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };

        let (guidance, skipped) = discover_optional_canon_guidance(
            &workspace,
            CapabilityPhase::Planning,
            std::slice::from_ref(&seed),
        );
        assert!(guidance.is_empty());
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].source_ref, ".canon/boundline/guidance");

        fs::create_dir_all(workspace.join(".canon/boundline/guidance")).unwrap();
        fs::write(
            workspace.join(".canon/boundline/guidance/clean-code.md"),
            "# Canon Clean Code\nPrefer the governed standard.\n",
        )
        .unwrap();

        let (guidance, skipped) =
            discover_optional_canon_guidance(&workspace, CapabilityPhase::Planning, &[seed]);
        assert!(skipped.is_empty());
        assert_eq!(guidance.len(), 1);
        assert_eq!(guidance[0].authority_source, GuidanceAuthoritySource::CanonGoverned);
        assert_eq!(guidance[0].title, "Canon Clean Code");
    }

    #[test]
    fn canon_guidance_supersedes_catalog_pack_for_the_same_capability() {
        let workspace = temp_workspace("guidance-runtime-canon-precedence");
        fs::create_dir_all(workspace.join(".canon/boundline/guidance")).unwrap();
        fs::write(
            workspace.join(".canon/boundline/guidance/clean-code.md"),
            "# Canon Clean Code\nPrefer the governed standard.\n",
        )
        .unwrap();

        let pack_capability = GuidanceCapability {
            capability_id: "clean_code".to_string(),
            title: "Shared Clean Code".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: "assistant/packs/guidance-catalog/guidance/clean-code.md".to_string(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/guidance-catalog".to_string(),
            pack_id: Some("boundline-guidance-catalog".to_string()),
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };
        let (canon_guidance, skipped) = discover_optional_canon_guidance(
            &workspace,
            CapabilityPhase::Planning,
            std::slice::from_ref(&pack_capability),
        );
        assert!(skipped.is_empty());

        let evidence = super::GuidanceRuntimeEvidence {
            goal_text: "Honor the governed clean code standard".to_string(),
            language: Some("rust".to_string()),
            selected_targets: vec!["src/lib.rs".to_string()],
            primary_inputs: vec!["src/lib.rs".to_string()],
            has_tests: true,
        };

        let (resolved, skipped) = resolve_guidance_candidates(
            vec![pack_capability, canon_guidance[0].clone()],
            &evidence,
        );

        assert_eq!(resolved, canon_guidance);
        assert_eq!(skipped.len(), 1);
        assert!(skipped[0].source_ref.contains("assistant/packs/guidance-catalog"));
        assert!(
            skipped[0]
                .reason
                .contains("shadowed by clean_code from .canon/boundline/guidance/clean-code.md")
        );
    }

    #[test]
    fn workspace_guidance_without_heading_uses_identifier_title() {
        let workspace = temp_workspace("guidance-runtime-workspace-title");
        fs::create_dir_all(workspace.join(".boundline/guidance")).unwrap();
        fs::write(
            workspace.join(".boundline/guidance/rust-testing.md"),
            "Prefer bounded Rust test coverage updates.\n",
        )
        .unwrap();

        let context_pack = ContextPack {
            pack_id: "context-pack".to_string(),
            summary: "bounded rust context".to_string(),
            credibility: crate::domain::goal_plan::ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "active target".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        };
        let signals = WorkspaceSignals {
            language: Some("rust".to_string()),
            file_count: 2,
            has_config: true,
            has_canon: false,
            has_tests: true,
        };

        let resolution = resolve_capabilities_for_phase(
            &workspace,
            CapabilityPhase::Planning,
            &planning_runtime_evidence("Tighten Rust testing guidance", &context_pack, &signals),
        );

        let capability = resolution
            .guidance
            .iter()
            .find(|capability| capability.capability_id == "rust-testing")
            .unwrap();
        assert_eq!(capability.title, "Rust Testing");
        assert_eq!(capability.source_ref, ".boundline/guidance/rust-testing.md");
    }

    #[test]
    fn workspace_override_discovery_loads_matching_guidance_and_guardians() {
        let workspace = temp_workspace("guidance-runtime-workspace-overrides");
        fs::create_dir_all(workspace.join(".boundline/guidance")).unwrap();
        fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
        fs::write(
            workspace.join(".boundline/guidance/clean-code.md"),
            "# Workspace Clean Code\nPrefer the local rule.\n",
        )
        .unwrap();
        fs::write(
            workspace.join(".boundline/guardians/verification.toml"),
            "[guardians.workspace-review]\ntitle = \"Workspace Review\"\nkind = \"hybrid\"\napplies_to = [\"review\"]\nrules = [\"workspace_review\"]\nseverity_floor = \"warn\"\ncommand = \"builtin:validation-evidence\"\ninstruction = \"../guardians/workspace-review.md\"\n",
        )
        .unwrap();

        let (guidance, skipped_guidance) =
            discover_workspace_guidance(&workspace, CapabilityPhase::Review);
        let (guardians, skipped_guardians) =
            discover_workspace_guardians(&workspace, CapabilityPhase::Review);

        assert!(skipped_guidance.is_empty());
        assert!(skipped_guardians.is_empty());
        assert_eq!(guidance.len(), 1);
        assert_eq!(guidance[0].title, "Workspace Clean Code");
        assert_eq!(guidance[0].roles, vec!["planner", "implementer", "verifier", "reviewer"]);
        assert_eq!(guidance[0].priority, GuidancePriority::High);
        assert_eq!(guidance[0].authority_source, GuidanceAuthoritySource::WorkspaceOverride);
        assert_eq!(guardians.len(), 1);
        assert_eq!(guardians[0].guardian_id, "workspace-review");
        assert_eq!(guardians[0].kind, GuardianKind::Hybrid);
        assert_eq!(
            guardians[0].instruction_ref.as_deref(),
            Some("../guardians/workspace-review.md")
        );
    }

    #[test]
    fn optional_canon_guidance_uses_default_metadata_when_no_pack_template_matches() {
        let workspace = temp_workspace("guidance-runtime-canon-fallback");
        fs::create_dir_all(workspace.join(".canon/boundline/guidance")).unwrap();
        fs::write(
            workspace.join(".canon/boundline/guidance/custom-rule.md"),
            "Prefer the governed fallback rule.\n",
        )
        .unwrap();

        let (guidance, skipped) =
            discover_optional_canon_guidance(&workspace, CapabilityPhase::Testing, &[]);

        assert!(skipped.is_empty());
        assert_eq!(guidance.len(), 1);
        assert_eq!(guidance[0].capability_id, "custom_rule");
        assert_eq!(guidance[0].title, "Custom Rule");
        assert_eq!(guidance[0].priority, GuidancePriority::High);
        assert_eq!(guidance[0].roles, vec!["planner", "implementer", "verifier", "reviewer"]);
        assert!(guidance[0].applies_to.contains(&CapabilityPhase::Testing));
        assert_eq!(guidance[0].authority_source, GuidanceAuthoritySource::CanonGoverned);
        assert!(guidance[0].pack_id.is_none());
    }

    #[test]
    fn semantic_route_availability_respects_validation_capabilities() {
        let workspace = temp_workspace("guidance-runtime-semantic-route");

        let mut supported = RoutingConfig::default();
        supported.set_slot(
            RouteSlot::Verification,
            ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-4o".to_string() },
        );
        supported.set_runtime_capability(
            RuntimeKind::Codex,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Supported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: None,
            },
        );
        save_local_routing(&workspace, supported);

        assert_eq!(
            semantic_route_availability(&workspace, CapabilityPhase::Verification),
            SemanticRouteAvailability::Available(RouteSlot::Verification)
        );

        let mut unsupported = RoutingConfig::default();
        unsupported.set_slot(
            RouteSlot::Verification,
            ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-4o".to_string() },
        );
        unsupported.set_runtime_capability(
            RuntimeKind::Codex,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Supported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Unsupported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: Some("validation disabled for test coverage".to_string()),
            },
        );
        save_local_routing(&workspace, unsupported);

        assert_eq!(
            semantic_route_availability(&workspace, CapabilityPhase::Verification),
            SemanticRouteAvailability::Unavailable {
                slot: RouteSlot::Verification,
                reason: "route verification is configured on codex but validation support is unavailable"
                    .to_string(),
            }
        );
    }

    #[test]
    fn utility_helpers_cover_relative_paths_source_labels_and_other_route_slots() {
        let workspace = temp_workspace("guidance-runtime-utility-helpers");
        let nested = workspace.join("assistant/packs/shared/pack.toml");
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        fs::write(&nested, "[guidance]\n").unwrap();

        assert_eq!(display_relative_path(&workspace, &nested), "assistant/packs/shared/pack.toml");
        assert_eq!(
            display_relative_path(&workspace, std::path::Path::new("/tmp/outside-path.md")),
            "/tmp/outside-path.md"
        );
        assert_eq!(
            super::authority_for_source(".canon/boundline/guidance"),
            GuidanceAuthoritySource::CanonGoverned
        );
        assert_eq!(
            super::authority_for_source("assistant/packs/shared/pack.toml"),
            GuidanceAuthoritySource::SharedPack
        );
        assert_eq!(
            super::authority_for_source("assistant/guidance/base.md"),
            GuidanceAuthoritySource::BuiltIn
        );
        assert_eq!(
            skipped_source_line(&SkippedCapabilitySource {
                source_ref: "assistant/packs/shared/pack.toml".to_string(),
                authority_source: GuidanceAuthoritySource::SharedPack,
                reason: "shadowed".to_string(),
            }),
            "assistant/packs/shared/pack.toml (shadowed)"
        );

        let mut routing = RoutingConfig::default();
        routing.set_slot(
            RouteSlot::Planning,
            ModelRoute { runtime: RuntimeKind::Claude, model: "claude-3-7".to_string() },
        );
        routing.set_slot(
            RouteSlot::Review,
            ModelRoute { runtime: RuntimeKind::Claude, model: "claude-3-7".to_string() },
        );
        routing.set_runtime_capability(
            RuntimeKind::Claude,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Supported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: None,
            },
        );
        save_local_routing(&workspace, routing);

        assert_eq!(
            semantic_route_availability(&workspace, CapabilityPhase::Planning),
            SemanticRouteAvailability::Available(RouteSlot::Planning)
        );
        assert_eq!(
            semantic_route_availability(&workspace, CapabilityPhase::Review),
            SemanticRouteAvailability::Available(RouteSlot::Review)
        );
        assert_eq!(normalized_identifier("Clean Code.md"), "clean-code-md");
        assert_eq!(title_from_identifier("clean-code.md"), "Clean Code Md");
        assert_eq!(markdown_title("# Rust Rules\nMore text\n"), Some("Rust Rules".to_string()));
        assert_eq!(markdown_title("No heading\n"), None);
    }

    #[test]
    fn finding_helpers_report_summary_and_blocking_outcomes() {
        let warning = GuardianFinding {
            finding_id: "warn-1".to_string(),
            guardian_id: "catalog_review".to_string(),
            rule_id: "review".to_string(),
            disposition: GuardianDisposition::Warn,
            summary: "warning".to_string(),
            evidence_refs: vec!["src/lib.rs".to_string()],
            confidence: FindingConfidence::Medium,
            recommended_action: "review it".to_string(),
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/packs/guidance-catalog".to_string(),
            phase: CapabilityPhase::Review,
        };
        let blocker = GuardianFinding {
            disposition: GuardianDisposition::Block,
            summary: "blocker".to_string(),
            ..warning.clone()
        };

        assert_eq!(guardian_findings_summary(&[]), None);
        assert_eq!(
            guardian_findings_summary(std::slice::from_ref(&warning)).as_deref(),
            Some("1 guardian finding(s); blocking=false")
        );
        assert_eq!(blocking_outcome_text(&[]), None);
        assert_eq!(
            blocking_outcome_text(&[warning]).as_deref(),
            Some("guardian findings recorded without a blocking outcome")
        );
        assert_eq!(
            blocking_outcome_text(&[blocker]).as_deref(),
            Some("blocking deterministic findings stop redundant semantic guardians")
        );
    }

    #[test]
    fn load_guidance_content_truncates_utf8_without_panicking() {
        let workspace = temp_workspace("guidance-runtime-utf8-truncation");
        let content_ref = "guidance/utf8.md";
        fs::create_dir_all(workspace.join("guidance")).unwrap();
        fs::write(workspace.join(content_ref), "🙂".repeat(MAX_GUIDANCE_CONTENT_CHARS + 5))
            .unwrap();

        let capability = GuidanceCapability {
            capability_id: "utf8-guidance".to_string(),
            title: "UTF-8 Guidance".to_string(),
            applies_to: vec![CapabilityPhase::Implementation],
            roles: Vec::new(),
            content_ref: content_ref.to_string(),
            priority: GuidancePriority::Medium,
            authority_source: GuidanceAuthoritySource::WorkspaceOverride,
            source_ref: content_ref.to_string(),
            pack_id: None,
            catalog_pillar: None,
            catalog_strength: None,
            catalog_authority_source: None,
        };

        let loaded = load_guidance_content(&workspace, &capability).unwrap_or_default();
        assert!(loaded.starts_with('🙂'), "{loaded}");
        assert!(loaded.ends_with("[truncated]"), "{loaded}");
    }
}
