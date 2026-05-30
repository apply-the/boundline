//! Routing, runtime-capability, and domain-template configuration models.

use std::collections::BTreeMap;
use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::context_intelligence::{
    RemoteTransmissionPolicyState, RetrievalBudgets, RetrievalMode,
};
use crate::domain::domain_templates::{
    DomainFamily, DomainTemplateSettings, ExternalContextBinding,
};
use crate::domain::governance::CanonModeSelectionPreference;

/// Supported assistant runtimes that can back configured routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Claude,
    Codex,
    Copilot,
    Gemini,
}

impl RuntimeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Gemini => "gemini",
        }
    }
}

impl fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Assistant package hosts that `boundline init --assistant` can scaffold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "kebab-case")]
pub enum AssistantHostKind {
    Claude,
    Codex,
    Copilot,
    Antigravity,
}

/// IDE setup surfaces that `boundline init --ide` can scaffold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum IdeKind {
    #[serde(rename = "vscode")]
    #[value(name = "vscode")]
    VsCode,
    Cursor,
    Antigravity,
    #[serde(rename = "jetbrains")]
    #[value(name = "jetbrains")]
    JetBrains,
}

impl IdeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VsCode => "vscode",
            Self::Cursor => "cursor",
            Self::Antigravity => "antigravity",
            Self::JetBrains => "jetbrains",
        }
    }
}

impl fmt::Display for IdeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Terminal command auto-approval profile for IDEs that expose a stable schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum TerminalAutoApproveProfile {
    ReadOnly,
    SessionSafe,
    Trusted,
}

impl TerminalAutoApproveProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::SessionSafe => "session-safe",
            Self::Trusted => "trusted",
        }
    }
}

impl fmt::Display for TerminalAutoApproveProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AssistantHostKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Antigravity => "antigravity",
        }
    }

    pub const fn default_runtime(self) -> Option<RuntimeKind> {
        match self {
            Self::Claude => Some(RuntimeKind::Claude),
            Self::Codex => Some(RuntimeKind::Codex),
            Self::Copilot => Some(RuntimeKind::Copilot),
            Self::Antigravity => None,
        }
    }

    pub const fn from_runtime(runtime: RuntimeKind) -> Option<Self> {
        match runtime {
            RuntimeKind::Claude => Some(Self::Claude),
            RuntimeKind::Codex => Some(Self::Codex),
            RuntimeKind::Copilot => Some(Self::Copilot),
            RuntimeKind::Gemini => None,
        }
    }
}

impl fmt::Display for AssistantHostKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Built-in init templates exposed by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum InitTemplate {
    BugFix,
    Change,
    Delivery,
}

/// Configuration scope targeted by init bootstrap flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum InitConfigScope {
    Global,
    Workspace,
    Both,
}

impl InitConfigScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Workspace => "workspace",
            Self::Both => "both",
        }
    }
}

impl fmt::Display for InitConfigScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Named routing slot used by effective model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RouteSlot {
    Planning,
    Implementation,
    Verification,
    Review,
}

impl RouteSlot {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Implementation => "implementation",
            Self::Verification => "verification",
            Self::Review => "review",
        }
    }
}

/// Config display scope accepted by configuration inspection commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ConfigShowScope {
    Effective,
    Workspace,
    Cluster,
    Global,
}

/// Config write scope accepted by configuration mutation commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ConfigWriteScope {
    Workspace,
    Cluster,
    Global,
}

/// Capability support state for runtime feature flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    Unsupported,
}

impl CapabilityState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
        }
    }

    pub const fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}

impl fmt::Display for CapabilityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Requested effort level for a routing slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    Max,
}

impl EffortLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }
}

impl fmt::Display for EffortLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Fallback policy when the requested effort level is unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum EffortFallbackPolicy {
    Preserve,
    AllowLower,
}

impl EffortFallbackPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preserve => "preserve",
            Self::AllowLower => "allow_lower",
        }
    }
}

impl fmt::Display for EffortFallbackPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Capability profile describing what one runtime can support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilityProfile {
    pub continuation: CapabilityState,
    pub resume: CapabilityState,
    pub validation: CapabilityState,
    pub handoff_target: CapabilityState,
    pub escalation_context: CapabilityState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl RuntimeCapabilityProfile {
    /// Validates the capability profile.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.notes.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(ConfigurationError::InvalidRuntimeCapability(
                "notes cannot be empty when provided".to_string(),
            ));
        }

        if self.handoff_target.is_supported() && !self.continuation.is_supported() {
            return Err(ConfigurationError::InvalidRuntimeCapability(
                "handoff_target requires continuation support".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns a compact human-readable summary of the profile.
    pub fn summary_text(&self) -> String {
        let mut parts = vec![
            format!("continuation={}", self.continuation),
            format!("resume={}", self.resume),
            format!("validation={}", self.validation),
            format!("handoff_target={}", self.handoff_target),
            format!("escalation_context={}", self.escalation_context),
        ];
        if let Some(notes) = self.notes.as_deref().map(str::trim).filter(|value| !value.is_empty())
        {
            parts.push(format!("notes={notes}"));
        }
        parts.join(", ")
    }
}

/// Slot-specific effort policy layered through config precedence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotEffortPolicy {
    pub level: EffortLevel,
    pub fallback: EffortFallbackPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

impl SlotEffortPolicy {
    /// Validates the slot effort policy.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.rationale.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(ConfigurationError::InvalidSlotEffortPolicy(
                "rationale cannot be empty when provided".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns a compact human-readable summary of the policy.
    pub fn summary_text(&self) -> String {
        let mut summary = format!("level={}, fallback={}", self.level, self.fallback);
        if let Some(rationale) =
            self.rationale.as_deref().map(str::trim).filter(|value| !value.is_empty())
        {
            summary.push_str(&format!(", rationale={rationale}"));
        }
        summary
    }
}

/// Typed advanced-context retrieval policy layered through configuration precedence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedContextConfig {
    #[serde(default = "default_retrieval_mode")]
    pub retrieval_mode: RetrievalMode,
    #[serde(default = "default_remote_policy_state")]
    pub remote_policy: RemoteTransmissionPolicyState,
    #[serde(default)]
    pub budgets: RetrievalBudgets,
}

impl Default for AdvancedContextConfig {
    fn default() -> Self {
        Self {
            retrieval_mode: default_retrieval_mode(),
            remote_policy: default_remote_policy_state(),
            budgets: RetrievalBudgets::default(),
        }
    }
}

impl AdvancedContextConfig {
    /// Validates the advanced-context retrieval policy.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        self.budgets
            .validate()
            .map_err(|error| ConfigurationError::InvalidAdvancedContextConfig(error.to_string()))?;

        if self.retrieval_mode == RetrievalMode::Remote {
            return Err(ConfigurationError::InvalidAdvancedContextConfig(
                "remote retrieval mode is not supported in the local-only V1 engine".to_string(),
            ));
        }

        if self.remote_policy == RemoteTransmissionPolicyState::RemoteAllowed {
            return Err(ConfigurationError::InvalidAdvancedContextConfig(
                "remote transmission is not supported in the local-only V1 engine".to_string(),
            ));
        }

        if self.retrieval_mode == RetrievalMode::Disabled
            && self.remote_policy != RemoteTransmissionPolicyState::Blocked
        {
            return Err(ConfigurationError::InvalidAdvancedContextConfig(
                "disabled retrieval requires blocked remote policy".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns a compact human-readable summary of the policy.
    pub fn summary_text(&self) -> String {
        format!(
            "mode={}, remote_policy={}, budgets=refinement:{}, refresh:{}, depth:{}, expansion:{}, traversal:{}, evidence:{}",
            self.retrieval_mode.as_str(),
            self.remote_policy.as_str(),
            self.budgets.refinement_budget,
            self.budgets.refresh_budget,
            self.budgets.depth_limit,
            self.budgets.expansion_limit,
            self.budgets.traversal_limit,
            self.budgets.evidence_limit,
        )
    }
}

const fn default_retrieval_mode() -> RetrievalMode {
    RetrievalMode::Local
}

const fn default_remote_policy_state() -> RemoteTransmissionPolicyState {
    RemoteTransmissionPolicyState::LocalOnly
}

/// Semantic-acceleration policy states exposed by the dedicated config surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SemanticAccelerationPolicyState {
    Disabled,
    Local,
}

impl SemanticAccelerationPolicyState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Local => "local",
        }
    }
}

