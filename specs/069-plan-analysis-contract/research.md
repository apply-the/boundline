# Research: Plan Analysis Contract

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-04 as required by the
constitution:

- OpenAI still documents the `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and
  `gpt-5.4-nano` lines relevant to Boundline routing:
  [Introducing GPT-5.5](https://openai.com/index/introducing-gpt-5-5/),
  [Introducing GPT-5.4](https://openai.com/index/introducing-gpt-5-4/), and
  [Introducing GPT-5.4 mini and nano](https://openai.com/index/introducing-gpt-5-4-mini-and-nano/).
- Anthropic still documents the Claude lines already represented by the
  bundled catalog for this slice, including Claude Opus 4.7, Claude Sonnet
  4.6, and Claude Haiku 4.5:
  [Claude Opus 4.7](https://www.anthropic.com/news/claude-opus-4-7),
  [Claude Sonnet 4.6](https://www.anthropic.com/news/claude-sonnet-4-6), and
  [Claude Haiku 4.5](https://www.anthropic.com/news/claude-haiku-4-5).
- Google still documents the Gemini 2.5 and Gemini 3 lines already carried in
  the bundled catalog, including `gemini-2.5-pro`, `gemini-2.5-flash`,
  `gemini-2.5-flash-lite`, and the stable `gemini-3.5-flash` example on the
  public models page:
  [Gemini models](https://ai.google.dev/gemini-api/docs/models).

Result: no bundled model-family delta is required for this feature. The
existing `assistant/catalog/model-catalog.toml` already carries the relevant
OpenAI, Anthropic, and Google families for Boundline's supported assistant
surfaces.

## Decision 1: Reuse and expand the existing typed planning-analysis projection

**Decision**: Keep `PlanningAnalysisProjection` as the single Boundline-owned
planning-analysis domain model, but extend its finding vocabulary, source
attribution, coverage summary, and assessment logic beyond the current
backlog-only implementation.

**Rationale**: The runtime, status output, orchestration flow, and assistant
assets already know how to project `planning_analysis_state`,
`planning_analysis_findings`, and `planning_analysis_coverage`. Reusing the
existing typed surface preserves backward compatibility and avoids introducing
competing definitions of the same gate.

**Alternatives considered**:

- Introduce a second planning-analysis service module with its own persisted
  shape: rejected because it would create drift between the existing session
  projection and execution-admission logic.
- Leave planning analysis as a backlog-only helper: rejected because that
  overlaps with the backlog gate and does not satisfy the end-to-end coherence
  contract defined in the new spec.

## Decision 2: Keep planning analysis deterministic and read-only

**Decision**: Limit the initial planning-analysis slice to deterministic
  coherence checks over typed Boundline fields and governed Canon evidence
  already present in the active session.

**Rationale**: The constitution forbids hidden intelligence. A runtime gate
  that relies on opaque semantic inference would be difficult to debug,
  unstable across releases, and hard to explain through traces or assistant
  output.

**Alternatives considered**:

- Use an LLM call to compare plan, backlog, and validation prose: rejected
  because it introduces hidden heuristics, runtime cost, and nondeterminism.
- Parse every free-form planning artifact semantically: rejected because the
  first slice should consume stable typed fields and governed packet evidence,
  not invent a full document-analysis engine.

## Decision 3: Expand coherence checks across the full planning chain, not just backlog

**Decision**: Treat planning analysis as the final cross-artifact coherence
gate across the active goal, plan outcomes, validation strategy, risks,
constraints, backlog packet, execution handoff evidence, and required governed
Canon artifacts.

**Rationale**: Backlog quality answers whether the backlog packet is credible
enough on its own. Planning analysis answers whether the whole planning picture
is mutually consistent and execution-ready. The gate only makes sense if it
bridges those artifacts rather than re-checking backlog alone.

**Alternatives considered**:

- Restrict planning analysis to backlog coverage: rejected because it would
  duplicate `068-backlog-contract` instead of delivering the next roadmap
  slice.
- Fold plan quality and backlog quality into one larger gate: rejected because
  the earlier gates already provide focused, actionable defects and should
  remain independent.

## Decision 4: Treat missing Canon-owned evidence as a producer contract gap

**Decision**: When execution readiness depends on Canon-authored evidence that
is absent from the governed packet, emit an explicit producer contract gap
finding and block execution handoff.

**Rationale**: Boundline must not invent or infer Canon fields. A producer gap
is a real execution-readiness defect, and the runtime should surface it
honestly instead of lowering standards or synthesizing hidden defaults.

**Alternatives considered**:

- Guess missing Canon values from surrounding prose: rejected because the spec
  explicitly forbids invented Canon data.
- Downgrade producer gaps to warnings: rejected because execution readiness
  cannot be established when required producer-owned evidence is missing.

## Decision 5: Preserve additive compatibility and assistant-safe routing

**Decision**: Keep planning analysis additive. Older snapshots may omit the
projection entirely; newer projections preserve `blocked`, `findings`, and
`clean` semantics while assistant hosts continue to route back to planning when
the gate blocks execution.

**Rationale**: Planning analysis is already partially present across CLI and
assistant assets. The feature should complete that contract without breaking
older workspaces or teaching hosts a second recovery protocol.

**Alternatives considered**:

- Force default planning-analysis state into older snapshots: rejected because
  synthetic state would create false blocking or false readiness.
- Add a separate assistant command just for analysis repair: rejected because
  the existing plan-stage continuation path is already the right bounded
  recovery surface.

## Decision 6: Close the slice as Boundline `0.70.0`

**Decision**: Ship the completed planning-analysis contract as version
`0.70.0`, keep Canon compatibility guidance aligned to `0.67.0`, update docs
and assistant assets, and require at least 95% changed-file coverage for
touched Rust implementation files.

**Rationale**: The feature changes execution-admission behavior and the public
runtime contract. Pre-1.0 semantics allow a minor release, but the docs,
assistant assets, distribution metadata, and coverage evidence must remain
synchronized.

**Alternatives considered**:

- Defer version and doc closure to a later sweep: rejected because the runtime
  behavior would drift from the published compatibility story.
- Rely only on workspace-level test green without changed-file coverage:
  rejected because this slice completes partially-landed scaffolding and needs
  proof that the changed logic is actually exercised.
