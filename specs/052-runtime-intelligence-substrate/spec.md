# Feature Specification: Runtime Intelligence Substrate

**Feature Branch**: `052-runtime-intelligence-substrate`  
**Created**: 2026-05-14  
**Status**: Implemented  
**Input**: User description: "Implement a local runtime substrate for Boundline that builds local runtime indexes, deterministic context packs, runtime state, and explainable trace projection, while consuming Canon artifacts only as optional enrichment and stopping explicitly when credible context cannot be built."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build A Credible Context Pack Before Planning (Priority: P1)

As a Boundline operator running the primary session-native workflow, I want
Boundline to build a deterministic context pack from local repository signals
before planning starts, so planning uses explicit context instead of keyword
guesswork.

**Why this priority**: If Boundline cannot assemble a credible context pack,
every later planning and execution decision becomes less trustworthy.

**Independent Test**: In a representative workspace, run the session-native
flow through `goal` and `plan` and verify that Boundline can expose the
constructed context pack, including matched files, selected evidence signals,
and missing-context warnings, without requiring Canon.

**Acceptance Scenarios**:

1. **Given** a workspace with enough local repository signals,
   **When** Boundline builds the runtime substrate before planning,
   **Then** it assembles a deterministic context pack that records the matched
   systems, relevant files, evidence signals, and current runtime state.
2. **Given** the same workspace and goal,
   **When** Boundline repeats the same substrate build without relevant repo
   changes, **Then** the resulting context pack is materially the same and its
   provenance remains inspectable.
3. **Given** Canon is unavailable,
   **When** Boundline builds the substrate from local signals,
   **Then** the session-native path remains functional and explicit about the
   absence of Canon enrichment.

---

### User Story 2 - Stop Explicitly When Context Is Not Credible (Priority: P1)

As a Boundline maintainer, I want the runtime substrate to classify missing or
non-credible context explicitly, so the system stops, warns, or replans instead
of continuing on hidden assumptions.

**Why this priority**: Failure to classify missing context turns planning into
guesswork and breaks the delivery-first trust model.

**Independent Test**: Present Boundline with missing-doc, missing-artifact,
unsupported-surface, and incomplete-context scenarios and verify that it enters
an explicit warning, replan, or terminal behavior instead of proceeding
silently.

**Acceptance Scenarios**:

1. **Given** a goal whose required context cannot be built credibly from local
   repo signals, **When** Boundline evaluates the substrate, **Then** it stops
  in an explicit insufficient or terminal path before planning continues.
2. **Given** partial but still usable context,
   **When** Boundline evaluates the substrate, **Then** it emits explicit
  warning or refresh guidance instead of treating the context as fully
  credible.
3. **Given** an invalid or stale substrate artifact,
   **When** Boundline reads it, **Then** it records the failure path and does
   not lose required session state.

---

### User Story 3 - Inspect Substrate Decisions Through Runtime Surfaces (Priority: P2)

As a reviewer or operator, I want `status`, `next`, and `inspect` to surface
why the substrate selected specific repository and Canon inputs, so I can
understand context assembly without reading internal code.

**Why this priority**: Inspectability is the trust boundary for a runtime that
claims explicit context assembly and bounded failure behavior.

**Independent Test**: Run a bounded session that builds a context pack and
verify that `status`, `next`, and `inspect` explain the selected inputs,
missing-context warnings, and any Canon enrichment used.

**Acceptance Scenarios**:

1. **Given** a successful substrate build,
   **When** a user inspects session state,
   **Then** the runtime surfaces the context sources, selected repository cues,
   and any Canon enrichment that influenced the pack.
2. **Given** a non-credible context path,
   **When** a user inspects the session,
   **Then** the runtime surfaces why planning stopped or downgraded and which
   evidence was missing.
3. **Given** a session that used local-only context,
   **When** a user inspects the trace,
   **Then** the runtime distinguishes local substrate reasoning from optional
   Canon-derived enrichment.

### Edge Cases

- A workspace has enough code signals to continue but no explicit project index
  file, so Boundline must decide whether to warn or proceed with local context.
- Canon metadata is available but incompatible with the currently supported
  contract line, so the substrate must ignore Canon enrichment without losing
  the local-only path.
- A prior substrate artifact is stale after repository changes and can no longer
  be reused safely.
- The same goal touches multiple plausible systems and the substrate cannot pick
  one credibly from current evidence.
- A session reaches planning with an incomplete context pack and must stop or
  replan before execution begins.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Boundline MUST build and persist a local runtime substrate that
  includes explicit runtime indexes, runtime state, and a deterministic context
  pack before planning continues.
