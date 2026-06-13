//! Typed completion-verification models and deterministic runtime helpers.
//!
//! This module keeps completion-verification state additive to the existing
//! task/session lifecycle. It defines the bounded claim, proof, freshness, and
//! projection vocabulary plus first-slice helpers for proof selection,
//! confirmation policy, and workspace fingerprint invalidation.

use std::collections::{BTreeMap, BTreeSet, hash_map::DefaultHasher};
use std::ffi::OsStr;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::trace::current_timestamp_millis;

const FINGERPRINT_PATH_SAMPLE_LIMIT: usize = 10;
const FINGERPRINT_PATH_TRUNCATION_MARKER: &str = "[truncated]";
const PATH_SEPARATOR: char = '/';
const GIT_DIRECTORY_NAME: &str = ".git";
const BOUNDLINE_DIRECTORY_NAME: &str = ".boundline";
const BOUNDLINE_TRACES_PATH: &str = ".boundline/traces";
const BOUNDLINE_ARTIFACTS_PATH: &str = ".boundline/artifacts";
const BOUNDLINE_CACHE_PATH: &str = ".boundline/cache";
const VOLATILE_DIRECTORY_NAMES: &[&str] =
    &["target", "node_modules", "dist", "build", ".next", ".venv"];
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "jsx", "ts", "tsx", "go", "java", "kt", "swift", "c", "cc", "cpp", "h",
    "hpp", "rb", "php", "cs",
];
const CONFIG_EXTENSIONS: &[&str] = &["toml", "yaml", "yml", "json", "jsonc", "ini", "conf"];
const TEST_EXTENSIONS: &[&str] = &["rs", "py", "js", "jsx", "ts", "tsx", "go", "java", "kt"];
const DOCUMENTATION_EXTENSIONS: &[&str] = &["md", "mdx", "rst", "adoc", "txt"];
const BUILD_FILE_NAMES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "settings.gradle.kts",
    "Makefile",
    "Justfile",
    "justfile",
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "requirements.txt",
    "build.rs",
];
const DOCUMENTATION_FILE_NAMES: &[&str] = &["README.md", "CHANGELOG.md"];
const TEST_KEYWORD: &str = "test";
const BUG_KEYWORDS: &[&str] = &["bug", "fix", "issue", "regression"];
const BUILD_KEYWORD: &str = "build";
const MIGRATION_KEYWORD: &str = "migration";
const VERIFICATION_KEYWORDS: &[&str] = &["verify", "verification", "evidence", "pass"];

/// Top-level completion-verification states surfaced to operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionVerificationState {
    Ready,
    ProofRequired,
    Blocked,
    Failed,
}

impl CompletionVerificationState {
    /// Returns the stable wire text for the state value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ProofRequired => "proof_required",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

/// Scope of the completion-verification projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionVerificationScope {
    Task,
    Stage,
    Run,
}

/// First-slice claim kinds that Boundline can prove.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionClaimKind {
    TestsPass,
    BugFixed,
    BuildClean,
    MigrationValid,
}

impl CompletionClaimKind {
    /// Returns the stable wire text for the claim kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TestsPass => "tests_pass",
            Self::BugFixed => "bug_fixed",
            Self::BuildClean => "build_clean",
            Self::MigrationValid => "migration_valid",
        }
    }
}

/// Source that produced the active completion claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionClaimSource {
    ExplicitMetadata,
    RuntimeInference,
    OperatorConfirmed,
    OperatorOverride,
}

impl CompletionClaimSource {
    /// Returns the stable wire text for the claim source.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitMetadata => "explicit_metadata",
            Self::RuntimeInference => "runtime_inference",
            Self::OperatorConfirmed => "operator_confirmed",
            Self::OperatorOverride => "operator_override",
        }
    }
}

/// Confidence attached to an inferred claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimInferenceConfidence {
    High,
    Medium,
    Low,
}

/// Required next action carried by a blocking completion-verification finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionRequiredAction {
    RunProof,
    RerunProof,
    ConfirmClaim,
    OverrideClaim,
    ResolveConflict,
    ClarifyClaim,
}

impl CompletionRequiredAction {
    /// Returns the stable wire text for the required action.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RunProof => "run_proof",
            Self::RerunProof => "rerun_proof",
            Self::ConfirmClaim => "confirm_claim",
            Self::OverrideClaim => "override_claim",
            Self::ResolveConflict => "resolve_conflict",
            Self::ClarifyClaim => "clarify_claim",
        }
    }
}

