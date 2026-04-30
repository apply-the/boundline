# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.17.0

Synod now has its core session-native orchestration baseline in place:

- session-native orchestration is the primary operator path
- routing, `execution_condition`, `status`, `next`, and `inspect` now tell one coherent session story
- Canon is integrated as a bounded stage-boundary governance runtime, including verify-stage `security-assessment`
- Canon remains downstream from Synod rather than becoming a second orchestration plane

## Next Priority: 018-session-native-workflow-layer

The next feature slice should add a thin declarative workflow layer above the existing session-native runtime. The goal is to import the best part of workflow systems, resumable named workflows, without turning Synod into a generic workflow engine and without moving orchestration into Canon.

This next spec should explicitly deliver three things:

- a mini spec for an eventual `synod workflow` command family
- a concrete workflow syntax compatible with the current architecture, with TOML as the preferred first format and YAML left as a possible later interoperability layer
- a file-by-file attachment plan showing where the feature plugs into the existing codebase

### Priority rationale

- this is the smallest high-value workflow slice that builds on the current architecture instead of replacing it
- it gives assistant packs and future UX surfaces a stable declarative entrypoint
- it preserves Synod as the sole orchestrator and Canon as a governance runtime

### Mini spec direction

A developer can run a named workflow through Synod and keep the same session, routing, governance, review, and inspect surfaces that already exist today.

Initial command surface:

- `synod workflow list`
- `synod workflow run <name>`
- `synod workflow status`
- `synod workflow inspect`
- `synod workflow resume`

First-slice scope:

- named workflows compile into existing Synod phases such as `capture`, `clarify`, `plan`, `run`, `review`, `govern`, and `inspect`
- workflow progress persists on top of the existing `.synod/session.json` and `.synod/traces/` surfaces
- one bounded phase runs at a time through the existing session-native runtime
- the first slice does not introduce generic `while`, `switch`, `fan-out`, `fan-in`, or shell-first workflow semantics
- the first slice does not let Canon become a second controller or a second source of truth for progression

### Concrete syntax direction

Preferred first format: TOML.

Rationale:

- Synod already uses TOML in its configuration surface
- the first workflow slice should avoid introducing a second primary config dialect unless interoperability pressure justifies it later
- TOML fits a bounded named-workflow registry better than a general-purpose automation DSL

Candidate persistence surface:

- workspace-local `.synod/workflows.toml` for the initial slice
- optional split workflow files later if the single-file model becomes too large

Representative shape:

```toml
[workflow.default]
goal_source = "session"
entry = "capture"
phases = ["capture", "clarify", "plan", "run", "inspect"]
allow_review = true
allow_governance = true

[workflow.default.when]
clarify = "missing_authored_input"
review = "review_triggered"
governance = "governance_required"

[workflow.default.output]
next_command = true
routing_summary = true
execution_condition = true
```

YAML can follow later only if Synod needs interoperability with external workflow authoring surfaces. It should not be the first-class format of the first slice.

### File-by-file attachment plan

- `src/cli.rs` adds the workflow command family and argument parsing
- `src/cli/output.rs` renders workflow summary, compiled phase, and resume guidance
- `src/cli/inspect.rs` exposes workflow-oriented inspection summaries through the existing inspect surface
- `src/domain/session.rs` projects workflow identity, active phase, and workflow next action into session status
- `src/domain/goal_plan.rs` attaches originating workflow metadata to the bounded plan when applicable
- `src/orchestrator/session_runtime.rs` compiles workflow phases into existing session-native transitions while keeping routing authoritative
- `src/orchestrator/planner.rs` connects declared workflow phases to bounded planning decisions
- `src/orchestrator/goal_planner.rs` preserves workflow-derived intent during goal-plan construction
- `README.md` documents the relationship between session-native commands and workflow commands
- `assistant/README.md` documents how assistant command packs target the workflow command family once the CLI surface exists

### Secondary follow-up directions

- deepen Canon governance beyond the first `security-assessment` slice with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- migrate more review and compatibility configuration onto session-native summaries without losing bounded manifest support
- expand multi-workspace cluster support into full cross-repository execution planning and mutation
- advanced goal negotiation and constraint modeling

To be challenged:
- add a provider-agnostic model gateway inside Synod, with first-class provider auth flows and capability discovery
- decouple assistant command packs from model backends so Claude/Codex/Copilot surfaces map to routing slots instead of hard-wired providers

## Architecture: User Through Execution

```text
User / Copilot / Claude
        ↓
      Synod
  ┌───────────────┐
  │ Orchestrator  │
  │ Flows         │
  │ Agents        │
  │ Execution     │
  │ Review        │
  │ Adaptive      │
  └───────────────┘
        ↓
     Canon
   (governed stage docs + artifact persistence)
```

## In One Sentence

Synod is a system that takes a problem and transforms it into working code, orchestrating bounded execution itself while using Canon to govern stage outputs and provide reusable documentation.