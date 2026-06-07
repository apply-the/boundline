#[path = "../../../src/orchestrator/capability_provider_runtime.rs"]
pub mod capability_provider_runtime;
#[path = "../../../src/orchestrator/context_intelligence.rs"]
pub mod context_intelligence;
#[path = "../../../src/orchestrator/decision_loop.rs"]
pub mod decision_loop;
#[path = "../../../src/orchestrator/engine.rs"]
pub mod engine;
#[path = "../../../src/orchestrator/flow_inference.rs"]
pub mod flow_inference;
#[path = "../../../src/orchestrator/goal_planner.rs"]
pub mod goal_planner;
#[path = "../../../src/orchestrator/governance.rs"]
pub mod governance;
#[path = "../../../src/orchestrator/guidance_catalog_runtime.rs"]
pub mod guidance_catalog_runtime;
#[path = "../../../src/orchestrator/guidance_runtime.rs"]
pub mod guidance_runtime;
#[path = "../../../src/orchestrator/planner.rs"]
pub mod planner;
#[path = "../../../src/orchestrator/recovery.rs"]
pub mod recovery;
#[path = "../../../src/orchestrator/refinement.rs"]
pub mod refinement;
#[path = "../../../src/orchestrator/review_trace.rs"]
mod review_trace;
#[path = "../../../src/orchestrator/session_runtime.rs"]
pub mod session_runtime;
#[path = "../../../src/orchestrator/session_runtime_observability.rs"]
pub mod session_runtime_observability;
#[path = "../../../src/orchestrator/terminal.rs"]
pub mod terminal;

pub mod framework_catalog;

pub use engine::{Orchestrator, OrchestratorError};
pub use framework_catalog::{
    FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, FRAMEWORK_HOOK_CATALOG, FRAMEWORK_STAGE_CATALOG,
    FrameworkHookKey, FrameworkStageKey,
};
pub use session_runtime::{SessionRuntime, SessionRuntimeError};
