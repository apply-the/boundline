//! Pure domain model: value objects, aggregates, and deterministic algorithms.
//!
//! This module contains no I/O; all types are serializable and all functions
//! are side-effect-free. The domain layer defines the vocabulary used by
//! adapters, orchestrator, and CLI.
//!
//! # Key submodules
//!
//! | Module | Responsibility |
//! |--------|----------------|
//! | [`session`] | `ActiveSessionRecord`, `SessionStatus`, storage paths |
//! | [`task`] | `Task`, `TaskStatus`, run requests/responses |
//! | [`step`] | `Step`, `StepKind`, execution requests/results |
//! | [`goal_plan`] | `GoalPlan`, context packs, planning projections |
//! | [`governance`] | Canon modes, governed stages, lifecycle states |
//! | [`configuration`] | Routing, runtime kinds, effort policies |
//! | [`trace`] | `ExecutionTrace`, `TraceEvent`, event types |
//! | [`audit`] | Audit entries, actors, algorithms, outcomes |
//! | [`flow`] | Built-in flow definitions and step metadata |
//! | [`framework_adapter`] | External adapter selection, capability, and trace vocabulary |
//! | [`review`] | Council profiles, vote rules, stop semantics |
//! | [`reasoning`] | Reasoning profiles, debate, confidence levels |
//! | [`cluster`] | Multi-workspace clusters and delivery stories |
//! | [`brief`] | Authored brief normalization and ingestion |
//! | [`project_memory`] | Governed evidence and contribution summaries |
//! | [`context_intelligence`] | Retrieval candidates and semantic states |
//! | [`guidance`] | Guidance/guardian capabilities and findings |
//! | [`limits`] | Run budgets and terminal conditions |

pub mod audit;
pub mod auth_profile;
pub mod brief;
pub mod checkpoint;
pub mod cluster;
pub mod configuration;
pub mod context_intelligence;
pub mod decision;
pub mod distribution;
pub mod domain_templates;
pub mod execution;
pub mod flow;
pub mod flow_policy;
pub mod follow_through;
pub mod framework_adapter;
pub mod goal_plan;
pub mod governance;
pub mod guidance;
pub mod guidance_catalog;
pub mod limits;
pub mod negotiation;
pub mod plan;
pub mod probe;
pub mod project_index;
pub mod project_memory;
pub mod review;
pub mod routing_decision;
pub mod session;
pub mod stage_council;
pub mod step;
pub mod task;
pub mod task_context;
pub mod tool_result;
pub mod trace;
pub mod workflow;
pub mod workspace_hygiene;
