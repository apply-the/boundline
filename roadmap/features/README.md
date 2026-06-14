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
   Canon Speckit feature in the sibling `canon` repo first.
5. Treat trace, inspectability, blocked states, and recovery as mandatory parts
   of the first slice, not later polish.
6. Avoid bundled systems. Provider protocol, sandboxing, browser validation,
   gateway routing, and memory should remain separate unless a smaller slice
   cannot deliver visible value.

## Sequencing

| Lane | Seeds | Purpose | First Speckit Slice |
|---|---|---|---|
| Delivered | 03 | Planning readiness gate | Shipped in 0.67.0; preserves one-question `phase_request` recovery and additive plan-quality projections. |
| Delivered | 04 | Backlog readiness gate | Shipped in 0.69.0; validates Canon backlog packets before execution handoff and preserves additive backlog-quality projections. |
| Delivered | 05 | Planning analysis gate | Shipped in 0.70.0; adds a read-only cross-artifact coherence gate with source-attributed findings and withheld execution handoff on contradictions or producer gaps. |
| Delivered | 06 / 070 | Large-repo safety | Shipped in 0.71.0 with typed fidelity tiers, omission findings, repository-map visibility, digest-backed compaction, and patch-safe edit guards. |
| Delivered | 07 / 071 | Provider boundary and setup | Shipped in 0.72.0 with explicit operator registration, setup-requirement projection, fail-closed activation and permission admission, and additive evidence-first provider runtime projections. |
| Next | 08 | Measurement substrate | Add local event schema plus a tiny golden eval corpus before expanding AI behavior. |
| Next | 09 | Operator discoverability | Start with Boundline `help-next`; keep Canon help as Canon-owned. |
| In Progress (spec 077) | 13 | Safe command execution | Add command intent classification, execution policy, evidence capture, artifact manifest, secret redaction, and mutation boundaries. No Docker required. |
| Deferred | 13B | Sandbox execution runtime | Docker sandbox with mount/network/filesystem policy and secret handle inheritance. Depends on provider permissions (07) and execution evidence foundation (13A). Moved to `unplanned/`. |
| Delivered (spec 079) | 18 | Completion proof gate | Shipped in 0.79.0 with runtime-owned claim inference, fresh proof execution, stale-proof invalidation, and additive completion-verification projections. |
| Next | 19 | Execution control plane | Add one sequential task runner with checkpoint and resume after proof gating is stable. |
| Next | 20 | Route economics | Add route telemetry and budgets after provider protocol and evals exist. |
| Next | 21 | Browser validation | Implement as a concrete provider over S10, not as core runtime. |
| Next | 22 | Memory hygiene | Start with confirmation-first trace distillation; no autonomous memory. |
| Later | 10, 11 | Governance hardening | Treat as deltas over shipped council/adaptive docs, not greenfield systems. |
| Later | 12 | Recursive refinement | Add one bounded, inspectable sequential stage-refinement profile after council and adaptive-governance hardening. |
| Experimental | 23 | RecursiveMAS provider | Evaluate real latent-space recursion only as an external read-only provider after provider, eval, route-budget, and host-refinement boundaries exist. |

Seed 02 is intentionally not revised by this pass.

## Dependency Graph

```text
03 plan gate
  -> 04 backlog gate
  -> 05 planning analysis
   -> 18 / 079 completion verification runtime
         -> 19 plan execution orchestration
            -> 20 AI gateway economics
               -> 21 browser provider
                  -> 22 session memory
  -> 08 evals and observability
  -> 07 provider protocol
      -> 13 safe command execution
      -> 21 browser provider
      -> 20 AI gateway economics

08 evals and observability
  -> 10 council hardening
      -> 11 adaptive governance hardening
          -> 12 recursive stage refinement
  -> 20 AI gateway economics
  -> 22 session memory

09 help-next can start after the probe/readiness surfaces are available.

18 completion verification runtime (delivered in spec 079 / 0.79.0)
   -> 19 plan execution orchestration
      -> 20 AI gateway economics
         -> 21 browser provider
            -> 22 session memory
               -> 23 recursiveMAS provider (experimental)

07 provider protocol (shipped in 0.72.0)
13 safe command execution (in progress)
13B sandbox execution runtime (deferred, unplanned/)
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
| Planning gates | 03, 04, and 05 all repeat gate state, `phase_request`, and assistant routing language. | Keep the shared gate and handoff mechanics in one planning-readiness interface; 03 is now shipped, and 04/05 should add only their own validation fields and findings. |
| Canon companion work | 04, 08, 09, and 21 all mention Canon-owned packets, help, evals, or project memory. | Boundline consumes stable Canon metadata. Canon schema, mode docs, packet-quality evals, and memory promotion need Canon Speckit features. |
| Provider permissions | 07, 13, and 21 all describe path, network, secret, artifact, and evidence permissions. | 07 owns the request permission envelope and provider setup or activation surface; 13 enforces sandbox policy and secret-handle execution rules; 21 consumes the envelope as a browser provider. |
| Execution ownership | 18, 19, Canon completion or handoff seeds, and provider or sandbox execution surfaces all mention proof, task state, checkpointing, or blocked execution. | 18 owns proof execution and completion blocking; 19 owns task ordering, checkpoint, and resume; 07 and 13 provide provider and sandbox backends; Canon owns packet semantics and evidence consumption only. |
| Telemetry | 08, 20, and 21 all list events, route metrics, artifacts, and latency/cost signals. | 08 owns event schema and eval fixtures; 20 owns route economics decisions; concrete providers emit events into 08. |
| Councils and adaptive governance | 10 and 11 overlap with shipped docs under `docs/review-*`, `tech-docs/adaptive-governance.md`, `tech-docs/control-graduation-model.md`, and `tech-docs/runtime-confidence-and-calibration.md`. | Future specs must name the missing delta instead of reimplementing council profiles, voting, confidence, or degradation from scratch. |
| Memory | 22 overlaps with `tech-docs/project-memory-and-evidence-structure.md` and Canon project memory. | 22 owns workspace-local, confirmation-first operational memory proposals. Durable governed knowledge remains docs/Canon-owned. |
| Help and docs | 09 overlaps with README, wiki, getting-started, and Canon docs work. | Start with runtime `help-next` state diagnosis; documentation IA work should follow the command surface and avoid duplicating README prose. |
| Recursive collaboration | 10, 11, 12, 20, and 23 all touch repeated reasoning, stop semantics, cost, and model execution. | 12 owns host-governed structured stage-refinement loops. 23 is an external latent-space provider experiment. Councils, calibration, and route economics remain owned by 10, 11, and 20. |

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