impl fmt::Display for SemanticAccelerationPolicyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Git hook actions supported for the disposable semantic derived index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SemanticIndexHookAction {
    Disabled,
    MarkStale,
}

impl SemanticIndexHookAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::MarkStale => "mark_stale",
        }
    }
}

impl fmt::Display for SemanticIndexHookAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed semantic-acceleration policy layered through configuration precedence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAccelerationPolicy {
    #[serde(default = "default_semantic_acceleration_policy_state")]
    pub policy: SemanticAccelerationPolicyState,
    #[serde(default = "default_semantic_index_hook_action")]
    pub index_hook_action: SemanticIndexHookAction,
}

impl Default for SemanticAccelerationPolicy {
    fn default() -> Self {
        Self {
            policy: default_semantic_acceleration_policy_state(),
            index_hook_action: default_semantic_index_hook_action(),
        }
    }
}

impl SemanticAccelerationPolicy {
    /// Validates the semantic-acceleration policy.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.index_hook_action == SemanticIndexHookAction::MarkStale
            && self.policy != SemanticAccelerationPolicyState::Local
        {
            return Err(ConfigurationError::InvalidAdvancedContextConfig(
                "semantic index hooks require local semantic acceleration".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns a compact human-readable summary of the policy.
    pub fn summary_text(&self) -> String {
        format!("policy={}, index_hook_action={}", self.policy, self.index_hook_action)
    }
}

const fn default_semantic_acceleration_policy_state() -> SemanticAccelerationPolicyState {
    SemanticAccelerationPolicyState::Disabled
}

const fn default_semantic_index_hook_action() -> SemanticIndexHookAction {
    SemanticIndexHookAction::Disabled
}

/// Concrete runtime and model selected for a route slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRoute {
    pub runtime: RuntimeKind,
    pub model: String,
}

impl ModelRoute {
    /// Validates the route payload.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.model.trim().is_empty() {
            return Err(ConfigurationError::MissingModelId);
        }
        Ok(())
    }
}

/// Persisted routing configuration layered across workspace, cluster, and global scopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default)]
    pub planning: Option<ModelRoute>,
    #[serde(default)]
    pub implementation: Option<ModelRoute>,
    #[serde(default)]
    pub verification: Option<ModelRoute>,
    #[serde(default)]
    pub review: Option<ModelRoute>,
    #[serde(default)]
    pub chat: Option<ModelRoute>,
    #[serde(default)]
    pub reviewer_roles: BTreeMap<String, ModelRoute>,
    #[serde(default)]
    pub adjudication: Option<ModelRoute>,
    #[serde(default)]
    pub assistant_hosts: Vec<AssistantHostKind>,
    #[serde(default)]
    pub assistant_runtimes: Vec<RuntimeKind>,
    #[serde(default)]
    pub runtime_capabilities: BTreeMap<RuntimeKind, RuntimeCapabilityProfile>,
    #[serde(default)]
    pub slot_effort_policies: BTreeMap<RouteSlot, SlotEffortPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advanced_context: Option<AdvancedContextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_acceleration: Option<SemanticAccelerationPolicy>,
    #[serde(default)]
    pub domain_templates: BTreeMap<DomainFamily, DomainTemplateSettings>,
}

impl RoutingConfig {
    /// Validates the routing configuration and nested policies.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        for route in [
            self.planning.as_ref(),
            self.implementation.as_ref(),
            self.verification.as_ref(),
            self.review.as_ref(),
            self.chat.as_ref(),
            self.adjudication.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            route.validate()?;
        }

        for (role, route) in &self.reviewer_roles {
            if role.trim().is_empty() {
                return Err(ConfigurationError::InvalidReviewerRole(
                    "role id cannot be empty".to_string(),
                ));
            }
            route.validate()?;
        }

        for profile in self.runtime_capabilities.values() {
            profile.validate()?;
        }

        for policy in self.slot_effort_policies.values() {
            policy.validate()?;
        }

        if let Some(policy) = self.advanced_context.as_ref() {
            policy.validate()?;
        }

        if let Some(policy) = self.semantic_acceleration.as_ref() {
            policy.validate()?;
        }

        for settings in self.domain_templates.values() {
            settings
                .validate()
                .map_err(|error| ConfigurationError::InvalidDomainTemplate(error.to_string()))?;
        }

        Ok(())
    }

    pub fn set_slot(&mut self, slot: RouteSlot, route: ModelRoute) {
        match slot {
            RouteSlot::Planning => self.planning = Some(route),
            RouteSlot::Implementation => self.implementation = Some(route),
            RouteSlot::Verification => self.verification = Some(route),
            RouteSlot::Review => self.review = Some(route),
        }
    }

    pub fn unset_slot(&mut self, slot: RouteSlot) {
        match slot {
            RouteSlot::Planning => self.planning = None,
            RouteSlot::Implementation => self.implementation = None,
            RouteSlot::Verification => self.verification = None,
            RouteSlot::Review => self.review = None,
        }
    }

    pub fn set_runtime_capability(
        &mut self,
        runtime: RuntimeKind,
        profile: RuntimeCapabilityProfile,
    ) {
        self.runtime_capabilities.insert(runtime, profile);
    }

    pub fn unset_runtime_capability(&mut self, runtime: RuntimeKind) {
        self.runtime_capabilities.remove(&runtime);
    }

    pub fn set_slot_effort_policy(&mut self, slot: RouteSlot, policy: SlotEffortPolicy) {
        self.slot_effort_policies.insert(slot, policy);
    }

    pub fn unset_slot_effort_policy(&mut self, slot: RouteSlot) {
        self.slot_effort_policies.remove(&slot);
    }

    pub fn set_semantic_acceleration_policy(&mut self, policy: SemanticAccelerationPolicy) {
        self.semantic_acceleration = Some(policy);
    }

    pub fn unset_semantic_acceleration_policy(&mut self) {
        self.semantic_acceleration = None;
    }

    pub fn set_domain_template_settings(
        &mut self,
        family: DomainFamily,
        settings: DomainTemplateSettings,
    ) {
        self.domain_templates.insert(family, settings);
    }

    pub fn unset_domain_template_settings(&mut self, family: DomainFamily) {
        self.domain_templates.remove(&family);
    }
}

