# Synod Roadmap

Canon is downstream from Synod in this roadmap: Synod thinks, decides, orchestrates, and executes, while Canon governs meaningful flow stages and persists structured artifacts that Synod can reuse for reasoning.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status: v0.9.0

All core features now complete:

- **Session & Delivery**: active session persisted in `.synod/session.json`, explicit flow `start -> capture -> flow -> plan -> step/run -> status/next -> inspect`
- **Flows**: built-in `bug-fix`, `change`, and `delivery` flow definitions with stage-aware session state
- **Execution**: execution-profile-backed red-to-green delivery under `.synod/execution.json` with legacy `.synod/fixture.json` fallback, changed-file and validation evidence projected into surfaces
- **Review**: bounded review councils with manifest-driven reviewers, majority or weighted voting, optional adjudication, and review evidence across all surfaces
- **Adaptive**: workspace-slice selection, deterministic candidate synthesis, bounded replanning after failed validation, adaptive evidence projected into all session and inspect surfaces
- **Assistant Integration**: command packs aligned with session model and reuse of `latest_trace_ref`
- **Governance**: local-first and Canon-backed stage governance with packet readiness checks, packet provenance, approval refresh, and autopilot decision evidence across CLI surfaces

### Immediate follow-up directions

- deepen Canon governance with richer escalation and broader governed stage coverage
- broaden adaptive heuristics beyond the current deterministic local repair patterns
- deepen delivery and review beyond the current bounded local execution manifests
- multi-workspace and cross-repository orchestration
- advanced goal negotiation and constraint modeling

## Next Feature — Canon Governance Adapter

### Outcome

Synod binds meaningful flow stages to Canon runs through a CLI adapter, uses Canon-produced documents as governed reasoning inputs, and keeps Synod in control of orchestration and execution.

### Why next

The current core is stable enough to add governance without confusing it with flow definition or local execution. Canon should now add governed packets and durable documentation around the stages Synod already knows how to run.

### In scope

- `GovernanceRuntime` abstraction with a local default and a `CanonCliRuntime`
- optional Canon integration controlled by Synod config rather than a compile-time crate dependency
- stage-to-mode mapping from Synod flows into Canon modes for `bug-fix`, `change`, and `delivery`
- generation of Canon input documents per mode and stage
- capture of Canon run IDs and statuses inside the Synod session record
- reuse of Canon-produced documents as reasoning material for later Synod stages
- stage-scoped governance at meaningful boundaries, while Synod retains its own internal step trace

### Out of scope

- direct Rust dependency on Canon internals
- one Canon run per micro-step inside Synod
- replacing `.synod` traces with Canon artifacts
- mandatory governance for all users or all runs

### Operating model

```text
Synod flow stage -> Canon mode run -> governed documents -> Synod reasoning/execution
Synod internal steps -> Synod trace
```

### Intended flow

- Synod chooses the next stage and prepares the bounded context for it.
- Canon produces the stage documents and governed packet for that mode.
- Synod reads those documents to reason, plan, and constrain execution.
- Synod performs the actual coding, testing, and adaptive retry loop locally.
- Synod optionally sends later verification or PR-review stages back through Canon.

### Initial stage mapping

- `delivery`: `requirements -> requirements`, `architecture -> architecture`, `backlog -> backlog`, `implementation -> implementation`
- `change`: `understand-change -> change`, `implement -> implementation`, `verify -> verification` with optional `pr-review`
- `bug-fix`: `investigate -> discovery` or `change` depending on uncertainty, `implement -> implementation`, `verify -> verification` with optional `pr-review`
- Canon documents become bounded stage inputs for Synod, not replacements for Synod planning or trace storage.

### Tangible result

Synod stays the orchestrator, but gains governed documentation that improves planning, implementation boundaries, verification context, and auditability across the full delivery flow.

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