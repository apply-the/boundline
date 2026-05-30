//! Domain model for the workspace probe preflight surface.
//!
//! The probe provides a lightweight, read-only snapshot of workspace readiness
//! that assistant hosts can query before running the full orchestrator.

use serde::Serialize;

/// Top-level probe report aggregating all workspace readiness signals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeReport {
    pub workspace: WorkspaceState,
    pub session: SessionState,
    pub providers: ProviderState,
    pub canon: CanonState,
    pub capabilities: CapabilitiesState,
    pub recommended_next: Option<RecommendedNext>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommended_handoffs: Vec<RecommendedHandoff>,
}

/// Workspace filesystem and initialization state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceState {
    pub path: String,
    pub initialized: bool,
    pub config_present: bool,
    pub execution_profile_present: bool,
}

/// Active session state summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionState {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_summary: Option<String>,
    pub waiting_for_phase_request: bool,
}

/// Provider configuration and health summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderState {
    pub configured: bool,
    pub healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_runtime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
}

/// Canon companion availability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonState {
    pub binary_available: bool,
    pub project_memory_present: bool,
    pub guidance_present: bool,
}

/// Static and runtime feature capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilitiesState {
    pub phase_request: bool,
    pub json_stream: bool,
    pub guidance_catalog: bool,
    pub guardians: bool,
    pub canon_governance: bool,
    pub semantic_index: bool,
    pub cluster: bool,
}

/// Recommended next action for the assistant host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecommendedNext {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_command: Option<String>,
    pub reason: String,
}

/// A recommended handoff entry for the assistant host to render.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecommendedHandoff {
    pub label: String,
    pub command: String,
    pub reason: String,
}
