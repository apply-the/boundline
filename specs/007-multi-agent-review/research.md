# Research: Multi-Agent Review & Voting

## Decision 1: Extend the existing execution manifest with review configuration

- Decision: Add bounded review configuration under `<workspace>/.boundline/execution.json` instead of introducing a separate review manifest.
- Rationale: Review is a post-execution quality-control step over the same bounded delivery run. Reusing the execution manifest keeps delivery and review coupled, avoids duplicate workspace-local config files, and preserves the same local-fixture test story used by the execution engine slice.
- Alternatives considered:
  - Create a new `.boundline/review.json` manifest: rejected because it would split one delivery lifecycle across two workspace contracts.
  - Hard-code councils in the CLI: rejected because it would make review behavior less testable and less inspectable.

## Decision 2: Model councils as sequential reviewer steps, not parallel fan-out

- Decision: Represent a review council as a bounded ordered set of reviewer steps executed sequentially through the existing runtime.
- Rationale: The constitution still requires sequential-first execution. Sequential reviewer steps preserve explicit traceability, make limits easy to enforce, and avoid introducing concurrency or hidden background work.
- Alternatives considered:
  - Parallel reviewer execution: rejected because it would introduce fan-out and coordination complexity before the runtime is ready for concurrency.
  - Single synthetic reviewer with aggregated output: rejected because it would not provide true multi-reviewer evidence.

## Decision 3: Persist review evidence as structured findings, vote summaries, and adjudication outcomes

- Decision: Store reviewer findings, vote resolution, and adjudication output as structured task state and trace payloads, then project the latest summary into run, status, next, and inspect surfaces.
- Rationale: Review quality control only adds value if a developer can inspect it later. The current session and trace model already supports structured state patches, so review evidence can reuse the same persistence surfaces as execution evidence.
- Alternatives considered:
  - Render review output only as CLI text: rejected because status and inspect need stable structured evidence.
  - Store review evidence in a separate file tree: rejected because it would fragment one delivery decision across multiple stores.

## Decision 4: Support majority and weighted voting with one optional adjudication step

- Decision: The initial slice supports majority and weighted vote rules, plus one bounded adjudication step when the first vote does not yield a credible terminal decision.
- Rationale: This matches the roadmap intent while keeping the capability minimal. One adjudication step is enough to demonstrate disagreement handling without turning the feature into open-ended debate simulation.
- Alternatives considered:
  - Majority voting only: rejected because the roadmap explicitly calls for weighted voting.
  - Multi-round adjudication or debate trees: rejected because they would violate the minimal bounded slice.

## Decision 5: Keep the user-facing review surface provider-agnostic while retaining reviewer source metadata

- Decision: Reviewer definitions may record source or provider labels for traceability, but status and inspect center on reviewer role, findings, vote rule, and final decision rather than provider-specific workflow.
- Rationale: The feature needs provider diversity without turning Boundline into a provider-routing framework. Provider labels remain useful evidence while the CLI stays focused on delivery decisions.
- Alternatives considered:
  - Hide provider/source metadata entirely: rejected because developers may need to understand which review source participated.
  - Expose provider-specific routing and selection controls as the main UX: rejected because it expands scope into provider abstraction complexity.

## Decision 6: Add dedicated user-facing voting documentation

- Decision: Ship a dedicated document explaining review triggers, finding severities, majority voting, weighted voting, and adjudication behavior.
- Rationale: Voting changes the meaning of a terminal delivery result. A dedicated document lowers ambiguity for developers and assistant command packs without overloading the README.
- Alternatives considered:
  - Document voting only inside code comments or tests: rejected because developers need a direct operational reference.
  - Put all voting details only in the feature spec: rejected because the spec is not the primary usage document.
