# Synod Roadmap

Canon is outside the scope of this roadmap: it is the runtime that persists structured artifacts. Synod is the system that thinks, decides, orchestrates, and executes.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status

The `Spec 1.3 — Session & Interaction Model Unification` slice and `Spec 005 — Delivery Flows` are now available in the local CLI.

- active session persisted in `.synod/session.json`
- explicit flow `start -> capture -> flow -> plan -> step/run -> status/next -> inspect`
- built-in `bug-fix`, `change`, and `delivery` flow definitions with stage-aware session state
- flow-aware `status`, `next`, `run`, and `inspect` output with stage transition and recovery traces
- fixture-backed red-to-green execution slice under `.synod/fixture.json`, replacing the old synthetic onboarding surface
- assistant command packs aligned with the session model and reuse of `latest_trace_ref`

Immediate follow-up:

- expand the execution engine beyond the current fixture-backed red-to-green slice
- add multi-agent review and voting on top of the execution output

## Spec 006 — Execution Engine (Code Delivery)

### Outcome

Synod actually performs development work:

- writes code
- modifies files
- runs tests
- validates output

### Why now

Without real execution, you do not deliver.

### In scope

- workspace interaction:
- read/write file
- diff generation
- test execution hooks
- validation loop: generate -> run -> fix -> retry
- runtime error handling

### Out of scope

- full CI/CD
- deploy
- deep governance, which remains in Canon

### Tangible result

Synod can take a slice and produce working code, not just suggestions.

## Spec 007 — Multi-Agent Review & Voting

### Outcome

Synod introduces multi-agent councils to validate output before considering it done.

### Why now

When you start generating code automatically, you need serious quality control.

### In scope

- multiple reviewers
- different providers: GPT, Claude, Gemini, etc.
- structured findings
- voting: majority and weighted
- base adjudication
- triggers on:
- high risk
- failing validation
- PR generation

### Out of scope

- artifact governance, which remains in Canon
- full debate simulation

### Tangible result

Synod does not rely on a single model and produces more robust output.

## Recommended Sequence

1. Spec 006 — Execution Engine
2. Spec 007 — Review & Voting

## Resulting Architecture

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
  └───────────────┘
        ↓
     Canon
 (artifact + governance)
```

## In One Sentence

Synod must become a system that takes a problem and transforms it into working code, with multi-agent quality control.