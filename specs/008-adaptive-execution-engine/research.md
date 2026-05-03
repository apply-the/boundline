# Research: Adaptive Execution Engine

## Decision 1: Extend the existing execution profile with an adaptive mode

- Decision: Add an optional adaptive execution mode to `<workspace>/.boundline/execution.json` instead of creating a second manifest or replacing the current attempt-based profile outright.
- Rationale: Spec 008 must broaden execution beyond fixed pre-authored attempts without breaking the working session, flow, trace, and review surfaces already established in Specs 006 and 007. Extending the existing profile keeps one delivery contract per workspace and allows legacy manifest behavior to remain valid.
- Alternatives considered:
  - Introduce a separate `.boundline/adaptive.json` manifest: rejected because it would split one bounded delivery loop across multiple configuration contracts.
  - Replace the current attempt-based profile entirely: rejected because it would make 006-style deterministic scenarios and legacy fixture conversion harder to preserve.

## Decision 2: Use a dedicated adaptive planner instead of precomputing all replans

- Decision: Add a planner path that synthesizes the initial adaptive attempt and chooses the next attempt only after validation failure or explicit replanning conditions.
- Rationale: The current `StaticPlanner` expects a full queue of replacement plans upfront. Spec 008 needs later attempts to depend on current workspace state, tried candidate history, and latest validation evidence. A dedicated planner reuses the orchestrator interface without introducing a second execution engine.
- Alternatives considered:
  - Precompute all adaptive attempts during startup: rejected because later attempts would not react to actual failure evidence.
  - Hide adaptive selection inside one code step: rejected because it would violate the constitution requirement for explicit planning and inspectable recovery behavior.

## Decision 3: Select one bounded workspace slice through deterministic scoring over read targets

- Decision: Score the configured `read_targets` using bounded evidence from the delivery goal, validation output, and path preferences, then choose one highest-scoring slice per adaptive attempt.
- Rationale: The feature needs real slice selection without hidden repository-wide exploration. Restricting selection to configured `read_targets` and scoring them deterministically keeps scope bounded, reproducible, and inspectable.
- Alternatives considered:
  - Search the entire repository every time: rejected because it expands scope beyond the declared workspace slice and weakens bounded execution.
  - Require the user to identify the exact file before every run: rejected because it would not broaden the execution engine beyond fixed pre-authored behavior.

## Decision 4: Generate candidate changes from deterministic built-in repair heuristics

- Decision: The initial adaptive slice generates candidate attempts from a small deterministic set of repair heuristics derived from the current file content, such as arithmetic operator swaps, comparison flips, and boolean literal flips.
- Rationale: Spec 008 needs a real adaptive change path, but a full generative code engine would be too broad for one bounded slice. Deterministic heuristics are reproducible, testable, and sufficient to demonstrate adaptive planning against simple real workspace failures.
- Alternatives considered:
  - Call an external model or provider at runtime: rejected because it introduces provider coupling and hidden intelligence before the planner and evidence model are ready.
  - Keep all candidate changes authored in the manifest: rejected because that does not move beyond fixed pre-authored attempts.

## Decision 5: Prevent repeated attempts through candidate signatures and lineage tracking

- Decision: Persist a stable signature for each adaptive candidate along with attempt lineage so the planner can reject materially identical retries unless new evidence justifies them.
- Rationale: The spec requires that adaptive execution stop explicitly instead of repeating the same failed path indefinitely. Signature and lineage tracking provide a simple, inspectable way to enforce that rule.
- Alternatives considered:
  - Compare only attempt IDs: rejected because adaptive candidates are synthesized at runtime and need content-based identity.
  - Allow retries until step limits alone stop the run: rejected because that permits non-credible repetition.

## Decision 6: Reuse existing review and CLI surfaces by synthesizing adaptive attempt metadata

- Decision: Adaptive attempts will emit the same core metadata surfaces already used by attempt-based execution: `attempt_id`, changed files, validation record, trace events, and session projections, plus new slice-selection and lineage fields.
- Rationale: Specs 006 and 007 already project delivery and review evidence into `run`, `status`, `next`, and `inspect`. Reusing those surfaces keeps the new slice inspectable without adding a second user-facing runtime model.
- Alternatives considered:
  - Create adaptive-only CLI commands or trace files: rejected because it would fragment the developer workflow.
  - Hide selection evidence inside raw logs only: rejected because it would make the adaptive behavior harder to inspect and reason about.
