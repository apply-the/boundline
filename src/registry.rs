//! In-memory registries for dynamically-registered agent and tool adapters.
//!
//! Provides name-keyed lookup tables used by the orchestrator to dispatch
//! step execution to the appropriate [`AgentAdapter`](crate::adapters::agent::AgentAdapter)
//! or [`ToolAdapter`](crate::adapters::tool::ToolAdapter) at runtime.

pub mod agent_registry;
pub mod tool_registry;
