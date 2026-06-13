# Experimental RecursiveMAS Provider Adapter

## Owner

External provider repository, integrated through Boundline

## Status

Experimental research track, not a core-runtime commitment

## Speckit Seed Notes

- Seed role: isolate any real latent-space RecursiveMAS experiment behind the
  Boundline provider boundary.
- First slice: run one read-only benchmark-oriented provider capability that
  accepts a bounded task, invokes an external RecursiveMAS runtime, and returns
  a final answer plus trace-linked metrics and limitations.
- Depends on: provider protocol from seed 07, event and eval substrate from seed
  08, route telemetry and budget policy from seed 14, recursive-stage
  boundaries from seed 12, and a proven bounded lifecycle for long-lived local
  providers when checkpoint loading makes one-shot execution impractical.
- De-duplication: seed 12 owns Boundline-native recursive refinement; this seed
  owns only the external ML experiment. It must not add hidden-state semantics
  to Boundline core.

## Inspiration And Boundary

RecursiveMAS is a latent-space recursive multi-agent framework:

- paper: `https://arxiv.org/abs/2604.25917`
- project: `https://recursivemas.github.io/`
- reference implementation: `https://github.com/RecursiveMAS/RecursiveMAS`

Its RecursiveLink modules transfer hidden representations across heterogeneous
agents. The released reference path loads role-specific model checkpoints and
outer links for GPU-oriented inference.

That runtime does not belong in Boundline core.

If the approach proves useful for software-delivery workloads, it should enter
through an external provider repository such as:

```text
boundline-provider-recursivemas
```

## Strategic Role

This track answers a research question without distorting Boundline:

```text
Can a local latent-space multi-agent provider improve selected bounded
software-delivery tasks enough to justify its operational cost?
```

The answer may be no. The roadmap should preserve that possibility.

## Problem

The paper reports strong benchmark results for its own evaluated tasks, but
Boundline needs local evidence before treating latent-space collaboration as a
useful delivery capability:

- software-delivery tasks may not match the paper's benchmark mix
- checkpoint loading and GPU requirements may be operationally expensive
- hidden-state execution is less inspectable than structured Boundline packets
- token savings do not automatically imply lower total cost
- provider outputs may be difficult to reproduce across model and checkpoint
  revisions

## Core Principle

Hidden state may exist inside a provider. It is never authoritative Boundline
state.

Boundline should send a bounded request and receive only inspectable outputs:

```text
Boundline request
  -> external RecursiveMAS provider
  -> final answer or artifact
  -> metrics, limitations, reproducibility metadata, evidence refs
  -> Boundline validation, trace, acceptance, rejection, or escalation
```

## First Slice

Start with one read-only, benchmark-oriented capability. Do not begin with code
mutation.

The provider should:

- declare health, model family, checkpoint versions, collaboration style, and
  local resource requirements
- accept one bounded task with explicit runtime and output limits
- run one supported RecursiveMAS collaboration style
- return the final decoded output only
- return latency, resource, and token metrics when available
- return reproducibility metadata and limitations
- expose failures as provider outcomes rather than mutating Boundline state

Boundline should:

- enforce admission control before provider execution
- keep the capability opt-in
- capture provider metrics through the normal event substrate
- compare outputs against a local eval corpus
- surface unsupported hardware, missing checkpoints, and provider failure
  clearly
- reject any attempt to persist opaque hidden state as runtime truth

## Provider Capability Sketch

```json
{
  "provider_id": "recursivemas-local",
  "capability": "latent_recursive_reasoning",
  "mode": "read_only",
  "collaboration_style": "sequential_light",
  "model_refs": [],
  "checkpoint_refs": [],
  "resource_requirements": {
    "accelerator": "required",
    "runtime_limit": "bounded"
  }
}
```

The exact provider schema belongs to the provider-protocol feature and its
follow-ups. This seed records the required experiment boundary, not a parallel
protocol.

## Evaluation Gate

Promotion beyond experiment status requires local evidence for representative
Boundline workloads:

- planning quality
- structured-output validity
- reproducibility across repeated runs
- latency and local resource use
- token use and total cost
- failure and degradation behavior
- trace completeness
- comparison against simpler structured recursive refinement from seed 12

