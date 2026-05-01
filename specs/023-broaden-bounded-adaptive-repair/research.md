# Research: Broaden Bounded Adaptive Repair

**Feature**: 023-broaden-bounded-adaptive-repair  
**Date**: 2026-05-01

## R1: Keep deeper adaptive repair on the explicit compatibility path for 0.23.0

**Decision**: Broaden adaptive mutation depth on the existing manifest-backed compatibility route instead of moving adaptive control onto the primary session-native or workflow-owned path in this slice.

**Rationale**: The roadmap now prioritizes stronger bounded adaptive behavior, but the continuity and routing work from `0.22.0` should remain stable. The smallest valuable next step is to make the existing compatibility path more capable without reopening orchestration ownership.

**Alternatives Considered**:
- Promote adaptive execution into the primary session-native route immediately: rejected because it would expand the slice far beyond bounded adaptive repair and reopen route-ownership questions.
- Add workflow-owned adaptive execution: rejected because workflows remain a bounded projection layer, not a second adaptive engine.

## R2: Add richer deterministic mutation families before introducing open-ended synthesis

**Decision**: Extend the built-in adaptive mutation vocabulary with additional deterministic, local, file-bounded change families instead of introducing model-generated or repository-wide mutation search.

**Rationale**: The current adaptive path is too narrow for representative non-arithmetic failures, but bounded local generators still preserve inspectability, stable signatures, and deterministic coverage.

**Alternatives Considered**:
- Add open-ended LLM-generated code edits: rejected because that would violate boundedness and make candidate provenance harder to explain.
- Improve path scoring only, without new mutation families: rejected because the current three change kinds still leave too many real failures unrepairable even when the correct file is selected.

## R3: Treat candidate credibility and rejection as first-class adaptive evidence

**Decision**: Persist and project explicit candidate credibility, rejection, and exhaustion reasons alongside selection headlines and attempt lineage.

**Rationale**: Broader mutation families increase the number of plausible bounded candidates. Without explicit credibility and rejection evidence, the runtime would become less understandable just as it becomes more adaptive.

**Alternatives Considered**:
- Keep credibility internal to candidate ranking: rejected because developers could not distinguish a real heuristic improvement from hidden magic.
- Surface only the selected candidate: rejected because it would hide why other bounded candidates were rejected or why the run exhausted.

## R4: Reuse validation guidance and signature history as the bounded decision inputs

**Decision**: Keep candidate ranking driven by the latest validation guidance, workspace-slice scoring, and prior candidate signatures already persisted in task context and traces.

**Rationale**: The repository already persists validation stdout, stderr, exit code, matched terms, slice headlines, and attempt lineage. Reusing those surfaces keeps the adaptive decision path explicit and avoids inventing a new analysis subsystem.

**Alternatives Considered**:
- Parse arbitrary repository state beyond configured `read_targets`: rejected because it weakens boundedness.
- Introduce a separate adaptive memory or cache file: rejected because existing task context, session projection, and trace state already carry the required evidence.

## R5: Keep exhaustion explicit instead of faking additional retries

**Decision**: When no remaining bounded candidate is credible or allowed, adaptive runs should stop in an explicit failed or exhausted terminal state with surfaced rejection rationale instead of inventing another fallback attempt.

**Rationale**: The constitution treats failure handling and explicit intelligence as core behavior. If deeper adaptive repair becomes broader, exhaustion semantics must become clearer too.

**Alternatives Considered**:
- Retry a weak candidate just to avoid exhaustion: rejected because it weakens credibility and duplicates already failed paths.
- Hide exhaustion behind generic validation failure text: rejected because the developer loses the key reason why bounded recovery ended.

## R6: Close the slice as 0.23.0 with release-aligned docs, coverage, clippy, and formatting

**Decision**: Reserve a version bump to `0.23.0` and include docs, assistant guidance, changelog, coverage refresh for modified Rust files, clippy cleanup, and `cargo fmt` as explicit implementation tasks.

**Rationale**: The user requested full release closeout, and the adaptive operator story changes materially in this slice.

**Alternatives Considered**:
- Defer docs and release hygiene until after implementation lands: rejected because maintainers and assistants need the updated bounded-adaptive story as part of the slice itself.
- Limit validation to a few targeted tests only: rejected because this slice changes cross-cutting runtime and read-side behavior and should close with repository-standard gates.