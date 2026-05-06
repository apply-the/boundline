//! Authored brief ingestion for feature 010.
//!
//! Normalizes developer-supplied free text and Markdown brief files into a
//! single [`AuthoredBriefBundle`] that can be projected into the existing
//! orchestrator goal pipeline. The current slice accepts direct text, explicit
//! Markdown brief files, and Markdown paths referenced from the direct text.
//! Governance intent, derived task drafts, and explicit clarification records
//! are persisted with the normalized bundle so later status and inspect
//! surfaces can explain why planning may continue or stop.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::governance::{CanonMode, GovernanceRuntimeKind};
use crate::domain::task::{
    ClarificationReasonKind, ClarificationRecord, ClarificationStatus, DerivedTaskDraft,
};
use crate::domain::trace::current_timestamp_millis;

/// Maximum number of Markdown brief sources accepted per capture invocation.
pub const MAX_BRIEF_SOURCES: usize = 10;
/// Maximum size in bytes for a single Markdown brief source.
pub const MAX_BRIEF_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoredBriefResolutionState {
    Captured,
    ClarificationRequired,
    Ready,
}

fn default_resolution_state() -> AuthoredBriefResolutionState {
    AuthoredBriefResolutionState::Ready
}

/// Normalized representation of human-authored input gathered from the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredBriefBundle {
    pub bundle_id: String,
    pub primary_goal_text: Option<String>,
    pub sources: Vec<InputSourceReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deduplicated_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_intent: Option<GovernanceIntent>,
    #[serde(default = "default_resolution_state")]
    pub resolution_state: AuthoredBriefResolutionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification: Option<ClarificationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_task_draft: Option<DerivedTaskDraft>,
    pub captured_at: u64,
}

impl AuthoredBriefBundle {
    /// Concatenated goal text suitable for the existing capture pipeline.
    ///
    /// The optional direct goal text is emitted first (when present) followed
    /// by each Markdown brief in deterministic precedence order, separated by
    /// stable headers that callers can use to recover provenance.
    pub fn render_goal_text(&self) -> String {
        let mut buffer = String::new();
        if let Some(text) = self.primary_goal_text.as_deref() {
            buffer.push_str(text.trim());
        }

        for source in &self.sources {
            if matches!(source.kind, InputSourceKind::DirectText) {
                continue;
            }
            if !buffer.is_empty() {
                buffer.push_str("\n\n");
            }
            buffer.push_str("## ");
            buffer.push_str(&source.display_name);
            buffer.push('\n');
            buffer.push_str(source.content.trim());
        }

        buffer
    }

    /// Number of Markdown sources included in the bundle.
    pub fn markdown_source_count(&self) -> usize {
        self.sources
            .iter()
            .filter(|source| !matches!(source.kind, InputSourceKind::DirectText))
            .count()
    }

    /// Compact human-facing summary of the accepted authored input.
    pub fn summary_text(&self) -> String {
        let has_direct_text =
            self.primary_goal_text.as_deref().map(str::trim).is_some_and(|text| !text.is_empty());
        let markdown_sources = self.markdown_source_count();

        match (has_direct_text, markdown_sources) {
            (true, 0) => "direct_text only".to_string(),
            (true, count) => format!("direct_text + {count} markdown source(s)"),
            (false, count) => format!("{count} markdown source(s)"),
        }
    }

    /// Ordered source labels suitable for compact status and inspect output.
    pub fn ordered_source_labels(&self) -> Vec<String> {
        self.sources.iter().map(InputSourceReference::display_label).collect()
    }

    /// Canonical workspace-relative source paths that were repeated and collapsed.
    pub fn deduplicated_source_labels(&self) -> Vec<String> {
        self.deduplicated_sources.clone()
    }

    pub fn planning_ready(&self) -> bool {
        self.derived_task_draft.as_ref().is_some_and(|draft| draft.planning_ready)
    }