/// Structured reason for a completion-verification finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionVerificationFindingKind {
    MissingProof,
    StaleProof,
    FailedProof,
    MismatchedProof,
    ClaimConflict,
    MissingChildProof,
    StaleChildProof,
    FailedChildProof,
}

impl CompletionVerificationFindingKind {
    /// Returns the stable wire text for the finding kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingProof => "missing_proof",
            Self::StaleProof => "stale_proof",
            Self::FailedProof => "failed_proof",
            Self::MismatchedProof => "mismatched_proof",
            Self::ClaimConflict => "claim_conflict",
            Self::MissingChildProof => "missing_child_proof",
            Self::StaleChildProof => "stale_child_proof",
            Self::FailedChildProof => "failed_child_proof",
        }
    }
}

/// Severity attached to a completion-verification finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionVerificationFindingSeverity {
    Blocking,
    Warning,
}

impl CompletionVerificationFindingSeverity {
    /// Returns the stable wire text for the finding severity.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::Warning => "warning",
        }
    }
}

/// Deterministic prompt requirement after claim inference and proof selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimConfirmationRequirement {
    SilentAllowed,
    ConfirmationRequired,
    ClarificationRequired,
}

/// Concrete claim that the runtime must prove before closeout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionClaim {
    pub claim_id: String,
    pub kind: CompletionClaimKind,
    pub scope: CompletionVerificationScope,
    pub source: CompletionClaimSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ClaimInferenceConfidence>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_signals: Vec<String>,
}

impl CompletionClaim {
    /// Validates the first-slice claim shape.
    pub fn validate(&self) -> Result<(), CompletionVerificationError> {
        if self.claim_id.trim().is_empty() {
            return Err(CompletionVerificationError::InvalidClaim(
                "claim_id must not be empty".to_string(),
            ));
        }
        if self.summary.trim().is_empty() {
            return Err(CompletionVerificationError::InvalidClaim(
                "summary must not be empty".to_string(),
            ));
        }
        match self.source {
            CompletionClaimSource::ExplicitMetadata => {
                if self.confidence.is_some() {
                    return Err(CompletionVerificationError::InvalidClaim(
                        "explicit_metadata claims must not carry inference confidence".to_string(),
                    ));
                }
            }
            CompletionClaimSource::RuntimeInference => {
                if self.confidence.is_none() {
                    return Err(CompletionVerificationError::InvalidClaim(
                        "runtime_inference claims require confidence".to_string(),
                    ));
                }
            }
            CompletionClaimSource::OperatorConfirmed | CompletionClaimSource::OperatorOverride => {}
        }
        Ok(())
    }
}

/// One deterministic proof rule candidate for a claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCommandRule {
    pub claim_kind: CompletionClaimKind,
    pub command_ref: String,
    pub command_line: String,
    pub selection_reason: String,
    pub breadth_rank: u8,
    #[serde(default)]
    pub fully_covers_claim: bool,
    #[serde(default)]
    pub documentation_relevant: bool,
}

impl ProofCommandRule {
    /// Validates the proof-command rule.
    pub fn validate(&self) -> Result<(), CompletionVerificationError> {
        if self.command_ref.trim().is_empty() {
            return Err(CompletionVerificationError::InvalidProofRule(
                "command_ref must not be empty".to_string(),
            ));
        }
        if self.command_line.trim().is_empty() {
            return Err(CompletionVerificationError::InvalidProofRule(
                "command_line must not be empty".to_string(),
            ));
        }
        if self.selection_reason.trim().is_empty() {
            return Err(CompletionVerificationError::InvalidProofRule(
                "selection_reason must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Selected proof command for the active claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCommandSelection {
    pub claim_id: String,
    pub command_ref: String,
    pub command_line: String,
    pub selection_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_note: Option<String>,
    #[serde(default)]
    pub documentation_relevant: bool,
}

impl ProofCommandSelection {
    /// Returns true when the proof fully covers the active claim.
    pub fn fully_covers_claim(&self) -> bool {
        self.coverage_note.is_none()
    }
}

/// One normalized file entry recorded inside a workspace fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFingerprintEntry {
    pub path: String,
    pub digest: String,
    pub tracked: bool,
}

/// Snapshot of meaningful workspace content at proof time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceContentFingerprint {
    pub fingerprint_id: String,
    pub captured_at: u64,
    pub content_digest: String,
    #[serde(default)]
    pub tracked_path_count: usize,
    #[serde(default)]
    pub untracked_path_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub included_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<WorkspaceFingerprintEntry>,
}

/// Bounded diff result for two workspace fingerprints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFingerprintDiff {
    pub changed_paths: Vec<String>,
    pub truncated: bool,
}

/// Structured completion-verification finding rendered to operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionVerificationFinding {
    pub kind: CompletionVerificationFindingKind,
    pub severity: CompletionVerificationFindingSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_paths: Vec<String>,
    pub required_action: CompletionRequiredAction,
}

