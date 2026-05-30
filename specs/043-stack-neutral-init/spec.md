# Feature Specification: Stack-Neutral Workspace Entry

**Feature Branch**: `043-stack-neutral-init`  
**Created**: 2026-05-06  
**Status**: Implemented  
**Input**: User description: "Add stack-neutral workspace entry so empty and non-Rust repositories can use the primary Boundline workflow, let init choose credible default models after the operator selects claude, copilot, codex, or gemini, and surface bounded technology-specific hygiene defaults for supported domain families."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start From A Stack-Neutral Workspace (Priority: P1)

An operator can point Boundline at an empty repository or a non-Rust repository
and enter the primary session-native workflow without first creating a
language-specific manifest, so Boundline can discover the task boundaries from
recorded goal text, authored briefs, and bounded repository evidence instead of
silently assuming one stack.

**Why this priority**: The primary operator path is not credible for a
multi-language product if empty or non-Rust repositories are rejected before
Boundline can even capture the task and decide what stack fits.

**Independent Test**: Run the primary `goal -> plan -> confirm ->
run` flow and the direct native entry flow on an empty repository, a Python
repository, and a Node repository. Each case must either proceed on the native
route or stop explicitly because the task context is not credible, without
requiring a Rust-specific prerequisite.

**Acceptance Scenarios**:

1. **Given** an empty repository with no language-specific manifest,
   **When** the operator starts a session and captures a bounded goal,
   **Then** Boundline accepts the workspace and evaluates the goal from captured
   inputs instead of failing readiness because one stack-specific file is absent.
2. **Given** a repository whose existing files indicate a non-Rust stack,
   **When** the operator uses the direct native entry flow,
   **Then** Boundline reaches planning on the native route without requiring a
   Rust-owned prerequisite.
3. **Given** an empty repository and a vague goal that does not provide enough
   product or stack evidence, **When** planning runs, **Then** Boundline stops
   with an explicit clarification or bounded credibility failure instead of
   silently defaulting to Rust.

---

### User Story 2 - Choose Assistant Target With Credible Model Defaults (Priority: P2)

An operator can initialize a workspace by naming an assistant target such as
Claude, Copilot, Codex, or Gemini and have Boundline seed a credible set of
default model routes for that assistant, so the workspace starts with a usable
routing baseline without making the operator discover model names manually.

**Why this priority**: Assistant-family selection is incomplete if operators
still have to guess model names or hand-author every route before the first
bounded run.

**Independent Test**: Initialize four fresh workspaces, one per supported
assistant target, without explicit model overrides. Verify that each workspace
persists deterministic slot routes and assistant bindings, and that operators
can inspect or override those defaults after initialization.

**Acceptance Scenarios**:

1. **Given** a fresh workspace and an assistant target selected during
   initialization, **When** the operator omits explicit model routes,
   **Then** Boundline persists one credible default model per required route slot
   for that assistant target and reports those selections on the standard init
   output.
2. **Given** a workspace initialized with default assistant-target models,
   **When** the operator inspects effective configuration,
   **Then** Boundline shows which defaults were selected automatically and which
   source made them authoritative.
3. **Given** an assistant target whose preferred default model is unavailable,
   **When** initialization runs, **Then** Boundline falls back explicitly to the
   next credible model for that assistant target or stops with an actionable
   correction instead of persisting a broken route.

---

### User Story 3 - Seed Bounded Hygiene Defaults By Selected Technology (Priority: P3)

An operator can let Boundline seed or verify technology-specific hygiene
defaults, such as ignore patterns and tool-adjacent exclusions, once the active
domain families or bounded plan make those defaults credible, so the initialized
workspace reflects the selected stack without dragging in irrelevant files from
other ecosystems.

**Why this priority**: Spec 038 promised first-party multi-language domain
coverage, but that promise stays shallow if Boundline cannot carry domain choice
through to the bounded workspace hygiene that real repositories need.

**Independent Test**: Initialize and plan representative repositories for at
least three supported stacks, including one mixed-stack example, then verify
that Boundline seeds only the relevant universal and technology-specific hygiene
defaults while leaving unrelated ecosystem patterns absent.

**Acceptance Scenarios**:

1. **Given** a repository whose active domain selection is Python plus Docker,
   **When** initialization or bounded setup hygiene runs, **Then** Boundline
   seeds the relevant universal, Python, and Docker defaults without adding
   Rust-only or Node-only defaults.
2. **Given** a mixed-stack repository with both backend and frontend domain
   families active, **When** Boundline prepares hygiene defaults, **Then** it
   combines only the credible defaults for the selected families and keeps the
   source of those defaults inspectable.
3. **Given** weak or contradictory domain evidence, **When** Boundline cannot
   determine whether a technology-specific hygiene pack is credible,
   **Then** it applies only universal defaults or stops explicitly instead of
   writing mismatched technology rules.

### Edge Cases

- What happens when a workspace is not under Git version control but is still a
  valid local directory for bounded session work?
- How does the system behave when an operator selects more than one assistant
  target or later overrides some, but not all, default model routes?
