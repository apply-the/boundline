//! Trace compaction domain types.
//!
//! This module defines the retention-class model, compaction actions,
//! metrics, and the classification policy used by the `boundline trace
//! compact` command. Every type is serializable for trace-visible event
//! emission and JSONL export.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The five retention classes that govern how a trace item survives
/// compaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionClass {
    /// Exact preservation required. Never destructively compacted.
    Lossless,
    /// Normalized into a structured event record. Source references
    /// preserved.
    Structured,
    /// Replaced with a lossy summary. Source references retained; summary
    /// must not become authority for completion decisions.
    Summary,
    /// Reduced to searchable metadata only. Usable for retrieval/navigation,
    /// not sufficient for edit, approval, or completion decisions.
    IndexOnly,
    /// Removable under retention policy. Never discard active stage evidence
    /// or rejection reasons.
    Discardable,
}

impl RetentionClass {
    /// All five retention classes.
    pub const fn all() -> [Self; 5] {
        [Self::Lossless, Self::Structured, Self::Summary, Self::IndexOnly, Self::Discardable]
    }

    /// Display name suitable for human-readable output.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Lossless => "lossless",
            Self::Structured => "structured",
            Self::Summary => "summary",
            Self::IndexOnly => "index-only",
            Self::Discardable => "discardable",
        }
    }

    /// Whether this class represents information-preserving storage.
    pub const fn is_preserving(self) -> bool {
        matches!(self, Self::Lossless | Self::Structured)
    }
}

/// A single trace item transformation recorded during compaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionAction {
    /// Identifier of the trace item being compacted.
    pub item_ref: String,
    /// The retention class assigned by the classification table (or
    /// tiebreaking) before any hard-rule override.
    pub from_class: RetentionClass,
    /// The final retention class after hard-rule overrides.
    pub to_class: RetentionClass,
    /// Whether the transformation is lossy (e.g., `from_class` was
    /// `Structured` and `to_class` is `Summary`).
    pub lossy: bool,
    /// Whether the classification was resolved by conservative tiebreaking
    /// rather than an explicit table entry.
    pub tiebreak: bool,
    /// When `true`, a hard survival rule (active stage evidence, rejection
    /// reason) overrode the classification-table result. The `from_class`
    /// holds the table-assigned class and `to_class` holds `Lossless`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub policy_override: bool,
    /// Reason for the policy override, populated when `policy_override` is
    /// `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,
}

/// Metrics captured during a single compaction run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionMetrics {
    /// Monotonic counter for this session's compaction runs.
    pub compaction_count: u64,
    /// Distribution of items across retention classes after compaction.
    pub class_distribution: HashMap<RetentionClass, u64>,
    /// Trace size in bytes before compaction.
    pub trace_size_before_bytes: u64,
    /// Trace size in bytes after compaction.
    pub trace_size_after_bytes: u64,
    /// Number of lossy transformations performed.
    pub lossy_count: u64,
    /// Number of accepted decisions preserved in exact form.
    pub preserved_decision_count: u64,
    /// Number of rejection reasons preserved in exact form.
    pub preserved_rejection_count: u64,
}

/// The compaction policy version identifier.
pub const COMPACTION_POLICY_VERSION: &str = "trace-compaction-v1";

/// Maximum number of trace items that compaction handles in a single
/// bounded pass. Traces exceeding this limit require operator confirmation
/// or chunked processing.
pub const COMPACTION_ITEM_LIMIT: u64 = 50_000;

/// Known trace item types that must be classified as [`RetentionClass::Lossless`].
pub const LOSSLESS_ITEM_TYPES: &[&str] = &[
    "accepted_decision",
    "approval",
    "final_stage_output",
    "rejection_reason",
    "operator_answer",
    "contract_validation_result",
    "evidence_reference",
    "release_validation_result",
    "active_stage_evidence",
];

/// Known trace item types that may be normalized into
/// [`RetentionClass::Structured`] event records.
pub const STRUCTURED_ITEM_TYPES: &[&str] = &[
    "guardian_finding",
    "provider_finding",
    "test_summary",
    "lint_summary",
    "phase_request",
    "route_decision",
    "context_selection_record",
];

/// Known trace item types that may be replaced with a
/// [`RetentionClass::Summary`].
pub const SUMMARY_ITEM_TYPES: &[&str] =
    &["assistant_transcript", "troubleshooting_attempt", "implementation_attempt", "command_log"];

/// Known trace item types that may be reduced to
/// [`RetentionClass::IndexOnly`] metadata.
pub const INDEX_ONLY_ITEM_TYPES: &[&str] =
    &["context_packet", "stale_trace_fragment", "intermediate_draft"];