/// Child summary surfaced by parent-scope verification projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildVerificationSummary {
    pub scope: CompletionVerificationScope,
    pub ready_children: usize,
    pub blocked_children: usize,
    pub failed_children: usize,
    pub stale_children: usize,
    pub missing_proof_children: usize,
    pub deferred_children: usize,
    pub skipped_children: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CompletionVerificationFinding>,
}

/// One child outcome consumed while aggregating stage- or run-scope verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildVerificationInput {
    pub task_id: String,
    pub required: bool,
    pub deferred_reason: Option<String>,
    pub skipped_reason: Option<String>,
    pub projection: Option<CompletionVerificationProjection>,
}

/// Additive operator-facing completion-verification projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionVerificationProjection {
    pub completion_verification_state: CompletionVerificationState,
    pub scope: CompletionVerificationScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim: Option<CompletionClaim>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completion_blocked_claims: Vec<CompletionClaimKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completion_evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completion_verification_findings: Vec<CompletionVerificationFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_summary: Option<ChildVerificationSummary>,
}

impl CompletionVerificationProjection {
    /// Validates the additive completion-verification projection.
    pub fn validate(&self) -> Result<(), CompletionVerificationError> {
        if matches!(
            self.completion_verification_state,
            CompletionVerificationState::Blocked
                | CompletionVerificationState::ProofRequired
                | CompletionVerificationState::Failed
        ) && self.completion_verification_findings.is_empty()
        {
            return Err(CompletionVerificationError::InvalidProjection(
                "blocking states require at least one finding".to_string(),
            ));
        }
        if self.completion_verification_state == CompletionVerificationState::Ready
            && self
                .completion_verification_findings
                .iter()
                .any(|finding| finding.severity == CompletionVerificationFindingSeverity::Blocking)
        {
            return Err(CompletionVerificationError::InvalidProjection(
                "ready projections must not carry blocking findings".to_string(),
            ));
        }
        Ok(())
    }
}

/// Policy inputs for deciding whether claim confirmation is required.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaimConfirmationContext {
    pub multiple_plausible_claims: bool,
    pub proof_only_partially_covers_claim: bool,
    pub risky_surface: bool,
    pub conflicting_claim_signals: bool,
    pub policy_allows_medium_without_confirmation: bool,
}

/// Error surfaced by completion-verification helpers.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompletionVerificationError {
    #[error("completion claim is invalid: {0}")]
    InvalidClaim(String),
    #[error("proof command rule is invalid: {0}")]
    InvalidProofRule(String),
    #[error("completion projection is invalid: {0}")]
    InvalidProjection(String),
    #[error("workspace root `{0}` is not a supported git workspace")]
    UnsupportedWorkspace(String),
    #[error("git command failed while capturing workspace fingerprint: {0}")]
    GitCommandFailed(String),
    #[error("failed to read fingerprint input `{path}`: {message}")]
    FingerprintReadFailed { path: String, message: String },
    #[error("completion claim could not be inferred from runtime context")]
    ClaimInferenceFailed,
}

