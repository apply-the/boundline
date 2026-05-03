//! Goal-derived planning from workspace state (feature 013).

use std::fs;
use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::domain::configuration::{
    RouteSlot, RoutingOverrides, SourcedRoute, SourcedRuntimeCapabilityProfile,
    SourcedSlotEffortPolicy, ValueSource, resolve_effective_domain_templates,
    resolve_effective_routing, resolve_effective_runtime_capabilities,
    resolve_effective_slot_effort_policies,
};
use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::domain_templates::{
    DomainFamily, ExternalContextBinding, ExternalContextStatus, detect_domain_families,
};
use crate::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, GoalPlanError,
    InferredFlow, PlannedTask, WorkspaceSignals,
};
use crate::domain::governance::{
    CanonCapabilitySnapshot, CompactedCanonMemory, MemoryCredibilityState,
};
use crate::domain::workflow::WorkflowProgressState;
use crate::orchestrator::flow_inference::{FlowInferenceContext, infer_flow_from_context};

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
    pub workflow_progress: Option<WorkflowProgressState>,
    pub canon_capability_snapshot: Option<CanonCapabilitySnapshot>,
    pub compacted_canon_memory: Option<CompactedCanonMemory>,
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

fn has_specific_workspace_targets(relevant_files: &[String]) -> bool {
    relevant_files.iter().any(|file_ref| {
        file_ref.starts_with("src/")
            || file_ref.starts_with("tests/")
            || file_ref.starts_with("test/")
            || file_ref.starts_with("specs/")
    })
}

struct DomainContextOutcome {
    summary_clause: String,
    credibility: ContextPackCredibility,
    inputs: Vec<ContextInput>,
    blocking_reason: Option<String>,
}

fn resolve_effective_domain_templates_for_workspace(
    workspace_ref: &Path,
) -> std::collections::BTreeMap<DomainFamily, crate::domain::configuration::ResolvedDomainTemplate>
{
    let workspace_routing =
        FileConfigStore::for_workspace(workspace_ref).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace_ref)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();

    resolve_effective_domain_templates(
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    )
}

