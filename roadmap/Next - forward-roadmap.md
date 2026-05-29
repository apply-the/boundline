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

## Priority Order

All roadmap work is now tracked in dedicated spec files under `features/`.

| Priority | Feature Spec | Boundline Position |
|---|---|---|
| **01** | [sqlite-vec-activation-and-db-merge-strategy.md](features/01-sqlite-vec-activation-and-db-merge-strategy.md) | Advanced context substrate execution |
| **02** | [agentic-framework-integration.md](features/02-agentic-framework-integration.md) | Architecture for external harness adapters |
| **03** | [plan-quality-contract.md](features/03-plan-quality-contract.md) | Planning UX - plan readiness gate |
| **04** | [backlog-contract.md](features/04-backlog-contract.md) | Planning UX - backlog execution gate |
| **05** | [plan-analysis-contract.md](features/05-plan-analysis-contract.md) | Planning UX - cross-artifact analysis |
| **06** | [large-codebase-context-substrate.md](features/06-large-codebase-context-substrate.md) | Long-term context handling limits |
| **07** | [external-capability-provider-protocol.md](features/07-external-capability-provider-protocol.md) | Native provider contract (replaces MCP) |
| **08** | [evals-and-runtime-observability.md](features/08-evals-and-runtime-observability.md) | Local quality and regression layer |
| **09** | [contextual-help-and-documentation-architecture.md](features/09-contextual-help-and-documentation-architecture.md) | Operator UX feature |
| **10** | [review-councils-and-role-gated-governance.md](features/10-review-councils-and-role-gated-governance.md) | Extend delivered S3/S056 work |
| **11** | [adaptive-governance-calibration.md](features/11-adaptive-governance-calibration.md) | Extend delivered S4/S057 work |
| **12** | [sandboxed-execution-and-secret-inheritance.md](features/12-sandboxed-execution-and-secret-inheritance.md) | Local safety boundaries |
| **13** | [ai-gateway-and-inference-economics.md](features/13-ai-gateway-and-inference-economics.md) | Scale and route health feature |
| **14** | [browser-and-visual-testing-provider.md](features/14-browser-and-visual-testing-provider.md) | Provider via protocol |
| **15** | [session-memory-and-repository-knowledge-distillation.md](features/15-session-memory-and-repository-knowledge-distillation.md) | Memory hygiene feature |

## Canon Companion Dependencies

The following work is intentionally outside the Boundline roadmap:

- Canon mode templates and packet quality validation.
- Canon `help-next` and Canon wiki restructuring.
- Canon project-memory promotion rules.

Boundline should consume those surfaces only through stable, versioned metadata:

- packet readiness state
- required document list
- evidence refs
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

## Features Explicitly Not Next

These remain out of scope until the visible trust roadmap is credible:

- swarm intelligence as a core runtime model
- GOAP as the main planning engine
- autonomous background workers
- zero-trust federation
- full client-server platform
- advanced multi-agent reasoning profiles beyond governed councils
- model marketplace

## Hard Rule

Build visible trust before advanced autonomy. The safe path must be easier to
understand and operate than bypassing governance.
