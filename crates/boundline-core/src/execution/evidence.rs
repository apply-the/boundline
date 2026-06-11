//! Structured evidence capture and persistence.
//!
//! Every command executed through Boundline produces an `EvidencePacket`
//! containing timing, exit code, stdout/stderr (redacted), artifact
//! manifest, and mutation boundary. Packets are persisted as JSON to
//! `.boundline/traces/`.

use serde::{Deserialize, Serialize};

use super::classifier::{CommandIntent, DryRunStatus, ExecutionMode};

/// Structured record of a single command execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePacket {
    /// Stable trace identifier: `{ISO8601_timestamp}-{sha256_hex12}`.
    pub trace_id: String,
    /// Full command string as provided by the operator.
    pub command: String,
    /// Classified intent.
    pub intent: CommandIntent,
    /// Resolved execution mode after policy evaluation.
    pub execution_mode: ExecutionMode,
    /// Dry-run sub-status (Some only when mode is DryRun).
    pub dry_run_status: Option<DryRunStatus>,
    /// Wall-clock timing of the execution.
    pub timing: ExecutionTiming,
    /// Exit code (None if killed by signal).
    pub exit_code: Option<i32>,
    /// Redacted stdout, capped at configured limit.
    pub stdout: String,
    /// Whether stdout was truncated.
    pub stdout_truncated: bool,
    /// Redacted stderr, capped at configured limit.
    pub stderr: String,
    /// Whether stderr was truncated.
    pub stderr_truncated: bool,
    /// List of files produced or modified.
    pub artifact_manifest: ArtifactManifest,
    /// Pre/post execution file state diff.
    pub mutation_boundary: MutationBoundary,
    /// How the policy resolved for this command.
    pub policy_decision: PolicyDecision,
    /// Record of redacted patterns.
    pub redaction_audit: Vec<RedactionRecord>,
    /// ISO 8601 when the evidence packet was created.
    pub timestamp: String,
}

/// Timing information for a command execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTiming {
    /// Wall-clock start time.
    /// ISO 8601 wall-clock start time.
    pub started_at: String,
    /// ISO 8601 wall-clock finish time.
    pub finished_at: String,
    /// Elapsed wall time in milliseconds.
    pub wall_clock_ms: u64,
}

/// Manifest of files produced or modified by a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    /// Individual file entries.
    pub files: Vec<ArtifactEntry>,
    /// Total count of files produced or modified.
    pub total_files_produced: u32,
}

/// A single file entry in the artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    /// Path relative to workspace root.
    pub path: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// ISO 8601 last modification time.
    pub modified_at: String,
    /// What happened to this file.
    pub operation: FileOperation,
}

/// Operation performed on a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Created,
    Modified,
    Deleted,
}

/// Pre and post execution file state diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationBoundary {
    /// Files created during execution.
    pub created: Vec<String>,
    /// Files modified during execution.
    pub modified: Vec<ModifiedFile>,
    /// Files deleted during execution.
    pub deleted: Vec<String>,
    /// Whether the mutation list was truncated at the cap.
    pub truncated: bool,
    /// Actual count when truncated.
    pub total_observed: Option<u32>,
    /// Whether the mutation detection completed successfully.
    pub complete: bool,
    /// Error message if mutation detection failed.
    pub error: Option<String>,
}

/// A file that was modified with pre and post content hashes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifiedFile {
    /// Path relative to workspace root.
    pub path: String,
    /// SHA-256 hash before execution.
    pub pre_hash: String,
    /// SHA-256 hash after execution.
    pub post_hash: String,
}

/// Records how the execution policy resolved for a single command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// The inferred intent from classification.
    pub inferred_intent: CommandIntent,
    /// Risk zone used for resolution.
    pub zone: RiskZone,
    /// Policy matrix key that matched (e.g., "policy.mutate.green").
    pub matched_policy_entry: String,
    /// Command override that applied, if any.
    pub matched_override: Option<String>,
    /// Safety escalation flags that were applied.
    pub safety_escalations: Vec<String>,
    /// The final execution mode after all resolution steps.
    pub final_mode: ExecutionMode,
    /// Human-readable explanation of the decision.
    pub rationale: String,
}