/// Infers a first-slice completion claim from runtime-visible summaries.
pub fn infer_completion_claim(
    claim_id: impl Into<String>,
    goal: &str,
    expected_outcome: Option<&str>,
    changed_files: &[String],
    validation_command: Option<&str>,
) -> Result<CompletionClaim, CompletionVerificationError> {
    let expected = expected_outcome.unwrap_or_default().trim();
    let combined = format!("{goal}\n{expected}").to_ascii_lowercase();
    let summary = if expected.is_empty() { goal.trim() } else { expected };
    let inferred_kind =
        infer_claim_kind_from_runtime_context(&combined, changed_files, validation_command);

    let kind = inferred_kind.ok_or(CompletionVerificationError::ClaimInferenceFailed)?;
    let mut supporting_signals = Vec::new();
    if !expected.is_empty() {
        supporting_signals.push(format!("expected_outcome:{expected}"));
    }
    if !goal.trim().is_empty() {
        supporting_signals.push(format!("goal:{goal}"));
    }
    if !changed_files.is_empty() {
        supporting_signals.push(format!("changed_files:{}", changed_files.join(",")));
    }
    if let Some(validation_command) = validation_command.filter(|value| !value.trim().is_empty()) {
        supporting_signals.push(format!("validation_command:{validation_command}"));
    }

    Ok(CompletionClaim {
        claim_id: claim_id.into(),
        kind,
        scope: CompletionVerificationScope::Task,
        source: CompletionClaimSource::RuntimeInference,
        confidence: Some(
            if matches!(kind, CompletionClaimKind::TestsPass | CompletionClaimKind::MigrationValid)
            {
                ClaimInferenceConfidence::High
            } else {
                ClaimInferenceConfidence::Medium
            },
        ),
        summary: summary.to_string(),
        supporting_signals,
    })
}

fn infer_claim_kind_from_runtime_context(
    combined_text: &str,
    changed_files: &[String],
    validation_command: Option<&str>,
) -> Option<CompletionClaimKind> {
    let validation_command = validation_command.unwrap_or_default().to_ascii_lowercase();
    let changed_file_text =
        changed_files.iter().map(|path| path.to_ascii_lowercase()).collect::<Vec<_>>();
    let changed_files_include_tests =
        changed_file_text.iter().any(|path| path.contains(TEST_KEYWORD));
    let text_mentions_failing_tests =
        combined_text.contains(TEST_KEYWORD) || combined_text.contains("failing");
    let text_mentions_verification =
        VERIFICATION_KEYWORDS.iter().any(|keyword| combined_text.contains(keyword));

    if validation_command.contains(TEST_KEYWORD)
        && (changed_files_include_tests
            || text_mentions_failing_tests
            || text_mentions_verification)
    {
        return Some(CompletionClaimKind::TestsPass);
    }

    if validation_command.contains(BUILD_KEYWORD)
        && (combined_text.contains(BUILD_KEYWORD)
            || changed_file_text.iter().any(|path| {
                path.ends_with("cargo.toml")
                    || path.ends_with("cargo.lock")
                    || path.ends_with("package.json")
                    || path.ends_with("build.rs")
            }))
    {
        return Some(CompletionClaimKind::BuildClean);
    }

    infer_claim_kind_from_text(combined_text)
}

/// Infers only the claim kind from a free-form text fragment.
pub fn infer_claim_kind_from_text(text: &str) -> Option<CompletionClaimKind> {
    let lower = text.to_ascii_lowercase();
    if lower.contains(TEST_KEYWORD) {
        Some(CompletionClaimKind::TestsPass)
    } else if BUG_KEYWORDS.iter().any(|keyword| lower.contains(keyword)) {
        Some(CompletionClaimKind::BugFixed)
    } else if lower.contains(BUILD_KEYWORD) {
        Some(CompletionClaimKind::BuildClean)
    } else if lower.contains(MIGRATION_KEYWORD) {
        Some(CompletionClaimKind::MigrationValid)
    } else {
        None
    }
}

/// Builds first-slice proof rules from the workspace validation command.
pub fn proof_rules_for_validation_command(
    command_ref: impl Into<String>,
    command_line: impl Into<String>,
) -> Vec<ProofCommandRule> {
    let command_ref = command_ref.into();
    let command_line = command_line.into();
    let lower = command_line.to_ascii_lowercase();
    let mut rules = Vec::new();

    if lower.contains(TEST_KEYWORD) {
        rules.push(ProofCommandRule {
            claim_kind: CompletionClaimKind::TestsPass,
            command_ref: command_ref.clone(),
            command_line: command_line.clone(),
            selection_reason:
                "workspace validation command is the narrowest available falsifier for the claimed test outcome"
                    .to_string(),
            breadth_rank: 0,
            fully_covers_claim: true,
            documentation_relevant: false,
        });
    }

    if lower.contains(BUILD_KEYWORD) {
        rules.push(ProofCommandRule {
            claim_kind: CompletionClaimKind::BuildClean,
            command_ref,
            command_line,
            selection_reason:
                "workspace validation command is the narrowest available falsifier for the claimed build outcome"
                    .to_string(),
            breadth_rank: 0,
            fully_covers_claim: true,
            documentation_relevant: false,
        });
    }

    rules
}

