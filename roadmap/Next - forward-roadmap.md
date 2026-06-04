# Next Boundline Roadmap

## Purpose

This document captures the next Boundline roadmap and normalizes it around
Boundline ownership.

Canon remains a governed knowledge and packet authority. Boundline owns runtime
movement, inspection, admission control, execution boundaries, guidance,
guardians, review, verification, and provider integration.

## Ownership Rule

A feature belongs in this Boundline roadmap only when Boundline is the runtime
owner or the feature changes a Boundline-owned operator surface.

Canon-exclusive work is not a Boundline roadmap item. It can appear here only as
a companion dependency that Boundline consumes through stable contracts.

## Feature Seeds

All roadmap work is now tracked in dedicated seed files under `features/`.
These are not Speckit specs. Use
[features/README.md](features/README.md) for conversion rules, sequencing,
MVP slices, dependencies, and duplication ownership before creating a real
feature under `specs/`.

| Priority | Feature Seed | Boundline Position |
|---|---|---|
| **02** | [agentic-framework-integration.md](features/02-agentic-framework-integration.md) | Delivered in 0.66.0: external framework-adapter runtime, corrected split-stage Speckit bridge revalidated on 2026-06-01 |
| **03** | plan-quality-contract.md | Delivered in 0.67.0: first plan-readiness gate, one-question recovery, additive plan-quality projections |
| **04** | [backlog-contract.md](features/04-backlog-contract.md) | Delivered in 0.69.0: Canon backlog packet gate, closure-limited blocking, and additive backlog-quality projections |
| **05** | [plan-analysis-contract.md](features/05-plan-analysis-contract.md) | Delivered in 0.70.0: read-only planning-coherence gate, source-attributed findings, and withheld execution handoff on contradictions or producer gaps |
| **06** | [large-codebase-context-substrate.md](features/06-large-codebase-context-substrate.md) | Long-term context handling limits |
| **07** | [external-capability-provider-protocol.md](features/07-external-capability-provider-protocol.md) | Native provider contract, setup, and activation surface (replaces MCP) |
| **08** | [evals-and-runtime-observability.md](features/08-evals-and-runtime-observability.md) | Local quality and regression layer |
| **09** | [contextual-help-and-documentation-architecture.md](features/09-contextual-help-and-documentation-architecture.md) | Operator UX feature |
| **10** | [review-councils-and-role-gated-governance.md](features/10-review-councils-and-role-gated-governance.md) | Extend delivered S3/S056 work |
| **11** | [adaptive-governance-calibration.md](features/11-adaptive-governance-calibration.md) | Extend delivered S4/S057 work |
| **12** | [recursive-stage-refinement-profiles.md](features/12-recursive-stage-refinement-profiles.md) | Later: bounded structured refinement after council and adaptive-governance hardening |
| **13** | [sandboxed-execution-and-secret-inheritance.md](features/13-sandboxed-execution-and-secret-inheritance.md) | Local safety boundaries |
| **14** | [ai-gateway-and-inference-economics.md](features/14-ai-gateway-and-inference-economics.md) | Scale and route health feature |
| **15** | [browser-and-visual-testing-provider.md](features/15-browser-and-visual-testing-provider.md) | Provider via protocol |
| **16** | [session-memory-and-repository-knowledge-distillation.md](features/16-session-memory-and-repository-knowledge-distillation.md) | Memory hygiene feature |
| **17** | [experimental-recursivemas-provider-adapter.md](features/17-experimental-recursivemas-provider-adapter.md) | Experimental: external latent-space provider research track |
| **18** | [completion-verification-runtime.md](features/18-completion-verification-runtime.md) | Next: fresh-proof gate before task or stage completion |
| **19** | [plan-execution-orchestration.md](features/19-plan-execution-orchestration.md) | Later: sequential execution control plane with checkpoint and resume |

## Canon Companion Dependencies

The following work is intentionally outside the Boundline roadmap:

- Canon mode templates and packet quality validation.
- Canon `help-next` and Canon wiki restructuring.
- Canon project-memory promotion rules.

Boundline should consume those surfaces only through stable, versioned metadata:

- packet readiness state
- required document list
- evidence refs
- progress or handoff packet schemas when Canon provides them
- lineage refs
- approval state
- project-memory promotion status
- mode template discovery

## Feature Boundaries