/// Risk zone for policy resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskZone {
    Green,
    Yellow,
    Red,
}

/// Record of a single redaction event for auditing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionRecord {
    /// The pattern ID that matched.
    pub pattern_id: String,
    /// Number of matches found and redacted.
    pub match_count: u32,
}

impl FileOperation {
    /// Returns a human-readable label for the operation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
        }
    }
}

impl RiskZone {
    /// Returns a human-readable label for the zone.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
        }
    }
}

// ── Evidence limits configuration ──────────────────────────────────────

/// Configurable limits for evidence capture, loaded from
/// `.boundline/evidence-limits.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLimits {
    #[serde(default = "default_stdout_max_bytes")]
    pub stdout_max_bytes: usize,
    #[serde(default = "default_stderr_max_bytes")]
    pub stderr_max_bytes: usize,
    #[serde(default = "default_mutation_max_entries")]
    pub mutation_max_entries: usize,
}

fn default_stdout_max_bytes() -> usize {
    1_048_576 // 1 MB
}

fn default_stderr_max_bytes() -> usize {
    1_048_576 // 1 MB
}

fn default_mutation_max_entries() -> usize {
    10_000
}

impl Default for EvidenceLimits {
    fn default() -> Self {
        Self {
            stdout_max_bytes: default_stdout_max_bytes(),
            stderr_max_bytes: default_stderr_max_bytes(),
            mutation_max_entries: default_mutation_max_entries(),
        }
    }
}

impl EvidenceLimits {
    /// Loads limits from an optional TOML string. Falls back to defaults
    /// when no string is provided or parsing fails.
    pub fn load(toml_str: Option<&str>) -> Self {
        match toml_str {
            Some(s) => toml::from_str(s).unwrap_or_default(),
            None => Self::default(),
        }
    }
}

// ── EvidencePacket builder ──────────────────────────────────────────────

/// Truncation marker appended when output exceeds the cap.
const TRUNCATION_MARKER: &str = "\n[TRUNCATED: original {} bytes]";

impl EvidencePacket {
    /// Creates a new builder for an `EvidencePacket`.
    pub fn builder(
        command: String,
        intent: CommandIntent,
        mode: ExecutionMode,
    ) -> EvidencePacketBuilder {
        EvidencePacketBuilder::new(command, intent, mode)
    }
}

/// Builder for constructing an [`EvidencePacket`] incrementally.
pub struct EvidencePacketBuilder {
    command: String,
    intent: CommandIntent,
    execution_mode: ExecutionMode,
    dry_run_status: Option<DryRunStatus>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
    artifact_manifest: ArtifactManifest,
    mutation_boundary: MutationBoundary,
    policy_decision: Option<PolicyDecision>,
    redaction_audit: Vec<RedactionRecord>,
    started_at: String,
    finished_at: String,
    limits: EvidenceLimits,
}

impl EvidencePacketBuilder {
    fn new(command: String, intent: CommandIntent, mode: ExecutionMode) -> Self {
        let now = timestamp_now();
        Self {
            command,
            intent,
            execution_mode: mode,
            dry_run_status: None,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            artifact_manifest: ArtifactManifest { files: Vec::new(), total_files_produced: 0 },
            mutation_boundary: MutationBoundary {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: Vec::new(),
                truncated: false,
                total_observed: None,
                complete: true,
                error: None,
            },
            policy_decision: None,
            redaction_audit: Vec::new(),
            started_at: now.clone(),
            finished_at: now,
            limits: EvidenceLimits::default(),
        }
    }