No paper benchmark result substitutes for these checks.

## Explicitly Out Of Scope

- embedding or hidden-state storage in the Boundline retrieval index
- RecursiveLink implementation in Rust core crates
- model checkpoint bundling in Boundline releases
- gradient training inside Boundline
- remote opaque hidden-state authority
- code mutation in the first slice
- automatic route promotion
- default enablement
- replacement of structured stage refinement from seed 12

## Acceptance Criteria

- The experimental provider runs outside Boundline core.
- Boundline can discover provider readiness and reject missing prerequisites
  before execution.
- One bounded read-only task returns a final decoded output plus metrics,
  limitations, and reproducibility metadata.
- Provider failure is trace-visible and does not mutate authoritative session
  state silently.
- Local evals compare the experiment with a simpler baseline.
- Hidden states remain provider-internal and are never persisted as
  authoritative Boundline state.
- The feature can be removed without changing Boundline core semantics.

## Risks

- The experiment is mistaken for a committed product direction.
- GPU and checkpoint costs outweigh quality improvements.
- Provider-local hidden state weakens reproducibility.
- Benchmark gains do not transfer to delivery workflows.
- The external provider boundary becomes a pretext for bypassing governance.

## Hard Rule

Latent-space execution is an optional provider experiment, never a Boundline
control plane.

# Note
## Decision

The real RecursiveMAS runtime must remain an external experimental provider.

It should not enter Boundline core and should not be confused with S22 Recursive Stage Refinement Profiles.

## Placement

S17 is the right place for:

- external RecursiveMAS provider
- benchmark-oriented read-only capability
- local hardware/resource requirement checks
- provider metrics
- limitations
- reproducibility metadata
- comparison against simpler Boundline-native refinement

## Dependency alignment

S17 should depend on:

- S10 provider protocol
- S11 event and eval substrate
- S19 route telemetry and budget policy
- S22 recursive stage refinement boundaries
- persistent provider lifecycle if checkpoint loading makes one-shot execution impractical

## Hard boundary

Hidden state may exist inside the provider. It is never authoritative Boundline state.

Boundline sends a bounded request and receives:

- final decoded output
- metrics
- limitations
- reproducibility metadata
- evidence refs

Boundline then validates, traces, accepts, rejects, or escalates.

## Hard rule

Latent-space execution is an optional provider experiment, never a Boundline control plane.

# Note
# Experimental RecursiveMAS Provider Adapter Note

## Purpose

This note clarifies how the RecursiveMAS experiment relates to the DS4-inspired roadmap additions.

## Decision

The real RecursiveMAS runtime must remain an external experimental provider.

It should not enter Boundline core and should not be confused with `12-recursive-stage-refinement-profiles.md`.

## Placement

`17-experimental-recursivemas-provider-adapter.md` remains the right place for:

- external RecursiveMAS provider
- benchmark-oriented read-only capability
- local hardware/resource requirement checks
- provider metrics
- limitations
- reproducibility metadata
- comparison against simpler Boundline-native refinement

## Dependency Alignment

This item should depend on:

- `specs/071-capability-provider-protocol/spec.md`
- `../specs/072-evals-runtime-observability/feat-evals-and-runtime-observability.md`
- `14-ai-gateway-and-inference-economics.md`
- `12-recursive-stage-refinement-profiles.md`
- a proven bounded lifecycle for long-lived local providers when checkpoint loading makes one-shot execution impractical

## Hard Boundary

Hidden state may exist inside the provider. It is never authoritative Boundline state.

Boundline sends a bounded request and receives:

- final decoded output
- metrics
- limitations
- reproducibility metadata
- evidence references

Boundline then validates, traces, accepts, rejects, or escalates.

## Explicit Non-Goals

- no hidden-state storage in Boundline retrieval index
- no RecursiveLink implementation in core Rust crates
- no checkpoint bundling in Boundline releases
- no default enablement
- no code mutation in first slice
- no replacement of `12-recursive-stage-refinement-profiles.md`

## Hard Rule

Latent-space execution is an optional provider experiment, never a Boundline control plane.
