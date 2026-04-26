# Synod Roadmap

Canon is outside the scope of this roadmap: it is the runtime that persists structured artifacts. Synod is the system that thinks, decides, orchestrates, and executes.

## Objective

Evolve Synod into a system capable of taking a problem and transforming it into working code, with multi-agent quality control.

## Current Status

The `Spec 1.3 — Session & Interaction Model Unification`, `Spec 005 — Delivery Flows`, `Spec 006 — Execution Engine`, `Spec 007 — Multi-Agent Review & Voting`, and `Spec 008 — Adaptive Execution Engine` slices are now available in the local CLI.

- active session persisted in `.synod/session.json`
- explicit flow `start -> capture -> flow -> plan -> step/run -> status/next -> inspect`
- built-in `bug-fix`, `change`, and `delivery` flow definitions with stage-aware session state
- flow-aware `status`, `next`, `run`, and `inspect` output with stage transition and recovery traces
- execution-profile-backed red-to-green delivery under `.synod/execution.json` with legacy `.synod/fixture.json` fallback
- changed-file and validation evidence projected into `run`, `status`, and `inspect`
- bounded review councils with manifest-driven reviewers, majority or weighted voting, optional adjudication, and review evidence projected into `run`, `status`, `next`, and `inspect`
- adaptive workspace-slice selection, deterministic candidate synthesis, bounded replanning after failed validation, and adaptive evidence projected into `run`, `status`, `next`, and `inspect`
- assistant command packs aligned with the session model and reuse of `latest_trace_ref`

Immediate follow-up:

- broaden adaptive heuristics beyond the current deterministic local repair patterns
- deepen delivery and review beyond the current bounded local execution manifests

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

Synod introduces bounded multi-agent councils to validate output before considering it done.

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

Synod does not rely on a single model and produces more robust output, with review evidence persisted into the same local trace and session surfaces used by the delivery runtime.

## Spec 008 — Adaptive Execution Engine

### Outcome

Synod can choose a bounded workspace slice and synthesize repair attempts without requiring pre-authored attempt lists in the execution manifest.

### Why now

The execution engine needed to move beyond fixed red-to-green manifests while preserving the same bounded local runtime model.

### In scope

- workspace-slice scoring from `read_targets`
- deterministic adaptive candidate generation
- signature-based non-repeat behavior
- bounded replanning after failed validation
- adaptive evidence in `run`, `status`, `next`, and `inspect`

### Out of scope

- open-ended code generation
- distributed execution
- unbounded search or background planning

### Tangible result

Synod can recover from a failed first fix by selecting a new deterministic candidate and show exactly which slice and attempt lineage drove the final result.

## Recommended Sequence

1. Spec 006 — Execution Engine
2. Spec 007 — Review & Voting
3. Spec 008 — Adaptive Execution

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