/// Selects the narrowest available proof command for the provided claim.
pub fn select_proof_command(
    claim: &CompletionClaim,
    rules: &[ProofCommandRule],
) -> Result<Option<ProofCommandSelection>, CompletionVerificationError> {
    claim.validate()?;

    let mut matching = Vec::new();
    for rule in rules {
        rule.validate()?;
        if rule.claim_kind == claim.kind {
            matching.push(rule);
        }
    }

    matching.sort_by(|left, right| {
        left.breadth_rank
            .cmp(&right.breadth_rank)
            .then_with(|| left.command_ref.cmp(&right.command_ref))
    });

    Ok(matching.first().map(|rule| ProofCommandSelection {
        claim_id: claim.claim_id.clone(),
        command_ref: rule.command_ref.clone(),
        command_line: rule.command_line.clone(),
        selection_reason: rule.selection_reason.clone(),
        coverage_note: if rule.fully_covers_claim {
            None
        } else {
            Some("selected proof validates only part of the active claim".to_string())
        },
        documentation_relevant: rule.documentation_relevant,
    }))
}

/// Determines whether the runtime must ask for claim confirmation.
pub fn claim_confirmation_requirement(
    confidence: Option<ClaimInferenceConfidence>,
    context: &ClaimConfirmationContext,
) -> ClaimConfirmationRequirement {
    if context.conflicting_claim_signals || context.multiple_plausible_claims {
        return ClaimConfirmationRequirement::ClarificationRequired;
    }
    if context.proof_only_partially_covers_claim || context.risky_surface {
        return ClaimConfirmationRequirement::ConfirmationRequired;
    }

    match confidence {
        Some(ClaimInferenceConfidence::High) => ClaimConfirmationRequirement::SilentAllowed,
        Some(ClaimInferenceConfidence::Medium)
            if context.policy_allows_medium_without_confirmation =>
        {
            ClaimConfirmationRequirement::SilentAllowed
        }
        Some(ClaimInferenceConfidence::Medium) | Some(ClaimInferenceConfidence::Low) => {
            ClaimConfirmationRequirement::ConfirmationRequired
        }
        None => ClaimConfirmationRequirement::ClarificationRequired,
    }
}

/// Captures the current workspace-content fingerprint using tracked files plus
/// non-ignored untracked files, excluding Boundline runtime artifacts and
/// other configured volatile paths.
pub fn capture_workspace_fingerprint(
    workspace_root: &Path,
    documentation_relevant: bool,
) -> Result<WorkspaceContentFingerprint, CompletionVerificationError> {
    let (tracked_paths, untracked_paths) = match capture_git_workspace_paths(workspace_root) {
        Ok(paths) => paths,
        Err(_) => (Vec::new(), filesystem_path_list(workspace_root)?),
    };

    let mut entries = Vec::new();
    let mut tracked_count = 0usize;
    let mut untracked_count = 0usize;

    for path in tracked_paths {
        if should_include_fingerprint_path(&path, documentation_relevant) {
            tracked_count += 1;
            entries.push(build_fingerprint_entry(workspace_root, &path, true)?);
        }
    }
    for path in untracked_paths {
        if should_include_fingerprint_path(&path, documentation_relevant) {
            untracked_count += 1;
            entries.push(build_fingerprint_entry(workspace_root, &path, false)?);
        }
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));

    let mut hasher = DefaultHasher::new();
    for entry in &entries {
        entry.path.hash(&mut hasher);
        entry.digest.hash(&mut hasher);
        entry.tracked.hash(&mut hasher);
    }
    let content_digest = format!("{:016x}", hasher.finish());

    Ok(WorkspaceContentFingerprint {
        fingerprint_id: format!("fp-{content_digest}"),
        captured_at: current_timestamp_millis(),
        content_digest,
        tracked_path_count: tracked_count,
        untracked_path_count: untracked_count,
        included_roots: vec!["tracked".to_string(), "untracked_non_ignored".to_string()],
        excluded_roots: vec![
            GIT_DIRECTORY_NAME.to_string(),
            BOUNDLINE_TRACES_PATH.to_string(),
            BOUNDLINE_ARTIFACTS_PATH.to_string(),
            BOUNDLINE_CACHE_PATH.to_string(),
        ],
        entries,
    })
}

fn capture_git_workspace_paths(
    workspace_root: &Path,
) -> Result<(Vec<String>, Vec<String>), CompletionVerificationError> {
    let tracked_paths = git_path_list(workspace_root, false)?;
    let untracked_paths = git_path_list(workspace_root, true)?;
    Ok((tracked_paths, untracked_paths))
}

