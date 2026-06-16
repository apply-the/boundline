//! Session-native orchestration engine.
//!
//! Drives the full lifecycle of a delivery session through goal capture,
//! planning, execution, governance, review, recovery, and terminal
//! finalization. The primary entry point is [`SessionRuntime`] which
//! coordinates workspace-scoped session state, while the legacy
//! [`Orchestrator`] provides a step-by-step task loop for compatibility.
//!
//! # Submodules
//!
//! | Module | Responsibility |
//! |--------|----------------|
//! | [`session_runtime`] | Workspace-scoped session orchestration facade |
//! | [`engine`] | Legacy step-by-step task loop with planner/recovery |
//! | [`decision_loop`] | Bounded observe-decide-act-verify-update loop |
//! | [`goal_planner`] | Goal-derived planning from workspace evidence |
//! | [`governance`] | Governance-decision helpers and Canon memory compaction |
//! | [`flow_inference`] | Flow inference from goal text and workspace signals |
//! | [`context_intelligence`] | Advanced-context retrieval (SQLite+FTS5) |
//! | [`capability_provider_runtime`] | Capability-provider admission and execution helpers |
//! | [`guidance_runtime`] | Capability discovery and bounded guardian execution |
//! | [`guidance_catalog_runtime`] | Directory-based guidance catalog pack discovery |
//! | [`planner`] | `Planner` trait and static test double |
//! | [`recovery`] | Retry vs. replan vs. terminate decision logic |
//! | [`terminal`] | Terminal-condition selection and status mapping |

pub mod decision_loop;
pub mod engine;
pub mod execution_orchestrator;
pub mod flow_inference;
pub mod goal_planner;
pub mod governance;
pub mod context_intelligence;
pub mod capability_provider_runtime;
pub mod guidance_catalog_runtime;
pub mod guidance_runtime;
pub mod planner;
pub mod recovery;
mod review_trace;
pub mod session_runtime;
pub mod session_runtime_observability;
pub mod terminal;

pub use engine::{Orchestrator, OrchestratorError};
pub use session_runtime::{SessionRuntime, SessionRuntimeError};