    pub fn clarification_headline(&self) -> Option<String> {
        self.clarification.as_ref().map(ClarificationRecord::headline)
    }

    pub fn clarification_prompt(&self) -> Option<String> {
        self.clarification.as_ref().map(|clarification| clarification.prompt.clone())
    }

    pub fn clarification_missing_fields(&self) -> Option<Vec<String>> {
        self.clarification.as_ref().and_then(|clarification| {
            (!clarification.missing_fields.is_empty())
                .then_some(clarification.missing_fields.clone())
        })
    }
}

/// Human-facing governance intent supplied alongside authored input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceIntent {
    pub requested: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_preference: Option<GovernanceRuntimeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explicit_mode: Option<CanonMode>,
    #[serde(default)]
    pub explicit_no_canon: bool,
}

/// Provenance entry for a single normalized input source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSourceReference {
    pub source_id: String,
    pub kind: InputSourceKind,
    pub display_name: String,
    pub workspace_path: Option<String>,
    pub precedence: usize,
    pub content: String,
}

impl InputSourceReference {
    pub fn display_label(&self) -> String {
        let kind = match self.kind {
            InputSourceKind::DirectText => "direct_text",
            InputSourceKind::AttachedMarkdown => "attached_markdown",
            InputSourceKind::ReferencedMarkdown => "referenced_markdown",
        };

        match self.workspace_path.as_deref() {
            Some(path) => format!("{kind}: {path}"),
            None => format!("{kind}: {}", self.display_name),
        }
    }
}

/// Kind of input source captured by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSourceKind {
    DirectText,
    AttachedMarkdown,
    ReferencedMarkdown,
}