- What happens when the bounded task suggests one stack, but existing repository
  evidence suggests another and neither is yet credible enough to choose?
- How does the system surface the primary session-native route when an explicit
  compatibility execution profile also exists for the same workspace?
- What happens when a mixed-stack workspace gains a new domain family after
  initialization and existing hygiene defaults need to be extended without
  wiping local overrides?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept any existing writable local workspace for the
  primary session-native path without requiring a language-specific manifest as a
  generic readiness prerequisite.
- **FR-002**: System MUST keep the session-native `goal -> plan ->
  confirm -> run` path primary for empty, single-stack, and mixed-stack
  workspaces.
- **FR-003**: System MUST let the direct native entry flow bootstrap from the
  same stack-neutral workspace assumptions as the explicit session-native path.
- **FR-004**: System MUST stop planning explicitly when recorded goal text,
  authored briefs, and bounded repository evidence are insufficient to choose a
  credible stack or domain, instead of silently preferring Rust.
- **FR-005**: System MUST preserve and surface the reason a workspace is ready,
  blocked, or non-credible on standard readiness and follow-through surfaces.
- **FR-006**: System MUST let initialization accept one or more supported
  assistant targets and persist the assistant family selection in workspace-local
  configuration.
- **FR-007**: System MUST select a deterministic default model for each required
  route slot after an assistant target is chosen, unless the operator supplies
  an explicit model override.
- **FR-008**: System MUST keep a maintained default-model catalog for at least
  Claude, Copilot, Codex, and Gemini targets.
- **FR-009**: System MUST fall back explicitly when a preferred default model is
  unavailable, and MUST stop with an actionable correction when no credible
  model remains for the selected assistant target.
- **FR-010**: System MUST surface which route slots were filled from assistant
  defaults, which were explicitly overridden, and which source currently owns
  the effective route projection.
- **FR-011**: System MUST let initialization or bounded setup hygiene seed
  technology-specific defaults only after the selected domain families,
  repository evidence, or authored plan make those defaults credible.
- **FR-012**: System MUST support hygiene defaults for the first-party domain
  catalog delivered by spec 038, including universal defaults plus the major
  supported technology families.
- **FR-013**: System MUST support bounded tool-specific hygiene defaults when
  the selected domains or authored plan justify Docker, ESLint, Prettier,
  Terraform, Helm, or Kubernetes-related exclusions.
- **FR-014**: System MUST preserve the provenance of seeded hygiene defaults so
  operators can inspect whether each default came from universal policy,
  domain-family selection, repository evidence, or explicit override.
- **FR-015**: System MUST avoid overwriting local workspace overrides when new
  technology-specific defaults are added after initialization.
- **FR-016**: System MUST keep explicit compatibility execution subordinate to
  the stack-neutral session-native route while reusing the same workspace and
  guidance summary when compatibility becomes the continuity authority.

### Scope Boundaries *(mandatory)*

- **In Scope**: stack-neutral workspace readiness for the native route;
  assistant-target selection with deterministic default-model routing; bounded
  fallback or stop behavior for unavailable defaults; technology-specific
  hygiene defaults tied to selected domain families or credible evidence;
  inspectable provenance for route and hygiene selections; docs and version
  updates for the new operator surface.
- **Out of Scope**: marketplace discovery for arbitrary providers; automatic
  package-manager scaffolding for every stack; full project generation beyond
  bounded setup hygiene; UI redesign; deployment pipeline automation;
  third-party template packs outside the first-party domain catalog.

### Key Entities *(include if feature involves data)*

- **Workspace Entry Assessment**: The bounded readiness view for one local
  workspace, including whether the workspace is writable, whether the native
  route can start, and why planning may still stop for credibility reasons.
- **Assistant Target Profile**: The workspace-local selection of assistant
  family plus the deterministic default-model mapping that fills route slots
  unless the operator overrides them.
- **Hygiene Defaults Profile**: The inspectable set of universal,
  domain-specific, and tool-specific workspace hygiene defaults that Boundline
  seeds or verifies for the active repository.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of representative empty, Python, Node, and mixed-stack test
  workspaces can enter the primary native planning path without adding a
  Rust-specific manifest first.
- **SC-002**: 100% of native planning attempts on under-specified empty
  workspaces stop with an explicit credibility or clarification outcome instead
  of silently selecting Rust.
- **SC-003**: 100% of init runs that specify Claude, Copilot, Codex, or Gemini
  without explicit routes persist deterministic default-model selections that are
  visible on init output and effective config inspection.
- **SC-004**: In representative stack-specific hygiene tests, Boundline adds no
  irrelevant technology pack more than 5% of the time and always preserves
  operator-authored overrides.

## Assumptions

- Operators may start from repositories that are empty, partially scaffolded,
  or already contain one or more supported stacks.
- The first delivery slice may use a repository-managed default-model catalog
  rather than performing live marketplace discovery.
- Technology-specific hygiene defaults are bounded setup aids, not a promise to
  generate full production project scaffolds for every supported stack.
- Existing routing, domain-template, and continuity surfaces remain the primary
  inspection path for the new defaults rather than adding a separate UI surface.