- **FR-002**: Boundline MUST support a local-only substrate path that remains
  functional even when Canon artifacts are unavailable or unusable.
- **FR-003**: Boundline MUST treat Canon artifacts as optional enrichment to the
  substrate rather than as authoritative runtime state.
- **FR-004**: Boundline MUST classify context credibility explicitly before
  planning with stable state values of `credible`, `stale`, or `insufficient`,
  while deriving explicit warning, refresh, replan, or terminal behavior from
  those state values.
- **FR-005**: Boundline MUST map non-credible context to explicit stop,
  refresh, or replan behavior rather than continuing on implicit assumptions.
- **FR-006**: Boundline MUST preserve enough runtime state that later steps can
  understand how the current context pack was assembled and why earlier context
  choices were made.
- **FR-007**: Boundline MUST expose substrate-visible reasoning through
  inspectable runtime surfaces including status, next, and inspect.
- **FR-008**: Boundline MUST record both successful and non-success substrate
  paths in traces, including missing-context warnings and terminal reasons.
- **FR-009**: Boundline MUST distinguish local repository inputs from Canon
  enrichment in substrate traces and inspection surfaces.
- **FR-010**: Boundline MUST remain sequential-first in substrate construction,
  with one active context-assembly path at a time.
- **FR-011**: Boundline MUST NOT introduce council policy, adaptive governance
  progression, or advanced reasoning profiles in this slice.
- **FR-012**: Boundline MUST allow later role-composition and governance layers
  to consume substrate-visible inputs without redefining them.

### Scope Boundaries *(mandatory)*

- **In Scope**: local runtime indexes, deterministic context packs, runtime
  state, local-only substrate operation, optional Canon enrichment,
  missing-context classification, and substrate explainability through traces
  and CLI surfaces.
- **Out of Scope**: pack manifests, workspace override precedence, council
  profiles, voting, adaptive governance behavior, advanced reasoning profiles,
  distributed execution, UI work, and deployment pipelines.

### Key Entities *(include if feature involves data)*

- **Runtime Index**: A local machine-readable index of project, evidence,
  symbol, and runtime-state signals used by substrate construction.
- **Context Pack**: The deterministic bundle of selected repository cues,
  Canon enrichment, warnings, and runtime facts used as input for planning.
- **Context Credibility Outcome**: The explicit classification of whether the
  current context is `credible`, `stale`, or `insufficient`, plus the
  operator-visible behavior derived from that state.
- **Substrate Trace Projection**: The inspectable runtime output that explains
  why context inputs were selected or rejected.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative bounded-delivery sessions, Boundline can build
  an inspectable context pack before planning without requiring Canon.
- **SC-002**: 100% of non-credible substrate runs stop, warn, or replan in an
  explicit state rather than silently continuing.
- **SC-003**: A maintainer can identify why a context pack was assembled or
  rejected from runtime surfaces in under 5 minutes.
- **SC-004**: Re-running the same local substrate build against unchanged
  inputs produces materially consistent context-pack output apart from
  generated ids, timestamps, and trace-write metadata.

## Catalog Research & Currency *(mandatory)*

- **Public Sources Reviewed**: OpenAI Models documentation at
  `https://developers.openai.com/api/docs/models`, Anthropic Models Overview at
  `https://platform.claude.com/docs/en/docs/about-claude/models`, and Google
  Gemini Models documentation at `https://ai.google.dev/gemini-api/docs/models`
  on 2026-05-14.
- **Catalog Delta**: No bundled catalog changes are required for this feature
  slice based on the spec-time audit.
- **No-Change Rationale**: The current bundled catalog in
  `assistant/catalog/model-catalog.toml` already matches the public text-and-
  coding model families relevant to Boundline runtime selection for this slice:
  OpenAI documents `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano`;
  Anthropic documents `claude-opus-4-7`, `claude-sonnet-4-6`, and
  `claude-haiku-4-5`; Google documents `gemini-2.5-pro`, `gemini-2.5-flash`,
  `gemini-2.5-flash-lite`, `gemini-3.1-pro-preview`, and
  `gemini-3.1-flash-lite`. Newly listed audio, media, and specialized research
  models do not change the runtime substrate contract for this feature.

## Assumptions

- The primary product path remains the session-native workflow and substrate
  behavior is evaluated there first.
- Canon enrichment is optional and may be absent, blocked, or incompatible
  without invalidating the local runtime substrate.
- Existing project-memory and project-index surfaces provide enough precedent to
  introduce substrate indexes without inventing a second runtime state system.
- This slice stops before pack composition, governance adaptation, or advanced
  reasoning and leaves those concerns to later layered features.
