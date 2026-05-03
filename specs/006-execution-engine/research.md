# Research: Execution Engine (Code Delivery)

## Decision 1: Introduce a workspace execution profile with legacy fixture fallback

- Decision: Load execution configuration from `.boundline/execution.json` and fall back to the existing `.boundline/fixture.json` manifest by converting the legacy shape into the new execution-profile model.
- Rationale: Spec 006 needs a more honest execution surface than a hard-coded red-to-green fixture, but the repository already has working fixture tests and session flows. A new execution profile provides a forward-looking contract without breaking existing workspaces or tests on day one.
- Alternatives considered:
  - Keep only `.boundline/fixture.json`: rejected because it preserves the old feature framing and makes it harder to express richer delivery evidence and multiple change attempts.
  - Replace the fixture manifest with a breaking format change: rejected because it would add migration risk while the feature is still establishing the new execution model.

## Decision 2: Reuse the existing orchestrator loop and registry interfaces

- Decision: Keep the current `Orchestrator`, `Planner`, `AgentAdapter`, and `ToolAdapter` abstractions and plug the execution engine in as a new concrete planner plus workspace-aware agent and tool handlers.
- Rationale: The current loop already enforces sequential execution, explicit terminal conditions, retries, replans, and persisted traces. Spec 006 needs real delivery work, not a second orchestration engine.
- Alternatives considered:
  - Build a separate execution runtime beside the orchestrator: rejected because it would duplicate retry, replan, and trace behavior that already exists.
  - Push all logic into CLI commands: rejected because it would bypass the shared task model and make session-based execution harder to inspect.

## Decision 3: Persist change evidence as structured task state and trace payloads

- Decision: Record changed files, before-or-after snippets, and validation outcomes as structured output from execution steps, then merge the relevant fields into task context and trace payloads.
- Rationale: Status, inspect, and post-run summaries all need stable evidence surfaces. The current task context already supports state patches and nested step outputs, so structured execution evidence can flow through existing persistence without adding another store.
- Alternatives considered:
  - Persist diffs only inside trace text output: rejected because status and next guidance also need access to the latest delivery evidence.
  - Add a separate change-evidence file store: rejected because it would split the source of truth for one bounded execution run.

## Decision 4: Model retries and replans as bounded attempt changes, not hidden loops

- Decision: Treat each delivery attempt as an explicit change-set plus validation pair. Validation failures can request retry or replan according to the profile and the existing task limits, and every retry or replan remains visible in traces.
- Rationale: Spec 006 requires generate -> run -> fix -> retry behavior, but the constitution also requires bounded execution and no hidden intelligence. Attempt-based replanning fits the current recovery model cleanly.
- Alternatives considered:
  - Retry the validation command without a new change attempt: rejected because it does not move delivery forward unless the failure was genuinely transient.
  - Hide multiple repair attempts inside one code step: rejected because it would conceal important recovery behavior from traces and coverage.

## Decision 5: Enforce the 90%-per-file coverage goal through repository-native coverage generation

- Decision: Use the existing `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` workflow and expand unit, contract, and integration coverage until every Rust source file in `src/` clears 90% line coverage.
- Rationale: The repository already publishes `lcov.info` in CI, so the same command is the right source of truth for the requested coverage gate.
- Alternatives considered:
  - Rely only on `cargo test`: rejected because passing tests do not prove the per-file coverage target.
  - Introduce a second coverage tool: rejected because the repo already standardizes on `cargo-llvm-cov` in CI.