/// Compares a passing proof fingerprint with the current fingerprint and
/// returns the bounded changed-path set used for stale findings.
pub fn compare_workspace_fingerprints(
    previous: &WorkspaceContentFingerprint,
    current: &WorkspaceContentFingerprint,
) -> WorkspaceFingerprintDiff {
    let previous_map = fingerprint_entry_map(previous);
    let current_map = fingerprint_entry_map(current);
    let mut changed_paths = BTreeSet::new();

    for (path, digest) in &previous_map {
        match current_map.get(path) {
            Some(current_digest) if current_digest == digest => {}
            _ => {
                changed_paths.insert(path.clone());
            }
        }
    }
    for path in current_map.keys() {
        if !previous_map.contains_key(path) {
            changed_paths.insert(path.clone());
        }
    }

    let truncated = changed_paths.len() > FINGERPRINT_PATH_SAMPLE_LIMIT;
    let sampled = changed_paths.into_iter().take(FINGERPRINT_PATH_SAMPLE_LIMIT).collect();
    WorkspaceFingerprintDiff { changed_paths: sampled, truncated }
}

/// Projects a previously passing proof into a stale proof-required state after
/// workspace content changes invalidate the recorded passing fingerprint.
pub fn stale_proof_projection(
    claim: &CompletionClaim,
    completion_evidence_refs: &[String],
    diff: &WorkspaceFingerprintDiff,
) -> CompletionVerificationProjection {
    let mut changed_paths = diff.changed_paths.clone();
    let mut message =
        "The previously passing proof is stale because workspace content changed after proof execution."
            .to_string();
    if diff.truncated {
        changed_paths.push(FINGERPRINT_PATH_TRUNCATION_MARKER.to_string());
        message.push_str(" Showing a capped list of changed paths.");
    }

    CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::ProofRequired,
        scope: CompletionVerificationScope::Task,
        claim: Some(claim.clone()),
        completion_blocked_claims: vec![claim.kind],
        completion_evidence_refs: completion_evidence_refs.to_vec(),
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::StaleProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message,
            proof_ref: None,
            task_id: None,
            changed_paths,
            required_action: CompletionRequiredAction::RerunProof,
        }],
        child_summary: None,
    }
}

/// Aggregates required child task projections into a parent-scope blocked or
/// ready projection without inventing replacement proof ownership.
pub fn aggregate_child_verification(
    scope: CompletionVerificationScope,
    children: &[ChildVerificationInput],
) -> CompletionVerificationProjection {
    let mut summary = ChildVerificationSummary {
        scope,
        ready_children: 0,
        blocked_children: 0,
        failed_children: 0,
        stale_children: 0,
        missing_proof_children: 0,
        deferred_children: 0,
        skipped_children: 0,
        findings: Vec::new(),
    };
    let mut blocked_claims = Vec::new();
    let mut evidence_refs = Vec::new();

    for child in children {
        if child.skipped_reason.is_some() || !child.required {
            summary.skipped_children += 1;
            continue;
        }
        if child.deferred_reason.is_some() {
            summary.deferred_children += 1;
            continue;
        }
        let Some(projection) = child.projection.as_ref() else {
            summary.blocked_children += 1;
            summary.missing_proof_children += 1;
            summary.findings.push(CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::MissingChildProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message: format!("required child `{}` is missing proof", child.task_id),
                proof_ref: None,
                task_id: Some(child.task_id.clone()),
                changed_paths: Vec::new(),
                required_action: CompletionRequiredAction::RunProof,
            });
            continue;
        };

        evidence_refs.extend(projection.completion_evidence_refs.clone());
        match projection.completion_verification_state {
            CompletionVerificationState::Ready => {
                summary.ready_children += 1;
            }
            CompletionVerificationState::Failed => {
                summary.blocked_children += 1;
                summary.failed_children += 1;
                blocked_claims.extend(projection.completion_blocked_claims.clone());
                summary.findings.extend(remap_child_findings(
                    &child.task_id,
                    &projection.completion_verification_findings,
                    CompletionVerificationFindingKind::FailedChildProof,
                    CompletionRequiredAction::RerunProof,
                ));
            }
            CompletionVerificationState::ProofRequired | CompletionVerificationState::Blocked => {
                summary.blocked_children += 1;
                blocked_claims.extend(projection.completion_blocked_claims.clone());
                let contains_stale = projection
                    .completion_verification_findings
                    .iter()
                    .any(|finding| finding.kind == CompletionVerificationFindingKind::StaleProof);
                if contains_stale {
                    summary.stale_children += 1;
                    summary.findings.extend(remap_child_findings(
                        &child.task_id,
                        &projection.completion_verification_findings,
                        CompletionVerificationFindingKind::StaleChildProof,
                        CompletionRequiredAction::RerunProof,
                    ));
                } else {
                    summary.missing_proof_children += 1;
                    summary.findings.extend(remap_child_findings(
                        &child.task_id,
                        &projection.completion_verification_findings,
                        CompletionVerificationFindingKind::MissingChildProof,
                        CompletionRequiredAction::RunProof,
                    ));
                }
            }
        }
    }

    let state = if summary.blocked_children == 0 && summary.failed_children == 0 {
        CompletionVerificationState::Ready
    } else {
        CompletionVerificationState::Blocked
    };
    let findings = summary.findings.clone();
    CompletionVerificationProjection {
        completion_verification_state: state,
        scope,
        claim: None,
        completion_blocked_claims: blocked_claims,
        completion_evidence_refs: evidence_refs,
        completion_verification_findings: findings,
        child_summary: Some(summary),
    }
}