/// Failures encountered while normalizing developer-supplied input.
#[derive(Debug, Error)]
pub enum BriefIngestionError {
    #[error("at least one of --goal or --brief is required")]
    NoInputProvided,
    #[error("governance field `{field}` is required when --governance {runtime} is requested")]
    MissingGovernanceField { field: &'static str, runtime: GovernanceRuntimeKind },
    #[error("human input accepts at most {MAX_BRIEF_SOURCES} Markdown sources, got {0}")]
    TooManySources(usize),
    #[error("brief source `{path}` is missing")]
    MissingSource { path: PathBuf },
    #[error("brief source `{path}` is not a regular file")]
    NotARegularFile { path: PathBuf },
    #[error("brief source `{path}` must be inside the workspace `{workspace}`")]
    OutsideWorkspace { path: PathBuf, workspace: PathBuf },
    #[error("brief source `{path}` must use the .md or .markdown extension")]
    UnsupportedExtension { path: PathBuf },
    #[error("brief source `{path}` exceeds the {MAX_BRIEF_BYTES}-byte limit ({size} bytes)")]
    SourceTooLarge { path: PathBuf, size: u64 },
    #[error("brief source `{path}` is empty")]
    EmptySource { path: PathBuf },
    #[error("failed to read brief source `{path}`: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to canonicalize workspace `{path}`: {source}")]
    InvalidWorkspace {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Normalize developer-supplied direct text and Markdown brief paths.
///
/// `workspace` MUST exist on disk (the CLI resolves it before invoking this
/// helper). Brief paths can be absolute or relative; they are resolved
/// against the canonical workspace and rejected if they escape it.
pub fn normalize_inputs(
    workspace: &Path,
    direct_text: Option<&str>,
    brief_paths: &[PathBuf],
) -> Result<AuthoredBriefBundle, BriefIngestionError> {
    normalize_inputs_with_governance(workspace, direct_text, brief_paths, None)
}

pub fn normalize_inputs_with_governance(
    workspace: &Path,
    direct_text: Option<&str>,
    brief_paths: &[PathBuf],
    governance_intent: Option<GovernanceIntent>,
) -> Result<AuthoredBriefBundle, BriefIngestionError> {
    let trimmed_text =
        direct_text.map(str::trim).filter(|text| !text.is_empty()).map(str::to_string);
    let referenced_paths =
        trimmed_text.as_deref().map(referenced_markdown_paths).unwrap_or_default();

    if trimmed_text.is_none() && brief_paths.is_empty() && referenced_paths.is_empty() {
        return Err(BriefIngestionError::NoInputProvided);
    }

    if brief_paths.len() > MAX_BRIEF_SOURCES {
        return Err(BriefIngestionError::TooManySources(brief_paths.len()));
    }

    let canonical_workspace = workspace.canonicalize().map_err(|source| {
        BriefIngestionError::InvalidWorkspace { path: workspace.to_path_buf(), source }
    })?;

    let mut sources = Vec::with_capacity(
        brief_paths.len() + referenced_paths.len() + usize::from(trimmed_text.is_some()),
    );
    let mut deduplicated_sources = Vec::new();
    let mut precedence = 0usize;
    let mut accepted_workspace_paths = HashSet::new();

    if let Some(text) = trimmed_text.as_ref() {
        sources.push(InputSourceReference {
            source_id: format!("direct-{precedence}"),
            kind: InputSourceKind::DirectText,
            display_name: "developer goal".to_string(),
            workspace_path: None,
            precedence,
            content: text.clone(),
        });
        precedence += 1;
    }

    for raw_path in brief_paths {
        push_markdown_source(
            &mut sources,
            &mut deduplicated_sources,
            &mut accepted_workspace_paths,
            &canonical_workspace,
            raw_path,
            InputSourceKind::AttachedMarkdown,
            &mut precedence,
        )?;
    }

    for raw_path in &referenced_paths {
        push_markdown_source(
            &mut sources,
            &mut deduplicated_sources,
            &mut accepted_workspace_paths,
            &canonical_workspace,
            raw_path,
            InputSourceKind::ReferencedMarkdown,
            &mut precedence,
        )?;
    }

    let markdown_source_count =
        sources.iter().filter(|source| !matches!(source.kind, InputSourceKind::DirectText)).count();
    if markdown_source_count > MAX_BRIEF_SOURCES {
        return Err(BriefIngestionError::TooManySources(markdown_source_count));
    }

    let bundle_id = Uuid::new_v4().to_string();
    let captured_at = current_timestamp_millis();
    let mut bundle = AuthoredBriefBundle {
        bundle_id,
        primary_goal_text: trimmed_text,
        sources,
        deduplicated_sources,
        governance_intent,
        resolution_state: AuthoredBriefResolutionState::Ready,
        clarification: None,
        derived_task_draft: None,
        captured_at,
    };

    let (derived_task_draft, clarification) = derive_task_draft(&bundle);
    bundle.resolution_state = if clarification.is_some() {
        AuthoredBriefResolutionState::ClarificationRequired
    } else {
        AuthoredBriefResolutionState::Ready
    };
    bundle.clarification = clarification;
    bundle.derived_task_draft = Some(derived_task_draft);

    Ok(bundle)
}

pub fn normalize_governance_intent(
    runtime_preference: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<Option<GovernanceIntent>, BriefIngestionError> {
    let risk = trimmed_field(risk);
    let zone = trimmed_field(zone);
    let owner = trimmed_field(owner);

    if runtime_preference.is_none() && risk.is_none() && zone.is_none() && owner.is_none() {
        return Ok(None);
    }

    if runtime_preference == Some(GovernanceRuntimeKind::Canon) {
        if risk.is_none() {
            return Err(BriefIngestionError::MissingGovernanceField {
                field: "risk",
                runtime: GovernanceRuntimeKind::Canon,
            });
        }
        if zone.is_none() {
            return Err(BriefIngestionError::MissingGovernanceField {
                field: "zone",
                runtime: GovernanceRuntimeKind::Canon,
            });
        }
        if owner.is_none() {
            return Err(BriefIngestionError::MissingGovernanceField {
                field: "owner",
                runtime: GovernanceRuntimeKind::Canon,
            });
        }
    }

    Ok(Some(GovernanceIntent {
        requested: true,
        runtime_preference,
        risk,
        zone,
        owner,
        explicit_mode: None,
        explicit_no_canon: false,
    }))
}

fn trimmed_field(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn push_markdown_source(
    sources: &mut Vec<InputSourceReference>,
    deduplicated_sources: &mut Vec<String>,
    accepted_workspace_paths: &mut HashSet<String>,
    canonical_workspace: &Path,
    raw_path: &Path,
    kind: InputSourceKind,
    precedence: &mut usize,
) -> Result<(), BriefIngestionError> {
    let resolved = resolve_markdown_source(canonical_workspace, raw_path)?;
    if !accepted_workspace_paths.insert(resolved.workspace_relative.clone()) {
        if !deduplicated_sources.iter().any(|path| path == &resolved.workspace_relative) {
            deduplicated_sources.push(resolved.workspace_relative);
        }
        return Ok(());
    }

    sources.push(InputSourceReference {
        source_id: format!("brief-{}", *precedence),
        kind,
        display_name: resolved.display_name,
        workspace_path: Some(resolved.workspace_relative),
        precedence: *precedence,
        content: resolved.contents,
    });
    *precedence += 1;
    Ok(())
}

fn resolve_markdown_source(
    canonical_workspace: &Path,
    raw_path: &Path,
) -> Result<ResolvedMarkdownSource, BriefIngestionError> {
    let candidate = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        canonical_workspace.join(raw_path)
    };

    if !candidate.exists() {
        return Err(BriefIngestionError::MissingSource { path: candidate });
    }

    let canonical = candidate
        .canonicalize()
        .map_err(|source| BriefIngestionError::ReadFailed { path: candidate.clone(), source })?;

    if !canonical.is_file() {
        return Err(BriefIngestionError::NotARegularFile { path: canonical });
    }

    if !canonical.starts_with(canonical_workspace) {
        return Err(BriefIngestionError::OutsideWorkspace {
            path: canonical,
            workspace: canonical_workspace.to_path_buf(),
        });
    }

    let extension = canonical.extension().and_then(|ext| ext.to_str()).map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("md") | Some("markdown") => {}
        _ => {
            return Err(BriefIngestionError::UnsupportedExtension { path: canonical });
        }
    }

    let metadata = fs::metadata(&canonical)
        .map_err(|source| BriefIngestionError::ReadFailed { path: canonical.clone(), source })?;
    let size = metadata.len();
    if size as usize > MAX_BRIEF_BYTES {
        return Err(BriefIngestionError::SourceTooLarge { path: canonical, size });
    }

    let contents = fs::read_to_string(&canonical)
        .map_err(|source| BriefIngestionError::ReadFailed { path: canonical.clone(), source })?;
    if contents.trim().is_empty() {
        return Err(BriefIngestionError::EmptySource { path: canonical });
    }

    let workspace_relative = canonical
        .strip_prefix(canonical_workspace)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| canonical.to_string_lossy().into_owned());
    let display_name = canonical
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| workspace_relative.clone());

    Ok(ResolvedMarkdownSource { workspace_relative, display_name, contents })
}

fn derive_task_draft(
    bundle: &AuthoredBriefBundle,
) -> (DerivedTaskDraft, Option<ClarificationRecord>) {
    let bounded_goal = extract_bounded_goal(bundle);
    let clarification = clarification_for_bundle(bundle, &bounded_goal);
    let blocking_clarification_ref =
        clarification.as_ref().map(|record| record.clarification_id.clone());

    (
        DerivedTaskDraft {
            draft_id: Uuid::new_v4().to_string(),
            bundle_id: bundle.bundle_id.clone(),
            bounded_goal: bounded_goal.clone(),
            flow_hint: derive_flow_hint(&bounded_goal),
            planning_ready: clarification.is_none(),
            validation_targets: bundle
                .sources
                .iter()
                .filter_map(|source| source.workspace_path.clone())
                .collect(),
            blocking_clarification_ref,
        },
        clarification,
    )
}

fn extract_bounded_goal(bundle: &AuthoredBriefBundle) -> String {
    if let Some(goal) = bundle.primary_goal_text.as_deref().map(str::trim)
        && !goal.is_empty()
    {
        return goal.to_string();
    }

    bundle
        .sources
        .iter()
        .filter(|source| !matches!(source.kind, InputSourceKind::DirectText))
        .find_map(|source| {
            source
                .content
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(|line| line.trim_start_matches('#').trim().to_string())
        })
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| bundle.render_goal_text())
}