### Large Codebase Context Hardening

Boundline already has the local SQLite and FTS5 advanced-context substrate plus
local semantic acceleration. The next slice should harden the runtime boundary:

- refuse huge full-file reads unless explicitly allowed
- support paged file reads with stable digests
- store lazy hash references for huge logs, diffs, CI output, and generated files
- show omitted context and skip reasons in inspect
- require source spans for edits, not summaries
- use anchored hunks and post-apply verification for large-file edits

### External Capability Provider Protocol

Provider output is not truth. Providers produce claims, findings, artifacts,
evidence, and state patch proposals. Boundline validates, traces, accepts,
rejects, or escalates.

V1 should define:

- `capabilities`
- `health`
- `prepare`
- `execute`
- `collect_evidence`
- explicit permissions: read files, write files, run commands, network,
  read secrets, write artifacts, allowed paths, runtime limit, output limit

#### Operator Setup And Activation

Boundline should own the provider onboarding runtime that Canon intentionally
does not own:

- explicit operator registration and activation
- setup requirement projection before first use
- health or connectivity dry-runs before a provider is marked ready
- no auto-enable from local executable discovery
- secret-handle routing rather than prompt-visible secret capture

Canon may later record local routing intent, but provider health, permissions,
and activation remain Boundline-owned.

#### Open Model Provider Support

Open-weight models should enter Boundline through the provider contract, not as
new core runtime assumptions. Boundline should treat Qwen-like, Gemma-like, and
Llama-like models as model families exposed by a provider adapter.

Candidate provider surfaces:

- Ollama
- `llama.cpp` server
- vLLM or Text Generation Inference
- OpenAI-compatible local gateways
- hosted OpenAI-compatible gateways such as OpenRouter, Together, or Fireworks
- organization-managed inference gateways

Provider capability metadata should include:

- provider kind: local, self-hosted remote, or hosted remote
- model family and exact model id
- context window and max output
- structured-output, JSON-schema, and tool/function-call support
- streaming support
- local resource envelope: memory, GPU, concurrency, and expected latency
- cost envelope for hosted routes
- network, secret, and repository-content transmission policy
- evidence artifacts the provider can return after execution

Suggested route presets:

- `open-code`: Qwen-like models for implementation, structured edits, tool
  invocation, and schema-constrained output.
- `open-reasoning`: Gemma-like models for planning, design review, complex
  debugging, and algorithmic reasoning.
- `large-context-audit`: Llama-like long-context models for broad repository
  discovery, legacy-system inspection, and context-heavy review.

Hard boundaries:

- no open model becomes a default route without Boundline eval evidence
- no benchmark claim is trusted without local regression checks
- no hosted route receives repository content unless the operator approves the
  remote transmission policy
- no provider can bypass Boundline stop rules, governance, trace, or evidence
  capture
- Canon-owned packet-quality validation remains Canon-owned even when an open
  model is used to produce or review supporting material

### Evals And Runtime Observability

Start with a local quality layer rather than an external observability platform:

- JSONL trace export
- stable event schema
- golden task corpus
- deterministic scoring where possible
- model/provider route telemetry
- context-selection, stop-semantics, guardian, council, and provider evals

Open-model promotion requires targeted evals before a route can be recommended:

- tool/function-call and JSON-schema validity
- multi-file patch accuracy and post-apply verification
- context retention across large files and omitted-context boundaries
- reasoning/debugging quality on known failure corpora
- stop-rule, governance, and evidence-capture compliance
- latency, memory, concurrency, and hosted-route cost envelopes
- regression gates when changing provider, model id, quantization, context
  window, prompt wrapper, or structured-output mode

Canon packet-quality evals belong to Canon. Boundline should only carry
consumer-side regression checks for the Canon metadata it relies on.

### Completion Verification Runtime

Boundline should own claim-matched proof execution before work is allowed to
close.

V1 should support:

- concrete claim derivation from runtime closeout
- narrowest proof-command selection
- fresh execution in the current working state
- blocked completion when proof is missing, stale, or failing
- `claim -> proof -> evidence_ref` projection for Canon consumption

Hard boundaries:

- Boundline must not report task or stage success before the proof is ready
- Canon may govern the meaning of approval or readiness, but it must not own
  proof execution

### Plan Execution Orchestration

Boundline should own the execution control plane for accepted task registries.

