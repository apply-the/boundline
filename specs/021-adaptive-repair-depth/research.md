# Research: Adaptive Repair Depth

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## R1: Keep adaptive repair on the explicit compatibility path for 0.21.0

**Decision**: Deepen adaptive execution on the existing manifest-backed compatibility route instead of moving adaptive control onto the primary session-native or workflow-owned path in this slice.

**Rationale**: The roadmap asks for stronger adaptive heuristics, but the smallest independently valuable slice is to improve bounded candidate selection where adaptive already exists. Moving adaptive ownership into the session-native runtime would be a broader architectural step.

**Alternatives Considered**:
- Promote adaptive execution into the primary session-native route immediately: rejected because it would expand the slice far beyond adaptive repair depth and reopen route ownership questions.
- Add workflow-owned adaptive execution: rejected because workflows remain a bounded projection layer, not a second orchestration engine.

## R2: Derive adaptive guidance from the existing validation record and failure message

**Decision**: Build validation-guided adaptive heuristics from the latest persisted `ValidationRecord` plus the current step failure message already available in task context and execution results.

**Rationale**: The repository already persists validation stdout, stderr, exit code, and success state into task context. Reusing those bounded artifacts keeps the heuristics inspectable and avoids introducing new external analyzers or opaque state.

**Alternatives Considered**:
- Parse arbitrary repository state beyond the current task context: rejected because it weakens boundedness and makes decisions harder to explain.
- Introduce a separate failure-analysis subsystem: rejected because the existing validation record already contains enough local evidence for a first slice.

## R3: Improve candidate ranking and slice selection before adding broader mutation generators

**Decision**: Use validation guidance to re-rank bounded workspace targets and existing local repair candidates before introducing additional open-ended change generators.

**Rationale**: The current adaptive path already has bounded candidate synthesis, signature tracking, and replanning. A better ranking layer produces immediate delivery value while preserving current safety limits and existing fixture machinery.

**Alternatives Considered**:
- Add several new mutation families first: rejected because new generators without better guidance could still exhaust in the wrong file or wrong local context.
- Keep deterministic ordering and only improve docs: rejected because the roadmap explicitly calls for stronger adaptive heuristics.

## R4: Persist validation-guided selection reasons as first-class adaptive evidence

**Decision**: Record the validation-guided rationale inside adaptive selection evidence, selection headlines, and attempt lineage so CLI and trace surfaces can explain why a bounded candidate changed.

**Rationale**: A smarter adaptive decision that is not visible to operators would violate the constitution's explicit-intelligence and observability rules. The smallest compliant slice must improve inspectability together with repair quality.

**Alternatives Considered**:
- Keep guidance internal to the candidate scorer: rejected because developers could not distinguish a real heuristic change from hidden magic.
- Only surface the final changed file: rejected because that omits why a new attempt became credible.

## R5: Close the slice as 0.21.0 with release-aligned docs, coverage, clippy, and formatting

**Decision**: Reserve a version bump to `0.21.0` and include docs, changelog, coverage refresh for modified Rust files, clippy cleanup, and `cargo fmt` in the implementation tasks.

**Rationale**: This repository treats each roadmap slice as a versioned delivery unit, and the user explicitly requested release hygiene plus validation closeout as part of the slice.

**Alternatives Considered**:
- Defer docs and release hygiene until after implementation: rejected because the operator story changes with this slice.
- Limit validation to targeted tests only: rejected because the slice changes bounded execution behavior and should finish with repository-standard gates.