/// Returns the default model route for a given assistant runtime.
pub fn assistant_default_model_route(runtime: RuntimeKind) -> ModelRoute {
    match runtime {
        RuntimeKind::Claude => ModelRoute { runtime, model: "sonnet-4".to_string() },
        RuntimeKind::Codex => ModelRoute { runtime, model: "o4-mini".to_string() },
        RuntimeKind::Copilot => ModelRoute { runtime, model: "gpt-4.1".to_string() },
        RuntimeKind::Gemini => ModelRoute { runtime, model: "gemini-2.5-pro".to_string() },
    }
}

/// Returns the built-in default route for a routing slot.
pub fn built_in_default_route(slot: RouteSlot) -> ModelRoute {
    let defaults = built_in_defaults();
    match slot {
        RouteSlot::Planning => defaults.planning,
        RouteSlot::Implementation => defaults.implementation,
        RouteSlot::Verification => defaults.verification,
        RouteSlot::Review => defaults.review,
    }
}

/// Seeds routing slots from the selected assistants while preserving built-in preferences when possible.
pub fn seeded_routes_for_assistants(assistants: &[RuntimeKind]) -> BTreeMap<RouteSlot, ModelRoute> {
    let Some(fallback_runtime) = assistants.first().copied() else {
        return BTreeMap::new();
    };

    [RouteSlot::Planning, RouteSlot::Implementation, RouteSlot::Verification, RouteSlot::Review]
        .into_iter()
        .map(|slot| {
            let preferred = built_in_default_route(slot);
            let route = if assistants.contains(&preferred.runtime) {
                preferred
            } else {
                assistant_default_model_route(fallback_runtime)
            };
            (slot, route)
        })
        .collect()
}

/// Default Canon governance preferences carried in config files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonPreferences {
    #[serde(default)]
    pub mode_selection: CanonModeSelectionPreference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_system_context: Option<String>,
}

/// Root persisted config file shared by workspace and global configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon: Option<CanonPreferences>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self { version: default_version(), routing: RoutingConfig::default(), canon: None }
    }
}

fn default_version() -> u32 {
    1
}

/// Source layer that contributed an effective value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Cli,
    Workspace,
    Cluster,
    Global,
    BuiltIn,
}

/// Route together with the precedence layer that supplied it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedRoute {
    pub route: ModelRoute,
    pub source: ValueSource,
}

/// Fully resolved routing view after precedence rules are applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveRouting {
    pub planning: SourcedRoute,
    pub implementation: SourcedRoute,
    pub verification: SourcedRoute,
    pub review: SourcedRoute,
    pub chat: Option<SourcedRoute>,
    pub adjudication: SourcedRoute,
    pub reviewer_roles: BTreeMap<String, SourcedRoute>,
}

/// Runtime capability profile annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedRuntimeCapabilityProfile {
    pub profile: RuntimeCapabilityProfile,
    pub source: ValueSource,
}

/// Slot effort policy annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedSlotEffortPolicy {
    pub policy: SlotEffortPolicy,
    pub source: ValueSource,
}

/// Advanced-context policy annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedAdvancedContextConfig {
    pub policy: AdvancedContextConfig,
    pub source: ValueSource,
}

/// Semantic-acceleration policy annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedSemanticAccelerationPolicy {
    pub policy: SemanticAccelerationPolicy,
    pub source: ValueSource,
}

/// Domain standards layer annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedDomainStandardsLayer {
    pub text: String,
    pub source: ValueSource,
}

/// External context binding annotated with its value source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedExternalContextBinding {
    pub binding: ExternalContextBinding,
    pub source: ValueSource,
}

/// Resolved domain-template view after enablement and layering rules are applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDomainTemplate {
    pub enabled: bool,
    pub enablement_source: ValueSource,
    pub standards_layers: Vec<SourcedDomainStandardsLayer>,
    pub external_context_bindings: Vec<SourcedExternalContextBinding>,
}

/// CLI routing overrides applied ahead of persisted configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RoutingOverrides {
    pub planning: Option<ModelRoute>,
    pub implementation: Option<ModelRoute>,
    pub verification: Option<ModelRoute>,
    pub review: Option<ModelRoute>,
    pub chat: Option<ModelRoute>,
    pub adjudication: Option<ModelRoute>,
    pub reviewer_roles: BTreeMap<String, ModelRoute>,
}