fn resolve_domain_context(
    workspace_ref: &Path,
    selected_target: Option<&str>,
) -> Option<DomainContextOutcome> {
    let effective_templates = resolve_effective_domain_templates_for_workspace(workspace_ref);
    if effective_templates.is_empty() {
        return None;
    }

    let candidate_families = detect_domain_families(workspace_ref, selected_target);
    let active_families = effective_templates
        .iter()
        .filter_map(|(family, template)| template.enabled.then_some(*family))
        .collect::<Vec<_>>();
    let selected_families = candidate_families
        .into_iter()
        .filter(|family| effective_templates.get(family).is_some_and(|template| template.enabled))
        .collect::<Vec<_>>();
    let target_label = selected_target.unwrap_or("workspace");

    if selected_families.is_empty() {
        let configured = if active_families.is_empty() {
            "none".to_string()
        } else {
            active_families.iter().map(|family| family.as_str()).collect::<Vec<_>>().join(", ")
        };
        let reason = format!(
            "no enabled domain template matched `{target_label}`; configured families: {configured}"
        );
        return Some(DomainContextOutcome {
            summary_clause: format!("domain context unavailable for {target_label}"),
            credibility: ContextPackCredibility::Insufficient,
            inputs: vec![ContextInput {
                kind: ContextInputKind::DomainTemplate,
                reference: configured,
                rationale: format!(
                    "configured active domain templates did not match the bounded target {target_label}"
                ),
                source: "domain_template_resolution".to_string(),
                primary: false,
            }],
            blocking_reason: Some(reason),
        });
    }

    let mut inputs = Vec::new();
    let mut required_stale = Vec::new();
    let mut required_missing = Vec::new();
    let mut used_count = 0usize;
    let mut stale_count = 0usize;
    let mut missing_count = 0usize;
    let mut standard_sources = Vec::new();

    for family in &selected_families {
        let template = effective_templates.get(family).expect("selected family should resolve");
        inputs.push(ContextInput {
            kind: ContextInputKind::DomainTemplate,
            reference: family.as_str().to_string(),
            rationale: format!("selected for bounded target {target_label}"),
            source: format!("domain_template:{}", value_source_text(template.enablement_source)),
            primary: false,
        });

        if let Some(layer) = template.standards_layers.last() {
            standard_sources.push(format!(
                "{} [{}]",
                family.as_str(),
                value_source_text(layer.source)
            ));
        }

        for layer in &template.standards_layers {
            inputs.push(ContextInput {
                kind: ContextInputKind::DomainStandard,
                reference: family.as_str().to_string(),
                rationale: format!(
                    "applied {} standards layer for the bounded target {target_label}",
                    value_source_text(layer.source)
                ),
                source: format!("domain_standard:{}", value_source_text(layer.source)),
                primary: false,
            });
        }

        for binding in &template.external_context_bindings {
            let status = binding.binding.status_for_target(workspace_ref, selected_target);
            match status {
                ExternalContextStatus::Used => used_count += 1,
                ExternalContextStatus::Stale => {
                    stale_count += 1;
                    if binding.binding.required {
                        required_stale.push(binding.binding.reference.clone());
                    }
                }
                ExternalContextStatus::Unavailable => {
                    missing_count += 1;
                    if binding.binding.required {
                        required_missing.push(binding.binding.reference.clone());
                    }
                }
                ExternalContextStatus::Skipped => {}
            }

            inputs.push(domain_binding_input(
                binding.binding.clone(),
                binding.source,
                status,
                target_label,
            ));
        }
    }

    let summary_clause = if standard_sources.is_empty() {
        format!(
            "domain: {}",
            selected_families.iter().map(|family| family.as_str()).collect::<Vec<_>>().join(", ")
        )
    } else {
        format!(
            "domain: {}; standards: {}; external context: used={used_count}, stale={stale_count}, unavailable={missing_count}",
            selected_families.iter().map(|family| family.as_str()).collect::<Vec<_>>().join(", "),
            standard_sources.join(", "),
        )
    };

    if !required_missing.is_empty() {
        return Some(DomainContextOutcome {
            summary_clause,
            credibility: ContextPackCredibility::Insufficient,
            inputs,
            blocking_reason: Some(format!(
                "required external context unavailable for {target_label}: {}",
                required_missing.join(", ")
            )),
        });
    }
    if !required_stale.is_empty() {
        return Some(DomainContextOutcome {
            summary_clause,
            credibility: ContextPackCredibility::Stale,
            inputs,
            blocking_reason: Some(format!(
                "required external context is stale for {target_label}: {}",
                required_stale.join(", ")
            )),
        });
    }

    Some(DomainContextOutcome {
        summary_clause,
        credibility: ContextPackCredibility::Credible,
        inputs,
        blocking_reason: None,
    })
}

fn domain_binding_input(
    binding: ExternalContextBinding,
    source: ValueSource,
    status: ExternalContextStatus,
    target_label: &str,
) -> ContextInput {
    let requirement = if binding.required { "required" } else { "optional" };
    ContextInput {
        kind: ContextInputKind::ExternalContextInput,
        reference: binding.reference,
        rationale: format!(
            "{requirement} {} binding is {} for bounded target {target_label}",
            binding.kind.as_str(),
            status.as_str()
        ),
        source: format!("external_context:{}", value_source_text(source)),
        primary: false,
    }
}