fn clarification_for_bundle(
    bundle: &AuthoredBriefBundle,
    bounded_goal: &str,
) -> Option<ClarificationRecord> {
    if !requires_unbounded_request_clarification(bounded_goal) {
        return None;
    }

    Some(ClarificationRecord {
        clarification_id: Uuid::new_v4().to_string(),
        reason_kind: ClarificationReasonKind::UnboundedRequest,
        prompt: "Narrow the request to one bounded bug-fix, change, or delivery outcome. Name the single document, component, or failing behavior Boundline should address before planning continues.".to_string(),
        missing_fields: vec!["bounded_scope".to_string()],
        blocking_sources: bundle.sources.iter().map(|source| source.source_id.clone()).collect(),
        turn_index: 1,
        status: ClarificationStatus::Open,
    })
}

fn requires_unbounded_request_clarification(goal: &str) -> bool {
    let lower = goal.to_ascii_lowercase();
    lower.contains("whatever")
        || lower.contains("anything")
        || lower.contains("everything")
        || (lower.starts_with("improve ") && lower.contains(" and "))
}

fn derive_flow_hint(goal: &str) -> Option<String> {
    let lower = goal.to_ascii_lowercase();
    if lower.contains("bug") || lower.contains("fix") || lower.contains("failing test") {
        return Some("bug-fix".to_string());
    }
    if lower.contains("change") || lower.contains("update") || lower.contains("prepare") {
        return Some("change".to_string());
    }
    if lower.contains("release") || lower.contains("ship") || lower.contains("deliver") {
        return Some("delivery".to_string());
    }
    None
}