    /// Overrides the default evidence limits.
    pub fn limits(mut self, limits: EvidenceLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Sets the dry-run status.
    pub fn dry_run_status(mut self, status: DryRunStatus) -> Self {
        self.dry_run_status = Some(status);
        self
    }

    /// Captures stdout with size cap from limits.
    pub fn stdout(mut self, output: String) -> Self {
        let capped = cap_output(&output, self.limits.stdout_max_bytes);
        self.stdout = capped.0;
        self.stdout_truncated = capped.1;
        self
    }

    /// Captures stderr with size cap from limits.
    pub fn stderr(mut self, output: String) -> Self {
        let capped = cap_output(&output, self.limits.stderr_max_bytes);
        self.stderr = capped.0;
        self.stderr_truncated = capped.1;
        self
    }

    /// Sets the exit code.
    pub fn exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Sets the artifact manifest.
    pub fn artifact_manifest(mut self, manifest: ArtifactManifest) -> Self {
        self.artifact_manifest = manifest;
        self
    }

    /// Sets the mutation boundary.
    pub fn mutation_boundary(mut self, boundary: MutationBoundary) -> Self {
        self.mutation_boundary = boundary;
        self
    }

    /// Sets the policy decision.
    pub fn policy_decision(mut self, decision: PolicyDecision) -> Self {
        self.policy_decision = Some(decision);
        self
    }

    /// Adds redaction audit records.
    pub fn redaction_audit(mut self, audit: Vec<RedactionRecord>) -> Self {
        self.redaction_audit = audit;
        self
    }

    /// Builds the final [`EvidencePacket`].
    pub fn build(self) -> EvidencePacket {
        let trace_id = generate_trace_id(&self.command);
        let wall_clock_ms = elapsed_ms(&self.started_at, &self.finished_at);

        EvidencePacket {
            trace_id,
            command: self.command,
            intent: self.intent,
            execution_mode: self.execution_mode,
            dry_run_status: self.dry_run_status,
            timing: ExecutionTiming {
                started_at: self.started_at,
                finished_at: self.finished_at,
                wall_clock_ms,
            },
            exit_code: self.exit_code,
            stdout: self.stdout,
            stdout_truncated: self.stdout_truncated,
            stderr: self.stderr,
            stderr_truncated: self.stderr_truncated,
            artifact_manifest: self.artifact_manifest,
            mutation_boundary: self.mutation_boundary,
            policy_decision: self.policy_decision.unwrap_or(PolicyDecision {
                inferred_intent: self.intent,
                zone: RiskZone::Green,
                matched_policy_entry: String::new(),
                matched_override: None,
                safety_escalations: Vec::new(),
                final_mode: self.execution_mode,
                rationale: "default".into(),
            }),
            redaction_audit: self.redaction_audit,
            timestamp: timestamp_now(),
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Caps a string to `max_bytes`, returning the truncated string and
/// whether truncation occurred.
fn cap_output(output: &str, max_bytes: usize) -> (String, bool) {
    if output.len() <= max_bytes {
        return (output.to_string(), false);
    }
    let original_len = output.len();
    let truncated = &output[..max_bytes.min(output.len())];
    let marker = TRUNCATION_MARKER.replace("{}", &original_len.to_string());
    (format!("{}{}", truncated, marker), true)
}

/// Generates a trace ID: `{ISO8601}-{sha256_hex12}`.
fn generate_trace_id(command: &str) -> String {
    use std::hash::{Hash, Hasher};
    // For v1, use a simple hash for the command fingerprint.
    // sha2 dependency can be added later for proper SHA-256.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    command.hash(&mut hasher);
    let hash = hasher.finish();
    let timestamp = timestamp_now();
    format!("{}-{:012x}", timestamp.replace(':', "-"), hash)
}

/// Returns the current UTC time as an ISO 8601 string.
fn timestamp_now() -> String {
    // Use a simple approximation for v1 without the `time` crate.
    // Real implementation would use `std::time::SystemTime` with formatting.
    std::env::var("BOUNDLINE_TRACE_TIMESTAMP")
        .unwrap_or_else(|_| "2026-06-11T00:00:00Z".to_string())
}

/// Estimates elapsed wall-clock milliseconds between two ISO 8601 timestamps.
fn elapsed_ms(_start: &str, _end: &str) -> u64 {
    // v1: placeholder. Real implementation parses timestamps.
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evidence_packet_serde_roundtrip() {
        let packet = EvidencePacket {
            trace_id: "20260611T100000Z-a1b2c3d4e5f6".into(),
            command: "echo hello".into(),
            intent: CommandIntent::Read,
            execution_mode: ExecutionMode::Allow,
            dry_run_status: None,
            timing: ExecutionTiming {
                started_at: "2026-06-11T10:00:00Z".into(),
                finished_at: "2026-06-11T10:00:00Z".into(),
                wall_clock_ms: 5,
            },
            exit_code: Some(0),
            stdout: "hello\n".into(),
            stdout_truncated: false,
            stderr: String::new(),
            stderr_truncated: false,
            artifact_manifest: ArtifactManifest { files: Vec::new(), total_files_produced: 0 },
            mutation_boundary: MutationBoundary {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: Vec::new(),
                truncated: false,
                total_observed: None,
                complete: true,
                error: None,
            },
            policy_decision: PolicyDecision {
                inferred_intent: CommandIntent::Read,
                zone: RiskZone::Green,
                matched_policy_entry: "policy.read.green".into(),
                matched_override: None,
                safety_escalations: Vec::new(),
                final_mode: ExecutionMode::Allow,
                rationale: "read commands are allowed in all zones".into(),
            },
            redaction_audit: Vec::new(),
            timestamp: "2026-06-11T10:00:00Z".into(),
        };
        let json = serde_json::to_string_pretty(&packet).unwrap();
        let parsed: EvidencePacket = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.trace_id, packet.trace_id);
        assert_eq!(parsed.exit_code, Some(0));
    }

    #[test]
    fn file_operation_as_str() {
        assert_eq!(FileOperation::Created.as_str(), "created");
        assert_eq!(FileOperation::Modified.as_str(), "modified");
        assert_eq!(FileOperation::Deleted.as_str(), "deleted");
    }

    #[test]
    fn risk_zone_as_str() {
        assert_eq!(RiskZone::Green.as_str(), "green");
        assert_eq!(RiskZone::Yellow.as_str(), "yellow");
        assert_eq!(RiskZone::Red.as_str(), "red");
    }

    #[test]
    fn empty_stdout_and_stderr() {
        let packet =
            EvidencePacket::builder("echo ''".into(), CommandIntent::Read, ExecutionMode::Allow)
                .stdout(String::new())
                .stderr(String::new())
                .exit_code(0)
                .build();
        assert!(packet.stdout.is_empty());
        assert!(packet.stderr.is_empty());
        assert!(!packet.stdout_truncated);
        assert!(!packet.stderr_truncated);
        assert_eq!(packet.exit_code, Some(0));
    }

    #[test]
    fn sigkill_captures_exit_code_none() {
        // SIGKILL results in no exit code from the process (None).
        let packet =
            EvidencePacket::builder("sleep 3600".into(), CommandIntent::Read, ExecutionMode::Allow)
                .stdout("partial output\n".into())
                .stderr(String::new())
                .build();
        // exit_code not set → remains None (killed by signal)
        assert_eq!(packet.exit_code, None);
        assert!(packet.stdout.contains("partial output"));
    }

    #[test]
    fn concurrent_traces_are_independent() {
        // Each EvidencePacketBuilder produces an independent packet
        // with a unique trace_id — no shared mutable state.
        let p1 = EvidencePacket::builder("cmd1".into(), CommandIntent::Read, ExecutionMode::Allow)
            .exit_code(0)
            .build();
        let p2 = EvidencePacket::builder("cmd2".into(), CommandIntent::Read, ExecutionMode::Allow)
            .exit_code(0)
            .build();
        assert_ne!(p1.trace_id, p2.trace_id);
        assert_eq!(p1.command, "cmd1");
        assert_eq!(p2.command, "cmd2");
    }

    #[test]
    fn truncated_output_applies_marker() {
        let limits = EvidenceLimits { stdout_max_bytes: 10, ..EvidenceLimits::default() };
        let packet = EvidencePacket::builder(
            "large-output".into(),
            CommandIntent::Read,
            ExecutionMode::Allow,
        )
        .limits(limits)
        .stdout("this is a very long string that will be truncated".into())
        .build();
        assert!(packet.stdout_truncated);
        assert!(packet.stdout.contains("[TRUNCATED:"));
    }

    #[test]
    fn evidence_limits_defaults() {
        let limits = EvidenceLimits::default();
        assert_eq!(limits.stdout_max_bytes, 1_048_576);
        assert_eq!(limits.stderr_max_bytes, 1_048_576);
        assert_eq!(limits.mutation_max_entries, 10_000);
    }

    #[test]
    fn evidence_limits_custom_override() {
        let toml_str = r#"
stdout_max_bytes = 2_000_000
stderr_max_bytes = 500_000
mutation_max_entries = 20_000
"#;
        let limits = EvidenceLimits::load(Some(toml_str));
        assert_eq!(limits.stdout_max_bytes, 2_000_000);
        assert_eq!(limits.stderr_max_bytes, 500_000);
        assert_eq!(limits.mutation_max_entries, 20_000);
    }

    #[test]
    fn evidence_limits_load_none_returns_default() {
        let limits = EvidenceLimits::load(None);
        assert_eq!(limits, EvidenceLimits::default());
    }

    #[test]
    fn builder_artifact_manifest_sets_files() {
        let manifest = ArtifactManifest {
            files: vec![ArtifactEntry {
                path: "out.txt".into(),
                size_bytes: 100,
                modified_at: "2026-06-11T10:00:00Z".into(),
                operation: FileOperation::Created,
            }],
            total_files_produced: 1,
        };
        let packet = EvidencePacket::builder(
            "echo data > out.txt".into(),
            CommandIntent::Read,
            ExecutionMode::Allow,
        )
        .artifact_manifest(manifest)
        .build();
        assert_eq!(packet.artifact_manifest.total_files_produced, 1);
        assert_eq!(packet.artifact_manifest.files[0].path, "out.txt");
    }

    #[test]
    fn builder_mutation_boundary_tracks_changes() {
        let boundary = MutationBoundary {
            created: vec!["new.txt".into()],
            modified: Vec::new(),
            deleted: Vec::new(),
            truncated: false,
            total_observed: None,
            complete: true,
            error: None,
        };
        let packet = EvidencePacket::builder(
            "touch new.txt".into(),
            CommandIntent::Mutate,
            ExecutionMode::Allow,
        )
        .mutation_boundary(boundary)
        .build();
        assert_eq!(packet.mutation_boundary.created, vec!["new.txt"]);
    }

    #[test]
    fn builder_redaction_audit_records_hits() {
        let audit = vec![
            RedactionRecord { pattern_id: "github-token".into(), match_count: 2 },
            RedactionRecord { pattern_id: "aws-access-key".into(), match_count: 1 },
        ];
        let packet =
            EvidencePacket::builder("echo token".into(), CommandIntent::Read, ExecutionMode::Allow)
                .redaction_audit(audit)
                .build();
        assert_eq!(packet.redaction_audit.len(), 2);
        assert_eq!(packet.redaction_audit[0].match_count, 2);
    }

    #[test]
    fn builder_dry_run_status_sets_field() {
        let packet = EvidencePacket::builder(
            "rm test.txt".into(),
            CommandIntent::Mutate,
            ExecutionMode::DryRun,
        )
        .dry_run_status(DryRunStatus::PlanOnly)
        .build();
        assert_eq!(packet.dry_run_status, Some(DryRunStatus::PlanOnly));
    }

    #[test]
    fn builder_policy_decision_with_explicit_decision() {
        let decision = PolicyDecision {
            inferred_intent: CommandIntent::Deploy,
            zone: RiskZone::Red,
            matched_policy_entry: "policy.deploy.red".into(),
            matched_override: None,
            safety_escalations: Vec::new(),
            final_mode: ExecutionMode::Deny,
            rationale: "deploy blocked in red zone".into(),
        };
        let packet = EvidencePacket::builder(
            "kubectl apply".into(),
            CommandIntent::Deploy,
            ExecutionMode::Deny,
        )
        .policy_decision(decision)
        .build();
        assert_eq!(packet.policy_decision.final_mode, ExecutionMode::Deny);
        assert_eq!(packet.policy_decision.zone, RiskZone::Red);
    }
}