pub fn build_context_pack(
    goal_text: &str,
    workspace_ref: &Path,
    context_sources: &PlanningContextSources,
) -> ContextPack {
    let relevant_files = select_relevant_workspace_files(workspace_ref, goal_text);
    let symbol_hints = extract_symbol_hints(workspace_ref, &relevant_files, goal_text);
    let canon_artifacts = selected_canon_artifacts(workspace_ref, goal_text);
    let canon_memory_targets = context_sources
        .compacted_canon_memory
        .as_ref()
        .map(|memory| memory.artifact_refs.clone())
        .unwrap_or_default();

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

    if let Some(snapshot) = context_sources.canon_capability_snapshot.as_ref() {
        inputs.push(ContextInput {
            kind: ContextInputKind::CanonCapability,
            reference: snapshot.summary_text(),
            rationale: "records the available Canon governance capability surface for planning"
                .to_string(),
            source: "canon_capabilities".to_string(),
            primary: false,
        });
    }

    if let Some(memory) = context_sources.compacted_canon_memory.as_ref() {
        inputs.push(ContextInput {
            kind: ContextInputKind::CanonMemory,
            reference: memory.summary_text(),
            rationale: "reuses compact Canon-grounded memory from prior governed evidence"
                .to_string(),
            source: "compacted_canon_memory".to_string(),
            primary: relevant_files.is_empty() && canon_artifacts.is_empty(),
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

    let selected_target_for_domain = relevant_files
        .first()
        .cloned()
        .or_else(|| canon_memory_targets.first().cloned())
        .or_else(|| canon_artifacts.first().cloned())
        .or_else(|| {
            let primary = select_primary_target(workspace_ref);
            (!primary.is_empty()).then_some(primary)
        });
    let domain_outcome =
        resolve_domain_context(workspace_ref, selected_target_for_domain.as_deref());
    if let Some(domain_outcome) = domain_outcome.as_ref() {
        inputs.extend(domain_outcome.inputs.iter().cloned());
    }

    let has_credible_context = !relevant_files.is_empty()
        || context_sources.authored_input_summary.is_some()
        || context_sources
            .compacted_canon_memory
            .as_ref()
            .is_some_and(|memory| memory.credibility == MemoryCredibilityState::Credible)
        || !canon_artifacts.is_empty()
        || context_sources.latest_trace_ref.is_some();
    let memory_staleness_reason =
        context_sources.compacted_canon_memory.as_ref().and_then(|memory| {
            (memory.credibility != MemoryCredibilityState::Credible)
                .then(|| memory.reason_code.clone().unwrap_or_else(|| memory.headline.clone()))
        });
    let credibility = if !has_credible_context
        || domain_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.credibility == ContextPackCredibility::Insufficient)
    {
        ContextPackCredibility::Insufficient
    } else if memory_staleness_reason.is_some()
        || domain_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.credibility == ContextPackCredibility::Stale)
    {
        ContextPackCredibility::Stale
    } else {
        ContextPackCredibility::Credible
    };
    let mut summary = if has_credible_context {
        format!(
            "bounded context from {} primary input(s)",
            usize::max(relevant_files.len(), canon_artifacts.len().max(canon_memory_targets.len()))
                .max(1)
        )
    } else {
        format!("no credible bounded context found for planning `{}`", goal_text.trim())
    };
    if let Some(domain_outcome) = domain_outcome.as_ref() {
        summary.push_str("; ");
        summary.push_str(&domain_outcome.summary_clause);
        if let Some(reason) = domain_outcome.blocking_reason.as_deref()
            && domain_outcome.credibility != ContextPackCredibility::Credible
        {
            summary.push_str("; ");
            summary.push_str(reason);
        }
    }
    if has_credible_context && let Some(memory) = context_sources.compacted_canon_memory.as_ref() {
        summary.push_str("; ");
        summary.push_str(&memory.summary_text());
    }
    let staleness_reason = if credibility == ContextPackCredibility::Stale {
        let mut reasons = Vec::new();
        if let Some(reason) = memory_staleness_reason.as_ref() {
            reasons.push(reason.clone());
        }
        if let Some(reason) = domain_outcome
            .as_ref()
            .filter(|outcome| outcome.credibility == ContextPackCredibility::Stale)
            .and_then(|outcome| outcome.blocking_reason.clone())
        {
            reasons.push(reason);
        }
        (!reasons.is_empty()).then(|| reasons.join("; "))
    } else {
        None
    };

    ContextPack {
        pack_id: Uuid::new_v4().to_string(),
        summary,
        credibility,
        inputs,
        selected_targets: if !canon_memory_targets.is_empty()
            && (!has_specific_workspace_targets(&relevant_files)
                || context_sources
                    .compacted_canon_memory
                    .as_ref()
                    .is_some_and(|memory| memory.credibility == MemoryCredibilityState::Credible))
        {
            canon_memory_targets
        } else if !relevant_files.is_empty() {
            relevant_files
        } else {
            canon_artifacts
        },
        staleness_reason,
    }
}

fn select_source_target(context_pack: &ContextPack, workspace_ref: &Path) -> String {
    context_pack
        .selected_targets
        .iter()
        .find(|target| target.starts_with("src/"))
        .cloned()
        .or_else(|| {
            context_pack
                .selected_targets
                .iter()
                .find(|target| !target.starts_with("tests/") && !target.starts_with("test/"))
                .cloned()
        })
        .unwrap_or_else(|| select_primary_target(workspace_ref))
}

fn select_test_target(context_pack: &ContextPack) -> Option<String> {
    context_pack
        .selected_targets
        .iter()
        .find(|target| {
            target.starts_with("tests/")
                || target.starts_with("test/")
                || target.starts_with("spec/")
                || target.contains("_test")
        })
        .cloned()
}

fn infer_verification_strategy(
    context_pack: &ContextPack,
    signals: &WorkspaceSignals,
    flow: Option<&InferredFlow>,
    workspace_ref: &Path,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) -> String {
    let source_target = select_source_target(context_pack, workspace_ref);
    if let Some(memory) = compacted_canon_memory
        && memory.credibility == MemoryCredibilityState::Credible
    {
        if let Some(recommended_next_action) = memory.recommended_next_action.as_ref() {
            return format!(
                "follow Canon-guided next action `{}` for {source_target}",
                recommended_next_action.action
            );
        }
        if let Some(mode_summary) = memory.mode_summary.as_ref() {
            return format!("verify against Canon-grounded evidence for {}", mode_summary.headline);
        }
    }

    if let Some(test_target) = select_test_target(context_pack) {
        return format!("run targeted verification against {test_target}");
    }

    if signals.has_tests {
        return match flow.map(|flow| flow.flow_name.as_str()) {
            Some("bug-fix") => {
                format!("run workspace tests covering the bounded fix target {source_target}")
            }
            Some("change") => {
                format!("run workspace tests covering the bounded change target {source_target}")
            }
            Some("delivery") => {
                "run the workspace validation suite before delivery completion".to_string()
            }
            _ => "run the workspace test suite for the proposed change".to_string(),
        };
    }

    format!("review bounded workspace evidence for {source_target}")
}

fn build_planning_rationale(
    context_pack: &ContextPack,
    flow: Option<&InferredFlow>,
    verification_strategy: &str,
    routing_policy_summary: Option<&str>,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) -> String {
    let target_summary = if context_pack.selected_targets.is_empty() {
        "no selected targets".to_string()
    } else {
        context_pack.selected_targets.join(", ")
    };

    let routing_policy_clause = routing_policy_summary
        .map(|summary| format!("; routing policy: {summary}"))
        .unwrap_or_default();
    let canon_memory_clause = compacted_canon_memory
        .map(|memory| format!("; canon memory: {}", memory.summary_text()))
        .unwrap_or_default();

    match flow {
        Some(flow) => format!(
            "{}; selected targets: {target_summary}; verification: {verification_strategy}{}{}",
            flow.confidence_reason, routing_policy_clause, canon_memory_clause
        ),
        None => format!(
            "bounded context selected targets: {target_summary}; verification: {verification_strategy}{}{}",
            routing_policy_clause, canon_memory_clause
        ),
    }
}

fn build_routing_policy_summary(workspace_ref: &Path) -> Option<String> {
    let workspace_routing =
        FileConfigStore::for_workspace(workspace_ref).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace_ref)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();

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

    let mut clauses = Vec::new();
    for (slot, route) in [
        (RouteSlot::Planning, &effective.planning),
        (RouteSlot::Implementation, &effective.implementation),
        (RouteSlot::Verification, &effective.verification),
        (RouteSlot::Review, &effective.review),
    ] {
        let capability = effective_capabilities.get(&route.route.runtime);
        let effort = effective_effort.get(&slot);
        if capability.is_none() && effort.is_none() {
            continue;
        }

        clauses.push(render_routing_policy_clause(slot, route, capability, effort));
    }

    (!clauses.is_empty()).then(|| clauses.join("; "))
}