V1 should support:

- one sequential execution profile
- active task locking
- task-local validation and completion-proof gating
- explicit checkpoint persistence and resume command projection
- progress and handoff projection that Canon can consume later

Hard boundaries:

- this feature is distinct from seed 12 recursive refinement
- no autonomous replanning in the first slice
- blocked, skipped, and deferred states must remain visible rather than being
  collapsed into complete

### Help-Next And Documentation Architecture

Boundline should own:

- `boundline help-next`
- workspace and session state diagnosis
- exact next command recommendation
- docs/wiki link projection
- recovery guidance for blocked, failed, or degraded states
- Boundline docs focused on governed movement

Canon help and Canon mode documentation remain Canon-owned companion work.

### Constitution Command And Standards Surface

This is a follow-up feature after planning-trust hardening, not part of the
current planning UX slice.

V1 should support:

- `boundline constitution create`
- `boundline constitution update`
- scoped principles rather than one monolithic policy blob
- repo-visible standards that planning and status can reference later
- later reminder behavior when no constitution exists

Hard boundaries:

- `init` must not scaffold the constitution automatically
- the constitution command must not become a second governance runtime
- Canon may consume constitution outputs later, but Boundline owns the command
  surface and workspace behavior
  
The first planning-UX slice should only reserve the roadmap slot for this work
and keep current status or planning output neutral until the constitution
feature actually ships.

### Sandboxed Execution And Secret Inheritance

Sandboxing is a Boundline runtime responsibility. Canon may classify work that
requires sandboxing, but Canon should not run sandboxes.

V1 should support:

- read-only, test, mutation, and migration-dry-run modes
- local Docker or equivalent local sandbox where available
- path and network policy
- secret handles rather than prompt-visible secret values
- patch/artifact/log/evidence bundle output
- explicit commit or rollback

### AI Gateway And Inference Economics

The gateway layer should make model selection operational rather than
brand-driven. It should sit after provider protocol and evals, then manage:

- route health and fallback policy
- local versus remote transmission decisions
- route budgets by slot, reviewer role, and delivery stage
- provider/model latency and reliability telemetry
- cost and local resource reporting
- eval-gated promotion, downgrade, and rollback of model routes

The first useful output is not a marketplace. It is an auditable route matrix:

| Route family | Intended use | Promotion gate |
|---|---|---|
| `open-code` | implementation, refactoring, structured tool calls | coding, schema, patch, and stop-rule evals |
| `open-reasoning` | planning, architecture review, complex debugging | reasoning, review quality, and governance evals |
| `large-context-audit` | repository-wide audit and legacy discovery | context-retention, omission, latency, and evidence evals |

### Recursive Stage Refinement Profiles

Recursive refinement should enter Boundline as a bounded runtime profile, not
as hidden multi-agent autonomy.

The first useful slice is one opt-in sequential planning profile:

```text
planner -> critic -> planner -> finalizer
```

Each round must persist a compact structured packet, reference artifacts rather
than copy transcripts, and expose its delta, blockers, stop reason, and final
outcome through the normal session and trace surfaces.

This feature belongs after council and adaptive-governance hardening because
existing council, calibration, degradation, and stop semantics must govern the
loop. It must remain useful without `sqlite-vec`; the retrieval index is not an
agent communication bus.

### Experimental RecursiveMAS Provider

Real RecursiveMAS latent-state transfer may be evaluated later as an external
provider experiment. It does not belong in Boundline core.

The experiment should:

- run outside core crates
- begin with one bounded read-only capability
- declare checkpoint, model, and hardware prerequisites
- return only final decoded outputs, metrics, evidence, limitations, and
  reproducibility metadata
- compare against the simpler structured-recursion baseline before any
  promotion decision

Paper benchmark gains do not replace local eval evidence for software-delivery
workloads.

## Features Explicitly Not Next

These remain out of scope until the visible trust roadmap is credible:

- swarm intelligence as a core runtime model
- GOAP as the main planning engine
- autonomous background workers
- zero-trust federation
- full client-server platform
- unbounded multi-agent reasoning profiles that bypass governed councils,
  adaptive calibration, route budgets, or trace-visible stop semantics
- latent-state execution inside Boundline core
- model marketplace

## Hard Rule

Build visible trust before advanced autonomy. The safe path must be easier to
understand and operate than bypassing governance.
