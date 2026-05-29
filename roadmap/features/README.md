# Boundline Feature Seeds

This directory holds roadmap seeds for future Speckit work. These files are not
Speckit specs. A seed explains why a feature may matter, where Boundline should
own the behavior, and what must be preserved when a maintainer later creates a
real feature under `specs/`.

Use the local Speckit templates as the source of truth when converting a seed:

- `.specify/templates/spec-template.md`
- `.specify/templates/plan-template.md`
- `.specify/templates/tasks-template.md`
- `.specify/memory/constitution.md`

## Conversion Rules

1. Convert one seed into the smallest independently valuable delivery slice.
2. Do not copy a seed directly into `spec.md`; rewrite it as user stories,
   failure paths, requirements, success criteria, and catalog-currency evidence.
3. Keep Boundline independently testable. Canon may provide governed inputs, but
   Canon must not define Boundline's core control flow.
4. If a Boundline slice requires new Canon output or schema, stop and create a
   Canon Speckit feature in `/Users/rt/workspace/apply-the/canon` first.
5. Treat trace, inspectability, blocked states, and recovery as mandatory parts
   of the first slice, not later polish.
6. Avoid bundled systems. Provider protocol, sandboxing, browser validation,
   gateway routing, and memory should remain separate unless a smaller slice
   cannot deliver visible value.

## Sequencing

| Lane | Seeds | Purpose | First Speckit Slice |
|---|---|---|---|
| Now | 03, 04, 05 | Planning readiness gates | Surface one runtime gate at a time and block invalid execution handoff. |
| Now | 08 | Measurement substrate | Add local event schema plus a tiny golden eval corpus before expanding AI behavior. |
| Now | 06 | Large-repo safety | Refuse unsafe huge reads, add paged reads, and show omitted context in inspect. |
| Next | 07 | Provider boundary | Implement one read-only provider lifecycle before mutation providers. |
| Next | 09 | Operator discoverability | Start with Boundline `help-next`; keep Canon help as Canon-owned. |
| Next | 12 | Execution isolation | Add one local test sandbox mode with artifact capture before mutation commit. |
| Later | 10, 11 | Governance hardening | Treat as deltas over shipped council/adaptive docs, not greenfield systems. |
| Later | 13 | Route economics | Add route telemetry and budgets after provider protocol and evals exist. |
| Later | 14 | Browser validation | Implement as a concrete provider over S10, not as core runtime. |
| Later | 15 | Memory hygiene | Start with confirmation-first trace distillation; no autonomous memory. |

Seeds 01 and 02 are intentionally not revised by this pass.

## Dependency Graph

```text
03 plan gate
  -> 04 backlog gate
  -> 05 planning analysis
  -> 08 evals and observability
  -> 06 large-codebase context substrate
  -> 07 provider protocol
      -> 12 sandbox execution
      -> 14 browser provider
      -> 13 AI gateway economics

08 evals and observability
  -> 10 council hardening
  -> 11 adaptive governance hardening
  -> 13 AI gateway economics
  -> 15 session memory

09 help-next can start after the probe/readiness surfaces are available.
```

## Speckit Readiness Checklist

Before creating a Speckit feature from any seed, answer these questions in the
new `spec.md`:

- What is the first operator-visible delivery improvement?
- Which single user story can be tested independently?
- What non-success path blocks, degrades, retries, or asks for clarification?
- What state is persisted in `.boundline/session.json` or traces?
- What does `status`, `next`, or `inspect` show?
- Which existing runtime or doc surface already owns part of this behavior?
- Which related ideas are explicitly out of scope for this slice?
- Does the slice need provider-model catalog research, and where is it recorded?

## Duplication Register

| Cluster | Overlap | Ownership Decision |
|---|---|---|
| Planning gates | 03, 04, and 05 all repeat gate state, `phase_request`, and assistant routing language. | Keep the shared gate and handoff mechanics in one planning-readiness interface; each seed should add only its own validation fields and findings. |
| Canon companion work | 04, 08, 09, and 15 all mention Canon-owned packets, help, evals, or project memory. | Boundline consumes stable Canon metadata. Canon schema, mode docs, packet-quality evals, and memory promotion need Canon Speckit features. |
| Provider permissions | 07, 12, and 14 all describe path, network, secret, artifact, and evidence permissions. | 07 owns the request permission envelope; 12 enforces sandbox policy; 14 consumes the envelope as a browser provider. |
| Telemetry | 08, 13, and 14 all list events, route metrics, artifacts, and latency/cost signals. | 08 owns event schema and eval fixtures; 13 owns route economics decisions; concrete providers emit events into 08. |
| Councils and adaptive governance | 10 and 11 overlap with shipped docs under `docs/review-*`, `docs/adaptive-governance.md`, `docs/control-graduation-model.md`, and `docs/runtime-confidence-and-calibration.md`. | Future specs must name the missing delta instead of reimplementing council profiles, voting, confidence, or degradation from scratch. |
| Memory | 15 overlaps with `docs/project-memory-and-evidence-structure.md` and Canon project memory. | 15 owns workspace-local, confirmation-first operational memory proposals. Durable governed knowledge remains docs/Canon-owned. |
| Help and docs | 09 overlaps with README, wiki, getting-started, and Canon docs work. | Start with runtime `help-next` state diagnosis; documentation IA work should follow the command surface and avoid duplicating README prose. |

## Seed-to-Spec Template

When promoting a seed, use this short bridge before writing the real Speckit
spec:

```text
Seed:
Candidate spec slug:
Primary operator path:
First independent story:
Success path:
Non-success path:
Runtime state written:
Inspect/status/next projection:
Existing owner to reuse:
Explicitly deferred:
Catalog research needed:
```