fn remap_child_findings(
    task_id: &str,
    findings: &[CompletionVerificationFinding],
    fallback_kind: CompletionVerificationFindingKind,
    fallback_action: CompletionRequiredAction,
) -> Vec<CompletionVerificationFinding> {
    if findings.is_empty() {
        return vec![CompletionVerificationFinding {
            kind: fallback_kind,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: format!("required child `{task_id}` is not verification-ready"),
            proof_ref: None,
            task_id: Some(task_id.to_string()),
            changed_paths: Vec::new(),
            required_action: fallback_action,
        }];
    }

    findings
        .iter()
        .map(|finding| CompletionVerificationFinding {
            kind: match finding.kind {
                CompletionVerificationFindingKind::StaleProof => {
                    CompletionVerificationFindingKind::StaleChildProof
                }
                CompletionVerificationFindingKind::FailedProof => {
                    CompletionVerificationFindingKind::FailedChildProof
                }
                _ => fallback_kind,
            },
            severity: finding.severity,
            message: finding.message.clone(),
            proof_ref: finding.proof_ref.clone(),
            task_id: Some(task_id.to_string()),
            changed_paths: finding.changed_paths.clone(),
            required_action: finding.required_action,
        })
        .collect()
}

fn fingerprint_entry_map(fingerprint: &WorkspaceContentFingerprint) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    for entry in &fingerprint.entries {
        entries.insert(entry.path.clone(), entry.digest.clone());
    }
    entries
}