fn referenced_markdown_paths(text: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();

    for token in text.split_whitespace() {
        let Some(path) = markdown_reference_from_token(token) else {
            continue;
        };
        let key = path.to_string_lossy().into_owned();
        if seen.insert(key.clone()) {
            paths.push(PathBuf::from(key));
        }
    }

    paths
}

fn markdown_reference_from_token(token: &str) -> Option<PathBuf> {
    let trimmed = token
        .trim_start_matches(['"', '\'', '(', '[', '{', '<'])
        .trim_end_matches(['"', '\'', ')', ']', '}', '>', ',', ';', ':', '!', '?', '.']);
    if trimmed.is_empty() || trimmed.contains("://") {
        return None;
    }

    let lowercase = trimmed.to_ascii_lowercase();
    if lowercase.ends_with(".md") || lowercase.ends_with(".markdown") {
        Some(PathBuf::from(trimmed))
    } else {
        None
    }
}

struct ResolvedMarkdownSource {
    workspace_relative: String,
    display_name: String,
    contents: String,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::*;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn rejects_invocation_without_goal_or_briefs() {
        let workspace = temp_workspace("boundline-brief-empty");
        let error = normalize_inputs(&workspace, None, &[]).unwrap_err();
        assert!(matches!(error, BriefIngestionError::NoInputProvided));
    }

