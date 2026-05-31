//! Configuration-side records for explicit framework-adapter selection.

use serde::{Deserialize, Serialize};

use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, StoredAdapterConfigValueState,
};

/// Persisted top-level adapter block stored in `.boundline/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedAdapterConfiguration {
    /// Operator-selected activation record.
    #[serde(flatten)]
    pub selection: AdapterSelectionRecord,
    /// Fingerprint of the config schema these values match.
    pub schema_fingerprint: String,
    /// Overall completeness of the stored value set.
    pub completeness_state: AdapterConfigCompletenessState,
    /// Whether the latest successful resolution used guided prompts.
    #[serde(default)]
    pub interactive_resolution: bool,
    /// Timestamp of the last successful validation pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<u64>,
    /// Materialized count of stored values for operator summaries.
    pub value_count: usize,
    /// Stored config values rendered as `[[adapter.values]]` in TOML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<AdapterConfigValueRecord>,
}

/// Persisted adapter selection written into workspace or global config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSelectionRecord {
    /// Operator-selected activation mode.
    pub selection_mode: AdapterSelectionMode,
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Human-facing adapter display name.
    pub display_name: String,
    /// Command or absolute path used to launch the adapter.
    pub command: String,
    /// Additional launch arguments persisted with the selection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Source that created the selection.
    pub registration_source: AdapterRegistrationSource,
    /// Discovery outcome recorded at selection time.
    pub discovery_state: AdapterDiscoveryState,
    /// Stable compatibility line expected by the host.
    pub compatibility_line: String,
    /// Timestamp of the last selection update.
    pub updated_at: u64,
}

/// Host-shipped default value for a known adapter profile field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownAdapterProfileFieldDefault {
    /// Stable field identifier.
    pub field_key: String,
    /// Serialized default value written into guided prompts.
    pub value_text: String,
}

/// Known adapter profile definition owned by the host registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownAdapterProfileDefinition {
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Human-facing adapter name.
    pub display_name: String,
    /// Default command used for registration and discovery.
    pub default_command: String,
    /// CLI alias accepted by registration flows.
    pub registration_alias: String,
    /// Canonical repository reference for the adapter implementation.
    pub adapter_repo_ref: String,
    /// Canonical repository reference for the framework template.
    pub template_repo_ref: String,
    /// Stable compatibility line expected by the profile.
    pub compatibility_line: String,
    /// Binary names that can be suggested during PATH discovery.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discovery_names: Vec<String>,
    /// Default field values prefilled during guided setup.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefilled_fields: Vec<KnownAdapterProfileFieldDefault>,
}

/// One resolved adapter config value stored for later preflight and execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterConfigValueRecord {
    /// Stable field identifier.
    pub field_key: String,
    /// Declared type of the field value.
    pub value_kind: AdapterValueKind,
    /// Whether the value must be redacted from operator-facing projections.
    #[serde(default)]
    pub secret: bool,
    /// String payload when the value kind is string or enum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    /// Filesystem path payload when the value kind is path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_value: Option<String>,
    /// Boolean payload when the value kind is boolean.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bool_value: Option<bool>,
    /// Integer payload when the value kind is integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub int_value: Option<i64>,
    /// Source that resolved the value.
    pub value_source: AdapterValueSource,
    /// Whether the value is usable for execution.
    pub resolution_state: StoredAdapterConfigValueState,
}

/// Stored set of resolved config values for the active adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedAdapterConfigSet {
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Fingerprint of the schema that produced these values.
    pub schema_fingerprint: String,
    /// Overall completeness state derived from the stored values.
    pub completeness_state: AdapterConfigCompletenessState,
    /// Whether the values were resolved through interactive prompts.
    #[serde(default)]
    pub interactive_resolution: bool,
    /// Timestamp of the last validation pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<u64>,
    /// Materialized count of stored values for quick operator summaries.
    pub value_count: usize,
    /// Stored field values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<AdapterConfigValueRecord>,
}

impl PersistedAdapterConfiguration {
    /// Returns the runtime validation projection for the persisted adapter values.
    pub fn resolved_config(&self) -> ResolvedAdapterConfigSet {
        ResolvedAdapterConfigSet {
            adapter_id: self.selection.adapter_id.clone(),
            schema_fingerprint: self.schema_fingerprint.clone(),
            completeness_state: self.completeness_state,
            interactive_resolution: self.interactive_resolution,
            last_validated_at: self.last_validated_at,
            value_count: self.value_count,
            values: self.values.clone(),
        }
    }
}
