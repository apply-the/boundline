//! Infrastructure adapters implementing I/O contracts.
//!
//! Each adapter encapsulates a single persistence or integration concern:
//! file-backed stores for session/trace/config/checkpoint state, provider
//! HTTP runtimes, governance CLI wrappers, and environment detection.
//!
//! # Submodules
//!
//! | Module | Responsibility |
//! |--------|----------------|
//! | [`session_store`] | `SessionStore` trait; workspace-local JSON persistence |
//! | [`trace_store`] | `TraceStore` trait; execution trace files under `.boundline/traces/` |
//! | [`config_store`] | Workspace and global `config.toml` / `.env` access |
//! | [`cluster_store`] | Multi-workspace `cluster.toml` persistence |
//! | [`checkpoint_store`] | Checkpoint capture, restore, and manifest bookkeeping |
//! | [`audit_store`] | Append-only NDJSON audit event log |
//! | [`auth_profile_store`] | Provider credential storage |
//! | [`governance_runtime`] | Canon CLI and local governance runtime adapters |
//! | [`provider_runtime`] | HTTP adapters for OpenAI, Anthropic, Copilot, etc. |
//! | [`capability_provider_runtime`] | Command/HTTP capability-provider transport helpers |
//! | [`agent`] | Step-agent adapters plus framework-adapter subprocess hosts |
//! | [`tool`] | `ToolAdapter` trait and function-based adapter |
//! | [`env_layer`] | Environment variable constants and availability checks |
//! | [`github_device_flow`] | GitHub OAuth device-flow token exchange |

pub mod audit_store;
pub mod agent;
pub mod auth_profile_store;
pub mod capability_provider_runtime;
pub mod checkpoint_store;
pub mod cluster_store;
pub mod config_store;
pub mod env_layer;
pub mod github_device_flow;
pub mod governance_runtime;
pub mod provider_runtime;
pub mod session_store;
pub mod tool;
pub mod trace_store;

/// Single process-wide mutex serialising all tests that mutate environment
/// variables. Shared across `env_layer` and `provider_runtime` test modules so
/// they cannot race on `OPENAI_API_KEY`, `XDG_CONFIG_HOME`, etc.
#[cfg(test)]
pub(crate) static SHARED_ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
    std::sync::OnceLock::new();