/// Resolves the effective route for every slot using CLI, workspace, cluster, global, and built-in precedence.
pub fn resolve_effective_routing(
    cli: &RoutingOverrides,
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> EffectiveRouting {
    let defaults = built_in_defaults();

    let planning = resolve_single(
        cli.planning.as_ref(),
        workspace.and_then(|cfg| cfg.planning.as_ref()),
        cluster.and_then(|cfg| cfg.planning.as_ref()),
        global.and_then(|cfg| cfg.planning.as_ref()),
        &defaults.planning,
    );
    let implementation = resolve_single(
        cli.implementation.as_ref(),
        workspace.and_then(|cfg| cfg.implementation.as_ref()),
        cluster.and_then(|cfg| cfg.implementation.as_ref()),
        global.and_then(|cfg| cfg.implementation.as_ref()),
        &defaults.implementation,
    );
    let verification = resolve_single(
        cli.verification.as_ref(),
        workspace.and_then(|cfg| cfg.verification.as_ref()),
        cluster.and_then(|cfg| cfg.verification.as_ref()),
        global.and_then(|cfg| cfg.verification.as_ref()),
        &defaults.verification,
    );
    let review = resolve_single(
        cli.review.as_ref(),
        workspace.and_then(|cfg| cfg.review.as_ref()),
        cluster.and_then(|cfg| cfg.review.as_ref()),
        global.and_then(|cfg| cfg.review.as_ref()),
        &defaults.review,
    );
    let chat = resolve_optional_single(
        cli.chat.as_ref(),
        workspace.and_then(|cfg| cfg.chat.as_ref()),
        cluster.and_then(|cfg| cfg.chat.as_ref()),
        global.and_then(|cfg| cfg.chat.as_ref()),
    );
    let adjudication = resolve_single(
        cli.adjudication.as_ref(),
        workspace.and_then(|cfg| cfg.adjudication.as_ref()),
        cluster.and_then(|cfg| cfg.adjudication.as_ref()),
        global.and_then(|cfg| cfg.adjudication.as_ref()),
        &defaults.adjudication,
    );

    let mut reviewer_roles = BTreeMap::new();
    let mut role_ids = BTreeMap::<String, ()>::new();
    for key in cli
        .reviewer_roles
        .keys()
        .chain(workspace.into_iter().flat_map(|cfg| cfg.reviewer_roles.keys()))
        .chain(cluster.into_iter().flat_map(|cfg| cfg.reviewer_roles.keys()))
        .chain(global.into_iter().flat_map(|cfg| cfg.reviewer_roles.keys()))
    {
        role_ids.insert(key.clone(), ());
    }

    for role_id in role_ids.into_keys() {
        let route = resolve_single(
            cli.reviewer_roles.get(&role_id),
            workspace.and_then(|cfg| cfg.reviewer_roles.get(&role_id)),
            cluster.and_then(|cfg| cfg.reviewer_roles.get(&role_id)),
            global.and_then(|cfg| cfg.reviewer_roles.get(&role_id)),
            &review.route,
        );
        reviewer_roles.insert(role_id, route);
    }

    EffectiveRouting {
        planning,
        implementation,
        verification,
        review,
        chat,
        adjudication,
        reviewer_roles,
    }
}

/// Resolves effective runtime capability profiles across config layers.
pub fn resolve_effective_runtime_capabilities(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> BTreeMap<RuntimeKind, SourcedRuntimeCapabilityProfile> {
    let mut runtime_ids = BTreeMap::<RuntimeKind, ()>::new();
    for runtime in workspace
        .into_iter()
        .flat_map(|cfg| cfg.runtime_capabilities.keys())
        .chain(cluster.into_iter().flat_map(|cfg| cfg.runtime_capabilities.keys()))
        .chain(global.into_iter().flat_map(|cfg| cfg.runtime_capabilities.keys()))
    {
        runtime_ids.insert(*runtime, ());
    }

    runtime_ids
        .into_keys()
        .filter_map(|runtime| {
            resolve_runtime_capability_profile(runtime, workspace, cluster, global)
                .map(|profile| (runtime, profile))
        })
        .collect()
}

fn resolve_runtime_capability_profile(
    runtime: RuntimeKind,
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> Option<SourcedRuntimeCapabilityProfile> {
    if let Some(profile) = workspace.and_then(|cfg| cfg.runtime_capabilities.get(&runtime)) {
        return Some(SourcedRuntimeCapabilityProfile {
            profile: profile.clone(),
            source: ValueSource::Workspace,
        });
    }

    if let Some(profile) = cluster.and_then(|cfg| cfg.runtime_capabilities.get(&runtime)) {
        return Some(SourcedRuntimeCapabilityProfile {
            profile: profile.clone(),
            source: ValueSource::Cluster,
        });
    }

    global.and_then(|cfg| cfg.runtime_capabilities.get(&runtime)).map(|profile| {
        SourcedRuntimeCapabilityProfile { profile: profile.clone(), source: ValueSource::Global }
    })
}

/// Resolves the effective advanced-context retrieval policy across config layers.
pub fn resolve_effective_advanced_context_config(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> SourcedAdvancedContextConfig {
    if let Some(policy) = workspace.and_then(|cfg| cfg.advanced_context.as_ref()) {
        return SourcedAdvancedContextConfig {
            policy: policy.clone(),
            source: ValueSource::Workspace,
        };
    }

    if let Some(policy) = cluster.and_then(|cfg| cfg.advanced_context.as_ref()) {
        return SourcedAdvancedContextConfig {
            policy: policy.clone(),
            source: ValueSource::Cluster,
        };
    }

    if let Some(policy) = global.and_then(|cfg| cfg.advanced_context.as_ref()) {
        return SourcedAdvancedContextConfig {
            policy: policy.clone(),
            source: ValueSource::Global,
        };
    }

    SourcedAdvancedContextConfig {
        policy: AdvancedContextConfig::default(),
        source: ValueSource::BuiltIn,
    }
}

/// Resolves the effective semantic-acceleration policy across config layers.
pub fn resolve_effective_semantic_acceleration_config(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> SourcedSemanticAccelerationPolicy {
    if let Some(policy) = workspace.and_then(|cfg| cfg.semantic_acceleration.as_ref()) {
        return SourcedSemanticAccelerationPolicy {
            policy: policy.clone(),
            source: ValueSource::Workspace,
        };
    }

    if let Some(policy) = cluster.and_then(|cfg| cfg.semantic_acceleration.as_ref()) {
        return SourcedSemanticAccelerationPolicy {
            policy: policy.clone(),
            source: ValueSource::Cluster,
        };
    }

    if let Some(policy) = global.and_then(|cfg| cfg.semantic_acceleration.as_ref()) {
        return SourcedSemanticAccelerationPolicy {
            policy: policy.clone(),
            source: ValueSource::Global,
        };
    }

    SourcedSemanticAccelerationPolicy {
        policy: SemanticAccelerationPolicy::default(),
        source: ValueSource::BuiltIn,
    }
}

/// Resolves effective slot effort policies across config layers.
pub fn resolve_effective_slot_effort_policies(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> BTreeMap<RouteSlot, SourcedSlotEffortPolicy> {
    let mut slots = BTreeMap::<RouteSlot, ()>::new();
    for slot in workspace
        .into_iter()
        .flat_map(|cfg| cfg.slot_effort_policies.keys())
        .chain(cluster.into_iter().flat_map(|cfg| cfg.slot_effort_policies.keys()))
        .chain(global.into_iter().flat_map(|cfg| cfg.slot_effort_policies.keys()))
    {
        slots.insert(*slot, ());
    }

    slots
        .into_keys()
        .filter_map(|slot| {
            let sourced = if let Some(policy) =
                workspace.and_then(|cfg| cfg.slot_effort_policies.get(&slot))
            {
                Some(SourcedSlotEffortPolicy {
                    policy: policy.clone(),
                    source: ValueSource::Workspace,
                })
            } else if let Some(policy) = cluster.and_then(|cfg| cfg.slot_effort_policies.get(&slot))
            {
                Some(SourcedSlotEffortPolicy {
                    policy: policy.clone(),
                    source: ValueSource::Cluster,
                })
            } else {
                global.and_then(|cfg| cfg.slot_effort_policies.get(&slot)).map(|policy| {
                    SourcedSlotEffortPolicy { policy: policy.clone(), source: ValueSource::Global }
                })
            };

            sourced.map(|policy| (slot, policy))
        })
        .collect()
}

/// Resolves effective domain-template settings across config layers.
pub fn resolve_effective_domain_templates(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> BTreeMap<DomainFamily, ResolvedDomainTemplate> {
    let mut family_ids = BTreeMap::<DomainFamily, ()>::new();
    for family in workspace
        .into_iter()
        .flat_map(|cfg| cfg.domain_templates.keys())
        .chain(cluster.into_iter().flat_map(|cfg| cfg.domain_templates.keys()))
        .chain(global.into_iter().flat_map(|cfg| cfg.domain_templates.keys()))
    {
        family_ids.insert(*family, ());
    }

    family_ids
        .into_keys()
        .map(|family| {
            let workspace_settings = workspace.and_then(|cfg| cfg.domain_templates.get(&family));
            let cluster_settings = cluster.and_then(|cfg| cfg.domain_templates.get(&family));
            let global_settings = global.and_then(|cfg| cfg.domain_templates.get(&family));

            let (enabled, enablement_source) = if let Some(enabled) =
                workspace_settings.and_then(|settings| settings.enabled)
            {
                (enabled, ValueSource::Workspace)
            } else if let Some(enabled) = cluster_settings.and_then(|settings| settings.enabled) {
                (enabled, ValueSource::Cluster)
            } else if let Some(enabled) = global_settings.and_then(|settings| settings.enabled) {
                (enabled, ValueSource::Global)
            } else {
                (false, ValueSource::BuiltIn)
            };

            let mut standards_layers = vec![SourcedDomainStandardsLayer {
                text: family.built_in_summary().to_string(),
                source: ValueSource::BuiltIn,
            }];
            for (settings, source) in [
                (global_settings, ValueSource::Global),
                (cluster_settings, ValueSource::Cluster),
                (workspace_settings, ValueSource::Workspace),
            ] {
                if let Some(text) = settings
                    .and_then(|settings| settings.standards.as_deref())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    standards_layers
                        .push(SourcedDomainStandardsLayer { text: text.to_string(), source });
                }
            }

            let mut external_context_bindings = Vec::new();
            for (settings, source) in [
                (global_settings, ValueSource::Global),
                (cluster_settings, ValueSource::Cluster),
                (workspace_settings, ValueSource::Workspace),
            ] {
                if let Some(settings) = settings {
                    external_context_bindings.extend(
                        settings
                            .external_context_bindings
                            .iter()
                            .cloned()
                            .map(|binding| SourcedExternalContextBinding { binding, source }),
                    );
                }
            }

            (
                family,
                ResolvedDomainTemplate {
                    enabled,
                    enablement_source,
                    standards_layers,
                    external_context_bindings,
                },
            )
        })
        .collect()
}

fn resolve_single(
    cli: Option<&ModelRoute>,
    workspace: Option<&ModelRoute>,
    cluster: Option<&ModelRoute>,
    global: Option<&ModelRoute>,
    default: &ModelRoute,
) -> SourcedRoute {
    if let Some(route) = cli {
        return SourcedRoute { route: route.clone(), source: ValueSource::Cli };
    }
    if let Some(route) = workspace {
        return SourcedRoute { route: route.clone(), source: ValueSource::Workspace };
    }
    if let Some(route) = cluster {
        return SourcedRoute { route: route.clone(), source: ValueSource::Cluster };
    }
    if let Some(route) = global {
        return SourcedRoute { route: route.clone(), source: ValueSource::Global };
    }
    SourcedRoute { route: default.clone(), source: ValueSource::BuiltIn }
}

fn resolve_optional_single(
    cli: Option<&ModelRoute>,
    workspace: Option<&ModelRoute>,
    cluster: Option<&ModelRoute>,
    global: Option<&ModelRoute>,
) -> Option<SourcedRoute> {
    if let Some(route) = cli {
        return Some(SourcedRoute { route: route.clone(), source: ValueSource::Cli });
    }
    if let Some(route) = workspace {
        return Some(SourcedRoute { route: route.clone(), source: ValueSource::Workspace });
    }
    if let Some(route) = cluster {
        return Some(SourcedRoute { route: route.clone(), source: ValueSource::Cluster });
    }
    global.map(|route| SourcedRoute { route: route.clone(), source: ValueSource::Global })
}

struct BuiltInDefaults {
    planning: ModelRoute,
    implementation: ModelRoute,
    verification: ModelRoute,
    review: ModelRoute,
    adjudication: ModelRoute,
}

fn built_in_defaults() -> BuiltInDefaults {
    BuiltInDefaults {
        planning: ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() },
        implementation: ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() },
        verification: ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4.1".to_string() },
        review: ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
        adjudication: ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() },
    }
}

/// Validation errors for configuration models and nested policies.
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("model id cannot be empty")]
    MissingModelId,
    #[error("invalid reviewer role: {0}")]
    InvalidReviewerRole(String),
    #[error("invalid runtime capability profile: {0}")]
    InvalidRuntimeCapability(String),
    #[error("invalid slot effort policy: {0}")]
    InvalidSlotEffortPolicy(String),
    #[error("invalid advanced-context policy: {0}")]
    InvalidAdvancedContextConfig(String),
    #[error("invalid domain template settings: {0}")]
    InvalidDomainTemplate(String),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::domain::context_intelligence::{
        RemoteTransmissionPolicyState, RetrievalBudgets, RetrievalMode,
    };
    use crate::domain::domain_templates::{
        DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
    };

    use super::{
        AdvancedContextConfig, CapabilityState, ConfigurationError, EffortFallbackPolicy,
        EffortLevel, ModelRoute, ResolvedDomainTemplate, RouteSlot, RoutingConfig,
        RoutingOverrides, RuntimeCapabilityProfile, RuntimeKind, SemanticAccelerationPolicy,
        SemanticAccelerationPolicyState, SemanticIndexHookAction, SlotEffortPolicy, ValueSource,
        assistant_default_model_route, resolve_effective_advanced_context_config,
        resolve_effective_domain_templates, resolve_effective_routing,
        resolve_effective_runtime_capabilities, resolve_effective_semantic_acceleration_config,
        resolve_effective_slot_effort_policies, seeded_routes_for_assistants,
    };

    #[test]
    fn assistant_default_routes_match_built_in_runtime_catalog() {
        assert_eq!(assistant_default_model_route(RuntimeKind::Codex).model, "o4-mini");
        assert_eq!(assistant_default_model_route(RuntimeKind::Copilot).model, "gpt-4.1");
        assert_eq!(assistant_default_model_route(RuntimeKind::Claude).model, "sonnet-4");
        assert_eq!(assistant_default_model_route(RuntimeKind::Gemini).model, "gemini-2.5-pro");
    }

    #[test]
    fn seeded_routes_prefer_selected_built_in_runtime_and_fallback_to_first_assistant() {
        let seeded = seeded_routes_for_assistants(&[RuntimeKind::Copilot, RuntimeKind::Claude]);
        assert_eq!(seeded.get(&RouteSlot::Planning).unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(seeded.get(&RouteSlot::Planning).unwrap().model, "gpt-4.1");
        assert_eq!(seeded.get(&RouteSlot::Implementation).unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(seeded.get(&RouteSlot::Verification).unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(seeded.get(&RouteSlot::Review).unwrap().runtime, RuntimeKind::Claude);

        let single = seeded_routes_for_assistants(&[RuntimeKind::Gemini]);
        assert!(single.values().all(|route| route.runtime == RuntimeKind::Gemini));
        assert!(single.values().all(|route| route.model == "gemini-2.5-pro"));
    }

    #[test]
    fn cli_precedence_wins_over_workspace_and_global() {
        let cli = RoutingOverrides {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Gemini,
                model: "gemini-2.5-pro".to_string(),
            }),
            ..RoutingOverrides::default()
        };
        let workspace = RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "sonnet-4".to_string(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "o4-mini".to_string(),
            }),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_routing(&cli, Some(&workspace), None, Some(&global));
        assert_eq!(resolved.planning.source, ValueSource::Cli);
        assert_eq!(resolved.planning.route.runtime, RuntimeKind::Gemini);
    }

    #[test]
    fn explicit_chat_route_prefers_workspace_over_global() {
        let workspace = RoutingConfig {
            chat: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "openai/gpt-5.4".to_string(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            chat: Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() }),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_routing(
            &RoutingOverrides::default(),
            Some(&workspace),
            None,
            Some(&global),
        );
        let chat = resolved.chat.as_ref().expect("chat route should resolve");
        assert_eq!(chat.source, ValueSource::Workspace);
        assert_eq!(chat.route.runtime, RuntimeKind::Codex);
        assert_eq!(chat.route.model, "openai/gpt-5.4");
    }

    #[test]
    fn explicit_chat_route_is_absent_when_not_configured() {
        let resolved = resolve_effective_routing(&RoutingOverrides::default(), None, None, None);

        assert!(resolved.chat.is_none());
        assert_eq!(resolved.planning.source, ValueSource::BuiltIn);
    }

    #[test]
    fn advanced_context_policy_rejects_remote_v1_settings() {
        let error = AdvancedContextConfig {
            retrieval_mode: RetrievalMode::Remote,
            remote_policy: RemoteTransmissionPolicyState::RemoteAllowed,
            budgets: RetrievalBudgets::default(),
        }
        .validate()
        .unwrap_err();

        match error {
            ConfigurationError::InvalidAdvancedContextConfig(message) => {
                assert!(message.contains("remote retrieval mode"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn semantic_index_hook_action_labels_and_validation_require_local_policy() {
        assert_eq!(SemanticIndexHookAction::Disabled.as_str(), "disabled");
        assert_eq!(SemanticIndexHookAction::MarkStale.as_str(), "mark_stale");

        SemanticAccelerationPolicy {
            policy: SemanticAccelerationPolicyState::Local,
            index_hook_action: SemanticIndexHookAction::MarkStale,
        }
        .validate()
        .unwrap();

        let error = SemanticAccelerationPolicy {
            policy: SemanticAccelerationPolicyState::Disabled,
            index_hook_action: SemanticIndexHookAction::MarkStale,
        }
        .validate()
        .unwrap_err();
        let expected = "semantic index hooks require local semantic acceleration";
        assert!(matches!(
            error,
            ConfigurationError::InvalidAdvancedContextConfig(message) if message == expected
        ));
    }

    #[test]
    fn advanced_context_policy_prefers_workspace_over_global() {
        let workspace = RoutingConfig {
            advanced_context: Some(AdvancedContextConfig {
                retrieval_mode: RetrievalMode::Disabled,
                remote_policy: RemoteTransmissionPolicyState::Blocked,
                budgets: RetrievalBudgets::default(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            advanced_context: Some(AdvancedContextConfig::default()),
            ..RoutingConfig::default()
        };

        let resolved =
            resolve_effective_advanced_context_config(Some(&workspace), None, Some(&global));

        assert_eq!(resolved.source, ValueSource::Workspace);
        assert_eq!(resolved.policy.retrieval_mode, RetrievalMode::Disabled);
        assert_eq!(resolved.policy.remote_policy, RemoteTransmissionPolicyState::Blocked);
    }

    #[test]
    fn semantic_acceleration_policy_defaults_to_disabled_and_prefers_workspace() {
        let workspace = RoutingConfig {
            semantic_acceleration: Some(SemanticAccelerationPolicy {
                policy: SemanticAccelerationPolicyState::Local,
                ..SemanticAccelerationPolicy::default()
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            semantic_acceleration: Some(SemanticAccelerationPolicy::default()),
            ..RoutingConfig::default()
        };

        let resolved =
            resolve_effective_semantic_acceleration_config(Some(&workspace), None, Some(&global));

        assert_eq!(
            SemanticAccelerationPolicy::default().policy,
            SemanticAccelerationPolicyState::Disabled
        );
        assert_eq!(resolved.source, ValueSource::Workspace);
        assert_eq!(resolved.policy.policy, SemanticAccelerationPolicyState::Local);
        assert_eq!(resolved.policy.summary_text(), "policy=local, index_hook_action=disabled");
    }

    #[test]
    fn review_role_falls_back_to_review_default() {
        let cli = RoutingOverrides::default();
        let workspace = RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "sonnet-4".to_string(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig::default();

        let resolved = resolve_effective_routing(&cli, Some(&workspace), None, Some(&global));
        assert!(resolved.reviewer_roles.is_empty());
        assert_eq!(resolved.review.route.runtime, RuntimeKind::Claude);
    }

    #[test]
    fn runtime_capability_profiles_reject_blank_notes() {
        let config = RoutingConfig {
            runtime_capabilities: BTreeMap::from([(
                RuntimeKind::Codex,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Supported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("   ".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn resolve_effective_runtime_capabilities_prefers_workspace_over_global() {
        let workspace = RoutingConfig {
            runtime_capabilities: BTreeMap::from([(
                RuntimeKind::Codex,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Supported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("workspace".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            runtime_capabilities: BTreeMap::from([(
                RuntimeKind::Codex,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Unsupported,
                    validation: CapabilityState::Unsupported,
                    handoff_target: CapabilityState::Unsupported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("global".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };

        let resolved =
            resolve_effective_runtime_capabilities(Some(&workspace), None, Some(&global));
        let profile = resolved.get(&RuntimeKind::Codex).unwrap();
        assert_eq!(profile.source, ValueSource::Workspace);
        assert_eq!(profile.profile.notes.as_deref(), Some("workspace"));
    }

    #[test]
    fn resolve_effective_slot_effort_policies_prefers_cluster_over_global() {
        let cluster = RoutingConfig {
            slot_effort_policies: BTreeMap::from([(
                RouteSlot::Planning,
                SlotEffortPolicy {
                    level: EffortLevel::High,
                    fallback: EffortFallbackPolicy::Preserve,
                    rationale: Some("cluster policy".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            slot_effort_policies: BTreeMap::from([(
                RouteSlot::Planning,
                SlotEffortPolicy {
                    level: EffortLevel::Low,
                    fallback: EffortFallbackPolicy::AllowLower,
                    rationale: Some("global policy".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_slot_effort_policies(None, Some(&cluster), Some(&global));
        let policy = resolved.get(&RouteSlot::Planning).unwrap();
        assert_eq!(policy.source, ValueSource::Cluster);
        assert_eq!(policy.policy.level, EffortLevel::High);
    }

    #[test]
    fn domain_template_settings_reject_blank_standards_text() {
        let config = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::React,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: Some("  ".to_string()),
                    external_context_bindings: Vec::new(),
                },
            )]),
            ..RoutingConfig::default()
        };

        assert!(matches!(config.validate(), Err(ConfigurationError::InvalidDomainTemplate(_))));
    }

    #[test]
    fn resolve_effective_domain_templates_layers_sources_and_bindings() {
        let global = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::React,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: Some("global react rules".to_string()),
                    external_context_bindings: vec![ExternalContextBinding {
                        kind: ExternalContextKind::DesignSystem,
                        reference: "mcp:design-system".to_string(),
                        required: false,
                        notes: Some("shared".to_string()),
                    }],
                },
            )]),
            ..RoutingConfig::default()
        };
        let workspace = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::React,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: Some("workspace react rules".to_string()),
                    external_context_bindings: vec![ExternalContextBinding {
                        kind: ExternalContextKind::DesignTokens,
                        reference: "design/tokens.json".to_string(),
                        required: true,
                        notes: None,
                    }],
                },
            )]),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_domain_templates(Some(&workspace), None, Some(&global));
        let template: &ResolvedDomainTemplate = resolved.get(&DomainFamily::React).unwrap();
        assert!(template.enabled);
        assert_eq!(template.enablement_source, ValueSource::Workspace);
        assert_eq!(template.standards_layers[0].source, ValueSource::BuiltIn);
        assert_eq!(template.standards_layers[1].source, ValueSource::Global);
        assert_eq!(template.standards_layers[2].source, ValueSource::Workspace);
        assert_eq!(template.external_context_bindings.len(), 2);
        assert_eq!(template.external_context_bindings[0].source, ValueSource::Global);
        assert_eq!(template.external_context_bindings[1].source, ValueSource::Workspace);
    }

    #[test]
    fn enum_helpers_and_summary_text_cover_all_variants() {
        assert_eq!(RuntimeKind::Claude.as_str(), "claude");
        assert_eq!(RuntimeKind::Codex.to_string(), "codex");
        assert_eq!(RuntimeKind::Copilot.as_str(), "copilot");
        assert_eq!(RuntimeKind::Gemini.to_string(), "gemini");

        assert_eq!(RouteSlot::Planning.as_str(), "planning");
        assert_eq!(RouteSlot::Implementation.as_str(), "implementation");
        assert_eq!(RouteSlot::Verification.as_str(), "verification");
        assert_eq!(RouteSlot::Review.as_str(), "review");

        assert_eq!(CapabilityState::Supported.to_string(), "supported");
        assert_eq!(CapabilityState::Unsupported.as_str(), "unsupported");
        assert!(CapabilityState::Supported.is_supported());
        assert!(!CapabilityState::Unsupported.is_supported());

        assert_eq!(EffortLevel::Low.to_string(), "low");
        assert_eq!(EffortLevel::Medium.as_str(), "medium");
        assert_eq!(EffortLevel::High.to_string(), "high");
        assert_eq!(EffortLevel::Max.as_str(), "max");

        assert_eq!(EffortFallbackPolicy::Preserve.to_string(), "preserve");
        assert_eq!(EffortFallbackPolicy::AllowLower.as_str(), "allow_lower");
        assert_eq!(DomainFamily::Systems.as_str(), "systems");
        assert_eq!(DomainFamily::React.display_name(), "React");

        let profile = RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: Some("  delegated continuation  ".to_string()),
        };
        assert!(profile.summary_text().contains("notes=delegated continuation"));

        let blank_note_profile = RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: Some("   ".to_string()),
        };
        assert!(!blank_note_profile.summary_text().contains("notes="));

        let policy = SlotEffortPolicy {
            level: EffortLevel::High,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: Some("  keep verification strict  ".to_string()),
        };
        assert!(policy.summary_text().contains("rationale=keep verification strict"));

        let blank_rationale = SlotEffortPolicy {
            level: EffortLevel::Low,
            fallback: EffortFallbackPolicy::AllowLower,
            rationale: Some("   ".to_string()),
        };
        assert_eq!(blank_rationale.summary_text(), "level=low, fallback=allow_lower");
    }

    #[test]
    fn routing_validation_and_slot_mutators_cover_invalid_policy_paths() {
        let invalid_notes = RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: Some("   ".to_string()),
        };
        assert!(matches!(
            invalid_notes.validate(),
            Err(ConfigurationError::InvalidRuntimeCapability(message))
                if message.contains("notes cannot be empty")
        ));

        let invalid_handoff = RuntimeCapabilityProfile {
            continuation: CapabilityState::Unsupported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: None,
        };
        assert!(matches!(
            invalid_handoff.validate(),
            Err(ConfigurationError::InvalidRuntimeCapability(message))
                if message.contains("handoff_target requires continuation support")
        ));

        let invalid_policy = SlotEffortPolicy {
            level: EffortLevel::Medium,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: Some(" ".to_string()),
        };
        assert!(matches!(
            invalid_policy.validate(),
            Err(ConfigurationError::InvalidSlotEffortPolicy(message))
                if message.contains("rationale cannot be empty")
        ));

        let mut routing = RoutingConfig::default();
        let planning = ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() };
        let implementation =
            ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() };
        let verification =
            ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() };
        let review =
            ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() };
        routing.set_slot(RouteSlot::Planning, planning.clone());
        routing.set_slot(RouteSlot::Implementation, implementation.clone());
        routing.set_slot(RouteSlot::Verification, verification.clone());
        routing.set_slot(RouteSlot::Review, review.clone());
        assert_eq!(routing.planning.as_ref(), Some(&planning));
        assert_eq!(routing.implementation.as_ref(), Some(&implementation));
        assert_eq!(routing.verification.as_ref(), Some(&verification));
        assert_eq!(routing.review.as_ref(), Some(&review));

        routing.unset_slot(RouteSlot::Planning);
        routing.unset_slot(RouteSlot::Implementation);
        routing.unset_slot(RouteSlot::Verification);
        routing.unset_slot(RouteSlot::Review);
        assert!(routing.planning.is_none());
        assert!(routing.implementation.is_none());
        assert!(routing.verification.is_none());
        assert!(routing.review.is_none());

        routing.reviewer_roles.insert(
            "   ".to_string(),
            ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
        );
        assert!(matches!(
            routing.validate(),
            Err(ConfigurationError::InvalidReviewerRole(message))
                if message.contains("role id cannot be empty")
        ));

        let policy = SlotEffortPolicy {
            level: EffortLevel::Low,
            fallback: EffortFallbackPolicy::AllowLower,
            rationale: None,
        };
        routing.set_slot_effort_policy(RouteSlot::Planning, policy.clone());
        assert_eq!(routing.slot_effort_policies.get(&RouteSlot::Planning), Some(&policy));
        routing.unset_slot_effort_policy(RouteSlot::Planning);
        assert!(!routing.slot_effort_policies.contains_key(&RouteSlot::Planning));

        let settings = DomainTemplateSettings {
            enabled: Some(true),
            standards: Some("keep it clean".to_string()),
            external_context_bindings: Vec::new(),
        };
        routing.set_domain_template_settings(DomainFamily::React, settings.clone());
        assert_eq!(routing.domain_templates.get(&DomainFamily::React), Some(&settings));
        routing.unset_domain_template_settings(DomainFamily::React);
        assert!(!routing.domain_templates.contains_key(&DomainFamily::React));
    }

    #[test]
    fn semantic_acceleration_policy_unset_and_global_fallback_cover_missing_paths() {
        // unset_semantic_acceleration_policy covers lines 515-517.
        let mut routing = RoutingConfig::default();
        routing.set_semantic_acceleration_policy(SemanticAccelerationPolicy {
            policy: SemanticAccelerationPolicyState::Local,
            ..SemanticAccelerationPolicy::default()
        });
        assert!(routing.semantic_acceleration.is_some());
        routing.unset_semantic_acceleration_policy();
        assert!(routing.semantic_acceleration.is_none());

        // resolve_effective_semantic_acceleration_config with only global config set
        // covers lines 878-881 (the global fallback return path).
        let global = RoutingConfig {
            semantic_acceleration: Some(SemanticAccelerationPolicy {
                policy: SemanticAccelerationPolicyState::Local,
                ..SemanticAccelerationPolicy::default()
            }),
            ..RoutingConfig::default()
        };
        let resolved = resolve_effective_semantic_acceleration_config(None, None, Some(&global));
        assert_eq!(resolved.source, ValueSource::Global);
        assert_eq!(resolved.policy.policy, SemanticAccelerationPolicyState::Local);
    }

    #[test]
    fn seeded_routes_for_assistants_returns_empty_map_when_no_assistants_given() {
        let routes = seeded_routes_for_assistants(&[]);
        assert!(routes.is_empty());
    }

    #[test]
    fn resolve_effective_domain_templates_uses_global_and_cluster_enabled_flags() {
        let global = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::React,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: Some("global react rules".to_string()),
                    external_context_bindings: Vec::new(),
                },
            )]),
            ..RoutingConfig::default()
        };
        let cluster = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::Vue,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: None,
                    external_context_bindings: Vec::new(),
                },
            )]),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_domain_templates(None, Some(&cluster), Some(&global));

        let react = resolved.get(&DomainFamily::React).unwrap();
        assert!(react.enabled);
        assert_eq!(react.enablement_source, ValueSource::Global);
        assert!(react.standards_layers.iter().any(|l| l.text.contains("global react rules")));

        let vue = resolved.get(&DomainFamily::Vue).unwrap();
        assert!(vue.enabled);
        assert_eq!(vue.enablement_source, ValueSource::Cluster);

        // Family with enabled=None in all layers → falls through to BuiltIn default (false)
        let workspace = RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::Angular,
                DomainTemplateSettings {
                    enabled: None,
                    standards: None,
                    external_context_bindings: Vec::new(),
                },
            )]),
            ..RoutingConfig::default()
        };
        let resolved2 = resolve_effective_domain_templates(Some(&workspace), None, None);
        let angular = resolved2.get(&DomainFamily::Angular).unwrap();
        assert!(!angular.enabled);
        assert_eq!(angular.enablement_source, ValueSource::BuiltIn);
    }

    #[test]
    fn effective_resolution_covers_cluster_global_and_built_in_fallbacks() {
        let cluster = RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Gemini,
                model: "gemini-2.5-pro".to_string(),
            }),
            reviewer_roles: BTreeMap::from([(
                "security".to_string(),
                ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
            )]),
            runtime_capabilities: BTreeMap::from([(
                RuntimeKind::Claude,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Unsupported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("cluster capability".to_string()),
                },
            )]),
            slot_effort_policies: BTreeMap::from([(
                RouteSlot::Planning,
                SlotEffortPolicy {
                    level: EffortLevel::High,
                    fallback: EffortFallbackPolicy::Preserve,
                    rationale: Some("cluster planning depth".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            verification: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "gpt-4o".to_string(),
            }),
            runtime_capabilities: BTreeMap::from([(
                RuntimeKind::Copilot,
                RuntimeCapabilityProfile {
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Supported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("global capability".to_string()),
                },
            )]),
            slot_effort_policies: BTreeMap::from([(
                RouteSlot::Review,
                SlotEffortPolicy {
                    level: EffortLevel::Low,
                    fallback: EffortFallbackPolicy::AllowLower,
                    rationale: Some("global review baseline".to_string()),
                },
            )]),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_routing(
            &RoutingOverrides::default(),
            None,
            Some(&cluster),
            Some(&global),
        );
        assert_eq!(resolved.planning.source, ValueSource::Cluster);
        assert_eq!(resolved.planning.route.runtime, RuntimeKind::Gemini);
        assert_eq!(resolved.verification.source, ValueSource::Global);
        assert_eq!(resolved.verification.route.runtime, RuntimeKind::Copilot);
        assert_eq!(resolved.implementation.source, ValueSource::BuiltIn);
        assert!(resolved.chat.is_none());
        assert_eq!(resolved.adjudication.source, ValueSource::BuiltIn);
        assert_eq!(resolved.reviewer_roles.get("security").unwrap().source, ValueSource::Cluster);

        let capabilities =
            resolve_effective_runtime_capabilities(None, Some(&cluster), Some(&global));
        assert_eq!(capabilities.get(&RuntimeKind::Claude).unwrap().source, ValueSource::Cluster);
        assert_eq!(capabilities.get(&RuntimeKind::Copilot).unwrap().source, ValueSource::Global);

        let effort = resolve_effective_slot_effort_policies(None, Some(&cluster), Some(&global));
        assert_eq!(effort.get(&RouteSlot::Planning).unwrap().source, ValueSource::Cluster);
        assert_eq!(effort.get(&RouteSlot::Review).unwrap().source, ValueSource::Global);
    }
}
