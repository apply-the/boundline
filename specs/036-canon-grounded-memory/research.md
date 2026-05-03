# Research: Canon-Grounded Reasoning And Structured Memory

## Decision 1: Consume Canon 0.39.0 capabilities and mode summaries as bounded reasoning signals

- **Decision**: Extend Synod's Canon-facing reasoning inputs to include the
  stable `canon governance capabilities --json` surface plus the current
  governance run and status summaries, not just packet existence or stage-end
  readiness.
- **Rationale**: Canon 0.39.0 exposes supported modes, status values, packet
  readiness vocabulary, compatibility notes, and mode-result summaries such as
  `artifact_packet_summary`, `execution_posture`, and primary-artifact actions.
  Those fields provide bounded evidence about what Canon can support and what a
  governed packet already says, which is exactly the signal Synod needs during
  planning and later decision selection.
- **Alternatives considered**:
  - Keep relying only on Synod's static `supported_canon_modes_for_stage` table.
    Rejected because it captures stage policy but not runtime compatibility or
    artifact-summary signals.
  - Re-read raw `.canon/artifacts/**` files on every plan or decision. Rejected
    because it would make Canon grounding heavier, less bounded, and harder to
    summarize credibly across loops.

## Decision 2: Persist compact Canon-grounded memory inside existing task and session state

- **Decision**: Store the durable compact Canon-grounded memory in existing
  session-owned state, centered on `TaskContext` and projected through the
  current session, goal-plan, and trace read-side surfaces.
- **Rationale**: `TaskContext` already owns durable state for latest governance
  stage, packet, reuse binding, and autopilot decisions. Extending that same
  stateful surface preserves sequential execution, keeps the new memory in the
  authoritative runtime story, and avoids introducing a second memory store.
- **Alternatives considered**:
  - Add a new standalone memory file under `.synod/`. Rejected because it would
    split authority away from the active session and task context.
  - Store compact memory only in traces. Rejected because later planning and
    decision steps need authoritative state before a new trace event exists.

## Decision 3: Ground planning context assembly on Canon context snapshots, not raw governance flags

- **Decision**: Build a normalized Canon context snapshot that captures packet
  lineage, artifact summaries, capability signals, and bounded compatibility
  notes, then feed that snapshot into planning context assembly and plan
  rationale.
- **Rationale**: Planning currently sees `.canon` mostly as presence plus file
  refs. A Canon context snapshot gives the planner bounded, reusable meaning: it
  can shape verification strategy, narrow target selection, and explain why a
  proposal should stop or continue without forcing the planner to understand the
  entire Canon artifact tree directly.
- **Alternatives considered**:
  - Append more raw Canon file paths into the context pack. Rejected because raw
    file refs alone do not expose capability limits, packet reuse meaning, or
    summary headlines.
  - Use only a human-readable planning note without structured snapshot fields.
    Rejected because later loops need machine-usable credibility and lineage.

## Decision 4: Let later decisions reuse compact Canon memory when it remains credible

- **Decision**: Extend decision selection and evidence attribution so later loop
  iterations can reuse compact Canon-grounded memory when that memory is still
  credible, instead of re-reading full Canon state for every step.
- **Rationale**: Spec 036 is not just about better planning. The decision loop
  must be able to say, for example, that a verification-focused next action is
  justified by carried-forward Canon packet constraints or capability posture.
  Reusing compact memory keeps that influence explicit and bounded.
- **Alternatives considered**:
  - Limit Canon grounding to the planning step. Rejected because later decisions
    would immediately lose the new reasoning input.
  - Add opaque heuristics inside selector choice. Rejected because the
    constitution requires visible decisions and explicit intelligence.

## Decision 5: Treat stale or contradictory Canon memory as an explicit bounded stop

- **Decision**: Model compact Canon memory with an explicit credibility state
  such as credible, stale, contradicted, or insufficient, and require the
  runtime to refresh, replan, or stop explicitly when the memory is no longer
  trustworthy.
- **Rationale**: Compaction is only safe when the runtime can say when it has
  compressed too far or when upstream Canon facts changed. Explicit credibility
  preserves operator trust and prevents Synod from silently reusing outdated
  governed assumptions.
- **Alternatives considered**:
  - Always refresh Canon state before every later decision. Rejected because it
    defeats the point of durable compact memory and adds repeated runtime cost.
  - Ignore staleness until a hard failure occurs. Rejected because that would
    make failure handling less inspectable and more expensive.

## Decision 6: Keep Canon grounding advisory to Synod's authority model, not a replacement for it

- **Decision**: Canon-grounded evidence may change planning and decision
  selection, but Synod's session-native runtime remains the authority that owns
  bounded start conditions, stop conditions, routing, and state projection.
- **Rationale**: The constitution forbids making external systems the hidden
  owner of Synod control flow. The correct slice is to let Canon materially
  influence bounded reasoning while Synod still decides when to continue,
  refresh, replan, or stop.
- **Alternatives considered**:
  - Delegate native routing authority to Canon when governed evidence exists.
    Rejected because it would violate external-separation rules and blur the
    product boundary.
  - Ignore explicit compatibility continuity. Rejected because compatibility
    traces remain a real authority surface in the existing product.

## Decision 7: Close the feature as a release-aligned 0.36.0 macrofeature

- **Decision**: Treat `0.36.0` closeout as part of the feature, including the
  Synod version bump, Canon compatibility narrative, docs and assistant-pack
  updates, roadmap closure, and coverage above 95% for modified Rust files.
- **Rationale**: Canon-grounded reasoning changes operator-visible behavior and
  must ship as one coherent product story rather than as hidden internal wiring.
- **Alternatives considered**:
  - Defer docs and release artifacts until a later cleanup. Rejected because the
    user explicitly requested release-complete delivery and the behavior is
    externally visible.