fn git_path_list(
    workspace_root: &Path,
    untracked_only: bool,
) -> Result<Vec<String>, CompletionVerificationError> {
    if !workspace_root.join(GIT_DIRECTORY_NAME).exists() {
        return Err(CompletionVerificationError::UnsupportedWorkspace(
            workspace_root.display().to_string(),
        ));
    }

    let mut command = Command::new("git");
    command.arg("-C").arg(workspace_root).arg("ls-files");
    if untracked_only {
        command.arg("--others").arg("--exclude-standard");
    }
    command.arg("-z");

    let output = command.output().map_err(|error| {
        CompletionVerificationError::GitCommandFailed(format!(
            "failed to execute git ls-files: {error}"
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CompletionVerificationError::GitCommandFailed(stderr));
    }

    let mut paths = Vec::new();
    for raw_path in output.stdout.split(|byte| *byte == 0) {
        if raw_path.is_empty() {
            continue;
        }
        paths.push(String::from_utf8_lossy(raw_path).to_string());
    }

    Ok(paths)
}

fn filesystem_path_list(workspace_root: &Path) -> Result<Vec<String>, CompletionVerificationError> {
    let mut paths = Vec::new();
    collect_filesystem_paths(workspace_root, workspace_root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_filesystem_paths(
    workspace_root: &Path,
    current: &Path,
    paths: &mut Vec<String>,
) -> Result<(), CompletionVerificationError> {
    for entry in fs::read_dir(current).map_err(|error| {
        CompletionVerificationError::FingerprintReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        }
    })? {
        let entry = entry.map_err(|error| CompletionVerificationError::FingerprintReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        let relative = path.strip_prefix(workspace_root).map_err(|error| {
            CompletionVerificationError::FingerprintReadFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            }
        })?;
        let normalized = normalize_rel_path(&relative.to_string_lossy());
        if path.is_dir() {
            if !normalized.is_empty() && path_is_excluded_from_fingerprint(&normalized) {
                continue;
            }
            collect_filesystem_paths(workspace_root, &path, paths)?;
        } else if path.is_file() && should_include_fingerprint_path(&normalized, true) {
            paths.push(normalized);
        }
    }
    Ok(())
}

fn path_is_excluded_from_fingerprint(path: &str) -> bool {
    path == GIT_DIRECTORY_NAME
        || path.starts_with(&format!("{GIT_DIRECTORY_NAME}/"))
        || path.starts_with(&format!("{BOUNDLINE_DIRECTORY_NAME}/"))
        || path_contains_volatile_directory(path)
}

fn build_fingerprint_entry(
    workspace_root: &Path,
    path: &str,
    tracked: bool,
) -> Result<WorkspaceFingerprintEntry, CompletionVerificationError> {
    let absolute = workspace_root.join(path);
    let bytes = fs::read(&absolute).map_err(|error| {
        CompletionVerificationError::FingerprintReadFailed {
            path: path.to_string(),
            message: error.to_string(),
        }
    })?;
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    bytes.hash(&mut hasher);
    let digest = format!("{:016x}", hasher.finish());
    Ok(WorkspaceFingerprintEntry { path: normalize_rel_path(path), digest, tracked })
}

fn should_include_fingerprint_path(path: &str, documentation_relevant: bool) -> bool {
    let normalized = normalize_rel_path(path);
    if normalized.is_empty() {
        return false;
    }
    if normalized == GIT_DIRECTORY_NAME || normalized.starts_with(&format!("{GIT_DIRECTORY_NAME}/"))
    {
        return false;
    }
    if normalized.starts_with(&format!("{BOUNDLINE_DIRECTORY_NAME}/")) {
        return false;
    }
    if path_contains_volatile_directory(&normalized) {
        return false;
    }

    if is_source_path(&normalized)
        || is_config_path(&normalized)
        || is_test_path(&normalized)
        || is_build_path(&normalized)
    {
        return true;
    }

    documentation_relevant && is_documentation_path(&normalized)
}

fn normalize_rel_path(path: &str) -> String {
    let mut normalized = String::new();
    let input = Path::new(path);
    for component in input.components() {
        if let Component::Normal(segment) = component
            && let Some(text) = segment.to_str()
        {
            if !normalized.is_empty() {
                normalized.push(PATH_SEPARATOR);
            }
            normalized.push_str(text);
        }
    }
    normalized
}

fn path_contains_volatile_directory(path: &str) -> bool {
    path.split(PATH_SEPARATOR).any(|segment| VOLATILE_DIRECTORY_NAMES.contains(&segment))
}

fn is_source_path(path: &str) -> bool {
    let path_ref = Path::new(path);
    path_has_component(path_ref, "src") || has_extension(path_ref, SOURCE_EXTENSIONS)
}

fn is_config_path(path: &str) -> bool {
    let path_ref = Path::new(path);
    has_extension(path_ref, CONFIG_EXTENSIONS)
        || file_name_is(path_ref, ".env")
        || path_ref.file_name().and_then(OsStr::to_str).is_some_and(|name| name.starts_with(".env"))
}

fn is_test_path(path: &str) -> bool {
    let path_ref = Path::new(path);
    path_has_component(path_ref, "tests") || has_extension(path_ref, TEST_EXTENSIONS)
}

fn is_build_path(path: &str) -> bool {
    let path_ref = Path::new(path);
    file_name_in(path_ref, BUILD_FILE_NAMES)
}

fn is_documentation_path(path: &str) -> bool {
    let path_ref = Path::new(path);
    path_has_component(path_ref, "docs")
        || path_has_component(path_ref, "specs")
        || path_has_component(path_ref, "roadmap")
        || has_extension(path_ref, DOCUMENTATION_EXTENSIONS)
        || file_name_in(path_ref, DOCUMENTATION_FILE_NAMES)
}

fn path_has_component(path: &Path, expected: &str) -> bool {
    path.components().any(|component| {
        if let Component::Normal(segment) = component {
            segment == OsStr::new(expected)
        } else {
            false
        }
    })
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extensions.contains(&extension))
}

fn file_name_is(path: &Path, file_name: &str) -> bool {
    path.file_name().and_then(OsStr::to_str) == Some(file_name)
}

fn file_name_in(path: &Path, file_names: &[&str]) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|file_name| file_names.contains(&file_name))
}