    #[test]
    fn normalizes_direct_text_only() {
        let workspace = temp_workspace("boundline-brief-direct");
        let bundle =
            normalize_inputs(&workspace, Some("  Fix the failing add test  "), &[]).unwrap();
        assert_eq!(bundle.primary_goal_text.as_deref(), Some("Fix the failing add test"));
        assert_eq!(bundle.markdown_source_count(), 0);
        assert_eq!(bundle.render_goal_text(), "Fix the failing add test");
    }

    #[test]
    fn ingests_markdown_brief_from_workspace() {
        let workspace = temp_workspace("boundline-brief-md");
        let brief = workspace.join("brief.md");
        fs::write(&brief, "# Goal\nReplace subtraction with addition\n").unwrap();

        let bundle = normalize_inputs(&workspace, None, std::slice::from_ref(&brief)).unwrap();
        assert_eq!(bundle.markdown_source_count(), 1);
        let goal = bundle.render_goal_text();
        assert!(goal.contains("## brief.md"));
        assert!(goal.contains("Replace subtraction with addition"));
    }

    #[test]
    fn rejects_brief_outside_workspace() {
        let workspace = temp_workspace("boundline-brief-out-ws");
        let foreign = temp_workspace("boundline-brief-out-foreign");
        let brief = foreign.join("brief.md");
        fs::write(&brief, "outside\n").unwrap();
        let error = normalize_inputs(&workspace, None, &[brief]).unwrap_err();
        assert!(matches!(error, BriefIngestionError::OutsideWorkspace { .. }));
    }

    #[test]
    fn rejects_unsupported_extension() {
        let workspace = temp_workspace("boundline-brief-ext");
        let brief = workspace.join("brief.txt");
        fs::write(&brief, "nope\n").unwrap();
        let error = normalize_inputs(&workspace, None, &[brief]).unwrap_err();
        assert!(matches!(error, BriefIngestionError::UnsupportedExtension { .. }));
    }

    #[test]
    fn rejects_missing_source() {
        let workspace = temp_workspace("boundline-brief-missing");
        let error =
            normalize_inputs(&workspace, None, &[workspace.join("missing.md")]).unwrap_err();
        assert!(matches!(error, BriefIngestionError::MissingSource { .. }));
    }

    #[test]
    fn rejects_empty_source() {
        let workspace = temp_workspace("boundline-brief-empty-src");
        let brief = workspace.join("empty.md");
        fs::write(&brief, "   \n").unwrap();
        let error = normalize_inputs(&workspace, None, &[brief]).unwrap_err();
        assert!(matches!(error, BriefIngestionError::EmptySource { .. }));
    }

    #[test]
    fn rejects_too_many_sources() {
        let workspace = temp_workspace("boundline-brief-too-many");
        let mut paths = Vec::new();
        for i in 0..(MAX_BRIEF_SOURCES + 1) {
            let path = workspace.join(format!("brief-{i}.md"));
            fs::write(&path, format!("brief {i}\n")).unwrap();
            paths.push(path);
        }
        let error = normalize_inputs(&workspace, None, &paths).unwrap_err();
        assert!(
            matches!(error, BriefIngestionError::TooManySources(n) if n == MAX_BRIEF_SOURCES + 1)
        );
    }

    #[test]
    fn combines_direct_text_and_markdown_briefs() {
        let workspace = temp_workspace("boundline-brief-combo");
        let brief = workspace.join("plan.md");
        fs::write(&brief, "Step 1: investigate\nStep 2: fix\n").unwrap();
        let bundle = normalize_inputs(&workspace, Some("Goal: deliver fix"), &[brief]).unwrap();

        assert_eq!(bundle.sources.len(), 2);
        assert_eq!(bundle.sources[0].kind, InputSourceKind::DirectText);
        assert_eq!(bundle.sources[1].kind, InputSourceKind::AttachedMarkdown);
        let rendered = bundle.render_goal_text();
        assert!(rendered.starts_with("Goal: deliver fix"));
        assert!(rendered.contains("## plan.md"));
        assert!(rendered.contains("Step 2: fix"));
    }
}