fn render_routing_policy_clause(
    slot: RouteSlot,
    route: &SourcedRoute,
    capability: Option<&SourcedRuntimeCapabilityProfile>,
    effort: Option<&SourcedSlotEffortPolicy>,
) -> String {
    let mut clause = format!(
        "{} route={}/{} [{}]",
        slot.as_str(),
        route.route.runtime.as_str(),
        route.route.model,
        value_source_text(route.source)
    );

    if let Some(capability) = capability {
        clause.push_str(&format!(
            ", capability={} [{}]",
            capability.profile.summary_text(),
            value_source_text(capability.source)
        ));
    }

    if let Some(effort) = effort {
        clause.push_str(&format!(
            ", effort={} [{}]",
            effort.policy.summary_text(),
            value_source_text(effort.source)
        ));
    }

    clause
}

fn value_source_text(source: ValueSource) -> &'static str {
    match source {
        ValueSource::Cli => "cli",
        ValueSource::Workspace => "workspace",
        ValueSource::Cluster => "cluster",
        ValueSource::Global => "global",
        ValueSource::BuiltIn => "built-in",
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
    inferred_flow: Option<&InferredFlow>,
    verification_strategy: &str,
) -> Vec<PlannedTask> {
    let primary_target = select_source_target(context_pack, workspace_ref);
    let verification_target = if signals.has_tests {
        select_test_target(context_pack).unwrap_or_else(|| "test suite".to_string())
    } else {
        primary_target.clone()
    };
    let flow_name = inferred_flow.map(|flow| flow.flow_name.as_str());
    let implementation_decision = match flow_name {
        Some("bug-fix") => DecisionType::Fix,
        _ => DecisionType::Code,
    };
    let verification_decision =
        if signals.has_tests { DecisionType::Test } else { DecisionType::Analyze };

    let analyze_description = match flow_name {
        Some("bug-fix") => format!("Investigate bounded failure evidence for {goal_text}"),
        Some("delivery") => format!("Assess delivery surface for {goal_text}"),
        _ => format!("Analyze bounded implementation surface for {goal_text}"),
    };
    let implementation_description = match flow_name {
        Some("bug-fix") => format!("Repair bounded implementation for {goal_text}"),
        Some("delivery") => format!("Complete bounded delivery changes for {goal_text}"),
        _ => format!("Implement bounded change for {goal_text}"),
    };

    vec![
        PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: analyze_description,
            target: primary_target.clone(),
            expected_outcome: Some(format!(
                "bounded understanding recorded from context: {}",
                context_pack.summary
            )),
            decision_type_hint: Some(DecisionType::Analyze),
        },
        PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: implementation_description,
            target: primary_target,
            expected_outcome: Some(
                "bounded change applied to the selected implementation surface".to_string(),
            ),
            decision_type_hint: Some(implementation_decision),
        },
        PlannedTask {
            task_id: Uuid::new_v4().to_string(),
            description: format!("Verify changes using {verification_strategy}"),
            target: verification_target,
            expected_outcome: Some("credible bounded verification evidence recorded".to_string()),
            decision_type_hint: Some(verification_decision),
        },
    ]
}

