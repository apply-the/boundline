# Research: Plan Quality Contract

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-02 as required by the
constitution:

- GitHub Copilot's supported-model reference lists the currently surfaced
  OpenAI, Anthropic, and Google families relevant to the bundled Copilot
  runtime, including GPT-5.5, GPT-5.4 variants, Claude Opus 4.8, Claude Sonnet
  4.6, Claude Haiku 4.5, Gemini 3.5 Flash, Gemini 3.1 Pro, Gemini 3 Flash, and
  Gemini 2.5 Pro:
  <https://docs.github.com/en/copilot/reference/ai-models/supported-models>
- GitHub Copilot's model-comparison reference also describes Claude Opus 4.8,
  Claude Sonnet 4.6, Gemini 3.5 Flash, Gemini 3.1 Pro, Gemini 3 Flash, and
  Gemini 2.5 Pro:
  <https://docs.github.com/en/copilot/reference/ai-models/model-comparison>
- OpenAI's public model reference remains the source for current OpenAI API
  route availability:
  <https://platform.openai.com/docs/models>
- Google's public Gemini model reference remains the source for current Gemini
  API route availability:
  <https://ai.google.dev/gemini-api/docs/models>

The bundled catalog already carries the relevant families for the supported
runtime surfaces, so this feature does not add a new model family. A local
hygiene issue was found: `assistant/catalog/model-catalog.toml` repeats the
Copilot `opus-4.8` entry. Implementation must remove the duplicate and refresh
the catalog metadata date while preserving the supported family set.

## Decision 1: Formalize the existing typed assessment instead of adding a second validator

**Decision**: Reuse `GoalPlan::assess_plan_quality()` and the typed
`PlanQualityAssessment` projection already owned by `src/domain/goal_plan.rs`.
Audit and complete the current behavior against the spec rather than layering a
new quality service over it.

**Rationale**: The current domain model already exposes the required additive
state, findings, assumptions, and serde defaults. A parallel validator would
create ordering drift between persisted session state, status output, and run
admission without delivering additional operator value.

**Alternatives considered**:

- Add a separate plan-quality module: rejected because the existing domain
  owner is already narrow and typed.
- Evaluate quality only in CLI presentation: rejected because execution
  admission and assistant projections must share the same runtime decision.
- Defer validation to Canon: rejected because Boundline owns execution
  admission and must remain independently testable.

## Decision 2: Keep one deterministic gate order

**Decision**: Evaluate goal quality first, plan quality second, backlog quality
third, and planning analysis fourth. Plan-quality recovery emits exactly one
`phase_request` for the highest-impact current finding.

**Rationale**: Operators need one actionable next step. Reporting later
cross-artifact findings while the plan still lacks its own validation strategy
would increase noise and make recovery order ambiguous.

**Alternatives considered**:

- Emit all possible questions at once: rejected because it breaks the
  sequential one-question contract.
- Evaluate backlog or analysis first: rejected because those checks depend on a
  credible plan.
- Silently infer validation strategy: rejected because it would hide a
  delivery-critical decision.

## Decision 3: Preserve additive persisted state and compatibility defaults

**Decision**: Keep `plan_quality` as an additive serde-backed field in the
persisted `GoalPlan`, default older snapshots to a ready empty assessment at
deserialization time, and recompute the effective assessment from the current
plan before presentation or admission decisions.

**Rationale**: Existing workspaces must remain readable after release `0.67.0`.
Recomputation prevents stale persisted projections from overriding current
plan contents while the additive default preserves backward compatibility.

**Alternatives considered**:

- Make the new field mandatory during deserialization: rejected because it
  would break existing `.boundline/session.json` files.
- Trust only the persisted projection: rejected because edited or migrated
  plan content could leave stale readiness state behind.
- Avoid persistence entirely: rejected because status and trace history need an
  inspectable state transition.

## Decision 4: Reuse the existing phase-request and assistant routing boundary

**Decision**: Keep `phase_request`, `assistant_resume_command`, and
`assistant_next_command` as the only recovery and continuation contract.
Supported assistant planning assets must preserve the runtime fields and stop
on blocked or clarification-required quality.

**Rationale**: Boundline already has one host-safe sequential handoff protocol.
Extending that protocol keeps the CLI authoritative across Copilot, Claude,
Codex, and Antigravity without adding host-specific control flow.

**Alternatives considered**:

- Add a plan-quality-specific assistant command: rejected because no new CLI or
  assistant command is needed.
- Let hosts infer recovery from prose: rejected because it would reintroduce
  chat-only behavior and inconsistent continuation.
- Add Speckit hooks to the Boundline assistant command: rejected because
  Boundline uses runtime-owned handoffs.

## Decision 5: Ship release closure as part of the slice

**Decision**: Close the implementation as version `0.67.0`, align release
metadata and package manifests, update user and engineering docs, remove the
catalog duplicate, run formatting and clippy, and prove at least 95% patch
coverage for changed or created implementation files.

**Rationale**: The user-visible gate changes planning admission behavior and
must ship as one documented, verifiable pre-1.0 minor release. Patch coverage
is the appropriate metric because this feature completes scaffolding already
present in the codebase and should measure newly changed behavior directly.

**Alternatives considered**:

- Leave release metadata for a later sweep: rejected because package and docs
  drift would make the active behavior ambiguous.
- Require only full-workspace coverage: rejected because it obscures whether
  newly changed lines are exercised.
- Skip catalog hygiene because no family changed: rejected because duplicate
  choices degrade operator setup even when the family set is current.
