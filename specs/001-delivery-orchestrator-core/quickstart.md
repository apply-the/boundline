# Quickstart: Delivery Orchestrator Core

## Prerequisites

1. Install Rust 1.95.0 with `rustfmt` and `clippy` available.
2. Work from the repository root on branch `001-delivery-orchestrator-core`.
3. Use a writable workspace path so trace files can be created under `.synod/traces/`.

## Current Implementation Surface

- `src/orchestrator/engine.rs` contains the synchronous bounded run loop.
- `src/orchestrator/planner.rs` provides the planner trait and `StaticPlanner` for deterministic tests.
- `src/registry/` holds the named agent and tool registries.
- `src/domain/` holds task, plan, step, limit, and trace models.
- `src/adapters/trace_store.rs` persists trace JSON files for post-run inspection.

## Minimal Usage

```rust
use serde_json::json;
use synod::{
   AgentRegistry, FileTraceStore, FnAgentAdapter, FnToolAdapter, Orchestrator, Plan,
   RunLimits, StaticPlanner, Step, StepExecutionRequest, StepExecutionResult,
   TaskRunRequest, ToolRegistry,
};

let plan = Plan::new(vec![
   Step::agent("analyze", "analyzer", json!({"phase": "analyze"}))?,
   Step::agent("code", "coder", json!({"phase": "code"}))?,
   Step::tool("verify", "tester", json!({"phase": "verify"}))?,
])?;

let planner = StaticPlanner::new(plan);
let mut agents = AgentRegistry::new();
agents.register(
   "analyzer",
   FnAgentAdapter::new(|_request: StepExecutionRequest| {
      StepExecutionResult::success(json!({"ready_for_code": true}))
   }),
)?;
agents.register(
   "coder",
   FnAgentAdapter::new(|_request: StepExecutionRequest| {
      StepExecutionResult::success(json!({"patch_applied": true}))
   }),
)?;

let mut tools = ToolRegistry::new();
tools.register(
   "tester",
   FnToolAdapter::new(|_request: StepExecutionRequest| {
      StepExecutionResult::success(json!({"tests_passed": true}))
   }),
)?;

let orchestrator = Orchestrator::new(
   planner,
   agents,
   tools,
   FileTraceStore::for_workspace("/tmp/synod-workspace"),
);

let response = orchestrator.run(TaskRunRequest {
   goal: "Fix a bounded engineering task".to_string(),
   input: json!({"ticket": "BUG-123"}),
   session_id: "session-1".to_string(),
   workspace_ref: "/tmp/synod-workspace".to_string(),
   limits: RunLimits::default(),
   initial_context: None,
})?;

assert!(response.trace_location.ends_with(".json"));
```

## Validation Commands

Run these commands from the repository root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Minimum Demo Scenarios

1. A three-step task that analyzes, changes, and verifies work, then exits successfully.
2. A task with one recoverable tool failure that retries and succeeds without losing context.
3. A task that triggers replanning after invalidating the remaining plan and then succeeds.
4. A task that exhausts its recovery budget and terminates with an inspectable failure trace.

## Exit Criteria

- The orchestrator can run a bounded multi-step task end to end.
- Later steps consume context produced by earlier steps.
- Retry and replanning remain bounded and deterministic.
- Execution traces are persisted and explain the final outcome.