pub fn build_goal_plan_with_sources(
    goal_text: &str,
    workspace_ref: &Path,
    context_sources: &PlanningContextSources,
    preferred_flow: Option<&str>,
) -> Result<GoalPlan, GoalPlannerError> {
    if goal_text.trim().is_empty() {
        return Err(GoalPlannerError::MissingGoal);
    }

    let signals = collect_workspace_signals(workspace_ref);
    let context_pack = build_context_pack(goal_text, workspace_ref, context_sources);
    let inferred_flow = preferred_flow
        .map(|flow_name| InferredFlow {
            flow_name: flow_name.to_string(),
            confidence_reason: format!("operator selected `{flow_name}` before planning"),
            confirmed: false,
        })
        .or_else(|| {
            infer_flow_from_context(&FlowInferenceContext {
                goal_text,
                context_pack: Some(&context_pack),
                workspace_signals: &signals,
                workflow_progress: context_sources.workflow_progress.as_ref(),
            })
        });
    let verification_strategy = infer_verification_strategy(
        &context_pack,
        &signals,
        inferred_flow.as_ref(),
        workspace_ref,
        context_sources.compacted_canon_memory.as_ref(),
    );
    let routing_policy_summary = build_routing_policy_summary(workspace_ref);
    let planning_rationale = build_planning_rationale(
        &context_pack,
        inferred_flow.as_ref(),
        &verification_strategy,
        routing_policy_summary.as_deref(),
        context_sources.compacted_canon_memory.as_ref(),
    );
    let tasks = derive_tasks_from_context(
        goal_text,
        &context_pack,
        workspace_ref,
        &signals,
        inferred_flow.as_ref(),
        &verification_strategy,
    );
    let canon_evidence = scan_canon_artifacts(workspace_ref);
    let mut source_evidence = canon_evidence;
    if let Some(snapshot) = context_sources.canon_capability_snapshot.as_ref() {
        source_evidence
            .push(EvidenceRef::canon(format!("capabilities: {}", snapshot.summary_text())));
    }
    if let Some(memory) = context_sources.compacted_canon_memory.as_ref() {
        source_evidence.push(EvidenceRef::canon(format!("memory: {}", memory.summary_text())));
    }

    let mut plan = GoalPlan::new(goal_text, tasks)
        .map_err(GoalPlannerError::PlanCreation)?
        .with_context_pack(context_pack)
        .with_signals(signals)
        .with_evidence(source_evidence)
        .with_planning_rationale(planning_rationale)
        .with_verification_strategy(verification_strategy);

    if let Some(routing_policy_summary) = routing_policy_summary {
        plan = plan.with_routing_policy_summary(routing_policy_summary);
    }

    if let Some(memory) = context_sources.compacted_canon_memory.clone() {
        plan = plan.with_compacted_canon_memory(memory);
    }

    if let Some(flow) = inferred_flow {
        plan = plan.with_flow(flow);
    }
    if let Some(workflow_progress) = context_sources.workflow_progress.clone() {
        plan = plan.with_workflow_progress(workflow_progress);
    }

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
    build_goal_plan_with_sources(goal_text, workspace_ref, &PlanningContextSources::default(), None)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use std::thread;
    use std::time::Duration;

    use super::{
        PlanningContextSources, build_context_pack, build_goal_plan_with_sources,
        resolve_domain_context,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::configuration::{ConfigFile, RoutingConfig};
    use crate::domain::domain_templates::{
        DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
    };
    use crate::domain::goal_plan::ContextPackCredibility;
    use crate::domain::governance::{
        CanonCapabilitySnapshot, CanonMode, CanonModeSummary, CanonRecommendedActionSummary,
        CanonResultActionSummary, CompactedCanonMemory, MemoryCredibilityState,
    };
    use crate::orchestrator::goal_planner::GoalPlannerError;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("Cargo.toml"), "[package]\nname='planner'\nversion='0.1.0'\n")
            .unwrap();
        workspace
    }

    fn save_local_routing(workspace: &std::path::Path, routing: RoutingConfig) {
        FileConfigStore::for_workspace(workspace)
            .save_local(&ConfigFile { version: 1, routing })
            .unwrap();
    }

    #[test]
    fn build_context_pack_marks_non_credible_canon_memory_as_stale() {
        let workspace = temp_workspace("goal-planner-stale-memory");
        let context_pack = build_context_pack(
            "investigate governed change",
            &workspace,
            &PlanningContextSources {
                compacted_canon_memory: Some(CompactedCanonMemory {
                    headline: "Canon verification memory is stale".to_string(),
                    credibility: MemoryCredibilityState::Stale,
                    stage_key: Some("change:verify".to_string()),
                    run_ref: Some("run-1".to_string()),
                    packet_ref: Some(".canon/runs/run-1".to_string()),
                    reason_code: Some("stale_packet".to_string()),
                    artifact_refs: vec![".canon/runs/run-1/verification.md".to_string()],
                    mode_summary: None,
                    possible_actions: Vec::new(),
                    recommended_next_action: None,
                    evidence_summary: None,
                }),
                ..PlanningContextSources::default()
            },
        );

        assert_eq!(
            context_pack.credibility,
            crate::domain::goal_plan::ContextPackCredibility::Stale
        );
        assert_eq!(context_pack.staleness_reason.as_deref(), Some("stale_packet"));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_goal_plan_with_sources_uses_canon_memory_targets_and_guidance() {
        let workspace = temp_workspace("goal-planner-canon-memory");
        let goal_plan = build_goal_plan_with_sources(
            "verify the governed delivery state",
            &workspace,
            &PlanningContextSources {
                compacted_canon_memory: Some(CompactedCanonMemory {
                    headline: "Canon verification packet remains credible".to_string(),
                    credibility: MemoryCredibilityState::Credible,
                    stage_key: Some("change:verify".to_string()),
                    run_ref: Some("run-2".to_string()),
                    packet_ref: Some(".canon/runs/run-2".to_string()),
                    reason_code: None,
                    artifact_refs: vec![".canon/runs/run-2/verification.md".to_string()],
                    mode_summary: Some(CanonModeSummary {
                        headline: "Verification packet ready".to_string(),
                        artifact_packet_summary: "Primary artifact is ready.".to_string(),
                        execution_posture: Some("recommendation-only".to_string()),
                        primary_artifact_title: "Verification".to_string(),
                        primary_artifact_path: ".canon/runs/run-2/verification.md".to_string(),
                        primary_artifact_action: CanonResultActionSummary {
                            label: "inspect".to_string(),
                            target: ".canon/runs/run-2/verification.md".to_string(),
                        },
                        result_excerpt: "No direct contradiction was found.".to_string(),
                        action_chip_labels: vec!["inspect".to_string()],
                    }),
                    possible_actions: Vec::new(),
                    recommended_next_action: Some(CanonRecommendedActionSummary {
                        action: "inspect".to_string(),
                        rationale: "Review the verification packet before continuing".to_string(),
                        target: Some(".canon/runs/run-2/verification.md".to_string()),
                    }),
                    evidence_summary: None,
                }),
                ..PlanningContextSources::default()
            },
            None,
        )
        .unwrap();

        assert_eq!(goal_plan.tasks[0].target, ".canon/runs/run-2/verification.md".to_string());
        assert!(
            goal_plan
                .verification_strategy
                .as_deref()
                .unwrap()
                .contains("Canon-guided next action `inspect`")
        );
        assert!(goal_plan.context_summary().unwrap().contains("canon memory"));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_goal_plan_with_sources_records_capabilities_and_mode_summary_guidance() {
        let workspace = temp_workspace("goal-planner-canon-capabilities");
        let goal_plan = build_goal_plan_with_sources(
            "verify the governed delivery state",
            &workspace,
            &PlanningContextSources {
                canon_capability_snapshot: Some(CanonCapabilitySnapshot {
                    canon_version: "0.39.0".to_string(),
                    supported_schema_versions: vec!["2026-02-01".to_string()],
                    operations: vec![
                        "start".to_string(),
                        "refresh".to_string(),
                        "capabilities".to_string(),
                    ],
                    supported_modes: vec![CanonMode::Verification],
                    status_values: vec!["governed_ready".to_string()],
                    approval_state_values: vec!["not_needed".to_string()],
                    packet_readiness_values: vec!["reusable".to_string()],
                    compatibility_notes: vec!["stable-json".to_string()],
                }),
                compacted_canon_memory: Some(CompactedCanonMemory {
                    headline: "Canon verification packet remains credible".to_string(),
                    credibility: MemoryCredibilityState::Credible,
                    stage_key: Some("change:verify".to_string()),
                    run_ref: Some("run-3".to_string()),
                    packet_ref: Some(".canon/runs/run-3".to_string()),
                    reason_code: None,
                    artifact_refs: vec![".canon/runs/run-3/verification.md".to_string()],
                    mode_summary: Some(CanonModeSummary {
                        headline: "Verification packet ready".to_string(),
                        artifact_packet_summary: "Primary artifact is ready.".to_string(),
                        execution_posture: None,
                        primary_artifact_title: "Verification".to_string(),
                        primary_artifact_path: ".canon/runs/run-3/verification.md".to_string(),
                        primary_artifact_action: CanonResultActionSummary {
                            label: "inspect".to_string(),
                            target: ".canon/runs/run-3/verification.md".to_string(),
                        },
                        result_excerpt: "No contradiction was found.".to_string(),
                        action_chip_labels: vec!["inspect".to_string()],
                    }),
                    possible_actions: Vec::new(),
                    recommended_next_action: None,
                    evidence_summary: None,
                }),
                ..PlanningContextSources::default()
            },
            None,
        )
        .unwrap();

        assert!(
            goal_plan
                .context_pack
                .as_ref()
                .unwrap()
                .inputs
                .iter()
                .any(|input| input.kind
                    == crate::domain::goal_plan::ContextInputKind::CanonCapability)
        );
        assert_eq!(goal_plan.tasks[0].target, ".canon/runs/run-3/verification.md");
        assert_eq!(
            goal_plan.verification_strategy.as_deref(),
            Some("verify against Canon-grounded evidence for Verification packet ready")
        );
        assert!(
            goal_plan
                .source_evidence
                .iter()
                .any(|entry| entry.reference.contains("Canon 0.39.0 capabilities available"))
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_goal_plan_with_sources_surfaces_selected_domain_context() {
        let workspace = temp_workspace("goal-planner-domain-context");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();
        fs::write(
            workspace.join("src/components/App.tsx"),
            "export function App() { return <div />; }\n",
        )
        .unwrap();
        fs::write(workspace.join("design/reference.md"), "button guidance\n").unwrap();
        save_local_routing(
            &workspace,
            RoutingConfig {
                domain_templates: BTreeMap::from([(
                    DomainFamily::React,
                    DomainTemplateSettings {
                        enabled: Some(true),
                        standards: Some("workspace react standards".to_string()),
                        external_context_bindings: vec![ExternalContextBinding {
                            kind: ExternalContextKind::DesignReference,
                            reference: "design/reference.md".to_string(),
                            required: true,
                            notes: None,
                        }],
                    },
                )]),
                ..RoutingConfig::default()
            },
        );

        let plan = build_goal_plan_with_sources(
            "update the react component",
            &workspace,
            &PlanningContextSources::default(),
            None,
        )
        .unwrap();

        assert!(plan.context_summary().unwrap().contains("domain: react"));
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("domain_template: react"))
        );
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("external_context_input: design/reference.md"))
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_goal_plan_with_sources_blocks_when_required_domain_input_is_missing() {
        let workspace = temp_workspace("goal-planner-domain-missing-binding");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::write(
            workspace.join("src/components/App.tsx"),
            "export function App() { return <div />; }\n",
        )
        .unwrap();
        save_local_routing(
            &workspace,
            RoutingConfig {
                domain_templates: BTreeMap::from([(
                    DomainFamily::React,
                    DomainTemplateSettings {
                        enabled: Some(true),
                        standards: Some("workspace react standards".to_string()),
                        external_context_bindings: vec![ExternalContextBinding {
                            kind: ExternalContextKind::DesignReference,
                            reference: "design/missing.md".to_string(),
                            required: true,
                            notes: None,
                        }],
                    },
                )]),
                ..RoutingConfig::default()
            },
        );

        let error = build_goal_plan_with_sources(
            "update the react component",
            &workspace,
            &PlanningContextSources::default(),
            None,
        )
        .unwrap_err();

        match error {
            GoalPlannerError::InsufficientContext { summary, goal_plan } => {
                assert!(summary.contains("required external context unavailable"));
                assert!(
                    goal_plan
                        .context_provenance_lines()
                        .iter()
                        .any(|line| line.contains("external_context_input: design/missing.md"))
                );
            }
            other => panic!("unexpected planner error: {other}"),
        }

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn resolve_domain_context_reports_no_enabled_matching_family() {
        let workspace = temp_workspace("goal-planner-domain-mismatch");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::write(
            workspace.join("src/components/App.tsx"),
            "export function App() { return <div />; }\n",
        )
        .unwrap();
        save_local_routing(
            &workspace,
            RoutingConfig {
                domain_templates: BTreeMap::from([(
                    DomainFamily::React,
                    DomainTemplateSettings {
                        enabled: Some(false),
                        standards: None,
                        external_context_bindings: Vec::new(),
                    },
                )]),
                ..RoutingConfig::default()
            },
        );

        let outcome = resolve_domain_context(&workspace, Some("src/components/App.tsx")).unwrap();

        assert_eq!(outcome.credibility, ContextPackCredibility::Insufficient);
        assert!(outcome.summary_clause.contains("domain context unavailable"));
        assert!(outcome.blocking_reason.as_deref().unwrap().contains("configured families: none"));
        assert_eq!(outcome.inputs.len(), 1);
        assert_eq!(outcome.inputs[0].reference, "none");

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn resolve_domain_context_marks_required_binding_as_stale() {
        let workspace = temp_workspace("goal-planner-domain-stale-binding");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("src/components")).unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();
        fs::write(workspace.join("design/reference.md"), "button guidance\n").unwrap();
        thread::sleep(Duration::from_millis(20));
        fs::write(
            workspace.join("src/components/App.tsx"),
            "export function App() { return <div />; }\n",
        )
        .unwrap();

        save_local_routing(
            &workspace,
            RoutingConfig {
                domain_templates: BTreeMap::from([(
                    DomainFamily::React,
                    DomainTemplateSettings {
                        enabled: Some(true),
                        standards: Some("workspace react standards".to_string()),
                        external_context_bindings: vec![ExternalContextBinding {
                            kind: ExternalContextKind::DesignReference,
                            reference: "design/reference.md".to_string(),
                            required: true,
                            notes: None,
                        }],
                    },
                )]),
                ..RoutingConfig::default()
            },
        );

        let outcome = resolve_domain_context(&workspace, Some("src/components/App.tsx")).unwrap();

        assert_eq!(outcome.credibility, ContextPackCredibility::Stale);
        assert!(outcome.summary_clause.contains("domain: react"));
        assert!(outcome.summary_clause.contains("stale=1"));
        assert!(
            outcome
                .blocking_reason
                .as_deref()
                .unwrap()
                .contains("required external context is stale")
        );
        assert!(outcome.inputs.iter().any(|input| {
            input.kind == crate::domain::goal_plan::ContextInputKind::ExternalContextInput
                && input.reference == "design/reference.md"
        }));

        fs::remove_dir_all(workspace).unwrap();
    }
}