/// Known trace item types eligible for [`RetentionClass::Discardable`].
pub const DISCARDABLE_ITEM_TYPES: &[&str] =
    &["duplicate_output", "temporary_debug_dump", "abandoned_local_diagnostic"];

/// Classification table that maps known trace item types to their default
/// retention class.
pub type ClassificationTable = HashMap<String, RetentionClass>;

/// Build the default classification table from the constant item-type lists.
///
/// The table assigns every known item type to its retention class. Types
/// not present in the table are resolved by conservative tiebreaking at
/// compaction time.
#[must_use]
pub fn default_classification_table() -> ClassificationTable {
    let mut table = ClassificationTable::with_capacity(
        LOSSLESS_ITEM_TYPES.len()
            + STRUCTURED_ITEM_TYPES.len()
            + SUMMARY_ITEM_TYPES.len()
            + INDEX_ONLY_ITEM_TYPES.len()
            + DISCARDABLE_ITEM_TYPES.len(),
    );
    for ty in LOSSLESS_ITEM_TYPES {
        table.insert((*ty).to_string(), RetentionClass::Lossless);
    }
    for ty in STRUCTURED_ITEM_TYPES {
        table.insert((*ty).to_string(), RetentionClass::Structured);
    }
    for ty in SUMMARY_ITEM_TYPES {
        table.insert((*ty).to_string(), RetentionClass::Summary);
    }
    for ty in INDEX_ONLY_ITEM_TYPES {
        table.insert((*ty).to_string(), RetentionClass::IndexOnly);
    }
    for ty in DISCARDABLE_ITEM_TYPES {
        table.insert((*ty).to_string(), RetentionClass::Discardable);
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_class_serialization_roundtrip() {
        for class in RetentionClass::all() {
            let json = serde_json::to_string(&class).unwrap();
            let parsed: RetentionClass = serde_json::from_str(&json).unwrap();
            assert_eq!(class, parsed);
        }
    }

    #[test]
    fn lossless_is_preserving() {
        assert!(RetentionClass::Lossless.is_preserving());
        assert!(RetentionClass::Structured.is_preserving());
        assert!(!RetentionClass::Summary.is_preserving());
    }

    #[test]
    fn display_name_returns_non_empty_for_all_classes() {
        for class in RetentionClass::all() {
            let name = class.display_name();
            assert!(!name.is_empty(), "empty display_name for {class:?}");
        }
        assert_eq!(RetentionClass::Lossless.display_name(), "lossless");
        assert_eq!(RetentionClass::Structured.display_name(), "structured");
        assert_eq!(RetentionClass::Summary.display_name(), "summary");
        assert_eq!(RetentionClass::IndexOnly.display_name(), "index-only");
        assert_eq!(RetentionClass::Discardable.display_name(), "discardable");
    }

    #[test]
    fn default_table_covers_all_known_types() {
        let table = default_classification_table();
        assert!(!table.is_empty());
        for ty in LOSSLESS_ITEM_TYPES {
            assert_eq!(
                table.get(*ty),
                Some(&RetentionClass::Lossless),
                "expected Lossless for {ty}"
            );
        }
        for ty in DISCARDABLE_ITEM_TYPES {
            assert_eq!(
                table.get(*ty),
                Some(&RetentionClass::Discardable),
                "expected Discardable for {ty}"
            );
        }
    }

    #[test]
    fn policy_version_is_non_empty() {
        assert!(!COMPACTION_POLICY_VERSION.is_empty());
    }

    #[test]
    fn compaction_action_lossy_flag_consistent() {
        let action = CompactionAction {
            item_ref: "item-1".into(),
            from_class: RetentionClass::Structured,
            to_class: RetentionClass::Summary,
            lossy: true,
            tiebreak: false,
            policy_override: false,
            override_reason: None,
        };
        assert!(action.lossy);

        let action2 = CompactionAction {
            item_ref: "item-2".into(),
            from_class: RetentionClass::Lossless,
            to_class: RetentionClass::Lossless,
            lossy: false,
            tiebreak: false,
            policy_override: false,
            override_reason: None,
        };
        assert!(!action2.lossy);
    }

    #[test]
    fn policy_override_action_records_reason() {
        let action = CompactionAction {
            item_ref: "stage-evidence-1".into(),
            from_class: RetentionClass::Structured,
            to_class: RetentionClass::Lossless,
            lossy: false,
            tiebreak: false,
            policy_override: true,
            override_reason: Some("active stage evidence must never be compacted".into()),
        };
        assert!(action.policy_override);
        assert!(action.override_reason.is_some());
    }
}
