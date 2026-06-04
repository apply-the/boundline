# Research: Backlog Contract

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-04:

- GitHub Copilot still documents the OpenAI, Anthropic, and Google model
  families already carried in `assistant/catalog/model-catalog.toml`.
- OpenAI still documents `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and
  `gpt-5.4-nano` as active coding-relevant models.
- Google still documents the Gemini 2.5 and Gemini 3 preview lines already
  represented by the bundled catalog.

Result: no model-family delta was required for this feature. The bundled
catalog metadata was refreshed to release `0.69.0` and audit date
`2026-06-04`, but the model entries themselves did not change.

## Canon `0.67.0` Packet Audit Result

The Canon follow-up closed the original producer gap. Boundline's backlog gate
now aligns to evidence already emitted by Canon `0.67.0`:

- the full backlog packet still includes the authored planning artifacts
  (`backlog-overview.md`, `epic-tree.md`, `capability-to-epic-map.md`,
  `dependency-map.md`, `delivery-slices.md`, `sequencing-plan.md`,
  `acceptance-anchors.md`, and `planning-risks.md`)
- Canon can now also publish `execution-handoff.md` for downstream-ready full
  packets
- delivery-slice evidence is stable enough for Boundline to validate
  `slice_id`, implementation refs, and verification anchors without inventing
  a Boundline-only schema

This lets Boundline replace the legacy checklist-style `backlog.md` heuristic
with a real Canon packet audit while preserving Canon as the producer and
Boundline as the execution-admission owner.

## Decision 1: Reuse the existing typed backlog-quality assessment

**Decision**: Keep `BacklogQualityAssessment`, `BacklogQualityState`, and the
existing lifecycle projection helpers in `src/domain/governance.rs` as the
single backlog-quality domain owner.

**Rationale**: The workspace already persists and projects typed backlog state
through runtime, CLI, and assistant surfaces. A second validator would create
ordering drift without adding operator value.

## Decision 2: Keep one deterministic planning-gate order

**Decision**: Preserve the planning-gate order of `goal quality -> plan quality
-> backlog quality -> planning analysis -> execution handoff`.

**Rationale**: Operators need one actionable next step. Later-stage findings
must not appear while an earlier gate is unresolved.

## Decision 3: Treat Canon packet shape literally

**Decision**: Validate the Canon multi-document packet directly. Do not fall
back to the legacy checklist-style `backlog.md` parsing model.

**Rationale**: The backlog gate exists to prevent hidden translation from weak
planning artifacts into executable work. Literal packet validation is the
smallest honest consumer contract.

## Decision 4: Reserve `blocked` for unsafe or closure-limited packets

**Decision**: Treat structurally unsafe packets and closure-limited risk-only
packets as `blocked`, while using `clarification_required` only for otherwise
credible full packets that still lack execution-handoff evidence.

**Rationale**: A closure-limited packet is not a near-miss. It is an explicit
statement that Canon did not finish backlog decomposition to a downstream-ready
state.

## Decision 5: Close the slice as Boundline `0.69.0`

**Decision**: Ship the backlog contract as version `0.69.0`, align release
metadata and package manifests, update Boundline docs to reference Canon
`0.67.0` wherever backlog compatibility is described, and prove at least 95%
changed-file coverage for touched Rust implementation files.

**Rationale**: The feature changes execution-admission behavior and the public
compatibility story. Pre-1.0 semantics allow this as a minor release, but the
behavior and release surfaces must stay synchronized.
