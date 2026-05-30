# Feature Specification: Domain Agent Templates

**Feature Branch**: `038-domain-agent-templates`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Support base agent templates with project-specific overrides, init-time customization, major language and framework specialists, and later refinement of company rules."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Apply The Right Domain Expert Per Task (Priority: P1)

An operator can run the primary session-native path on a mixed-stack repository
and have Boundline choose the most relevant domain expert for the current bounded
task, so the resulting plan and code guidance reflect the affected language,
framework, and project standards instead of a generic assistant style.

**Why this priority**: This is the core operating-model change. If Boundline still
approaches repository work with one generic expert, the rest of the feature is
just configuration without delivery value.

**Independent Test**: Use a repository that contains multiple supported stacks,
run the primary `goal -> plan -> run` flow against different task targets,
and verify that Boundline identifies and surfaces the matching domain expert before
the first modifying step.

**Acceptance Scenarios**:

1. **Given** a repository with a Rust service and a React application,
   **When** the operator runs a bounded task against the Rust service,
   **Then** Boundline applies the Rust-focused domain guidance as the primary
   expert for that task and shows that choice on the standard operator path.
2. **Given** a bounded task that moves from backend files to frontend files,
   **When** Boundline replans or advances to the next bounded step,
   **Then** it updates the applied domain guidance to match the new target
   instead of reusing one generic expert across the whole session.

---

### User Story 2 - Apply Layered Template Inheritance (Priority: P2)

An operator can initialize a workspace from Boundline's base domain templates,
optionally inherit reusable company-wide standards shared across repositories,
and then refine those standards with project-specific rules without polluting
other workspaces through a clear base-template, shared-standard, and
workspace-override layering model.

**Why this priority**: The feature is incomplete if standards can only live in
one place. Platform teams need one reusable baseline, while each repository
still needs local context, terminology, architecture, and exceptions. This is
the story that defines how shared defaults and local overrides coexist without
cross-repository leakage.

**Independent Test**: Initialize two workspaces from the same shared standards,
add different local rules in each one, and verify that later bounded tasks keep
the shared baseline but apply only the local overrides for the active
repository.

**Acceptance Scenarios**:

1. **Given** a new workspace that opts into shared company standards,
   **When** the operator selects supported domain families during
   initialization,
   **Then** Boundline prepares project guidance by combining the default domain
   expertise with the shared standards before applying any workspace-specific
  overrides.
2. **Given** two workspaces that share the same reusable standards,
   **When** one workspace changes its local rules after initialization,
   **Then** later planning and execution reflect that local change only for the
   edited workspace without changing the other workspace's guidance.

---

### User Story 3 - Keep Domain Coverage Bounded And Inspectable (Priority: P3)

An operator can see which domain experts Boundline supports, which ones are active
for the current project, and when Boundline cannot select a credible expert for the
current bounded task.

**Why this priority**: Broad domain coverage is only credible if operators can
inspect the support story and if unsupported or ambiguous work stops explicitly
instead of silently drifting into low-quality output.

**Independent Test**: Inspect a workspace with declared domain support, run one
task inside the supported catalog and one outside it, and verify that Boundline
either names the active expert clearly or stops with an explicit bounded gap.

**Acceptance Scenarios**:

1. **Given** a workspace with supported language, framework, and project
   standards already selected,
   **When** the operator inspects the active task context,
   **Then** Boundline shows which domain expert families and project rules are
   shaping the current bounded step.
2. **Given** a task whose target files do not match any supported domain or
   match conflicting domain signals,
   **When** Boundline cannot identify a credible expert,
   **Then** Boundline stops or falls back explicitly with a bounded explanation
   instead of silently applying mismatched guidance.

---

### User Story 4 - Reuse Governed And External Context Inputs (Priority: P4)

An operator can let Boundline enrich the active domain expert with approved
governed artifacts and optional external context inputs, such as design
references, design-system guidance, or token sources, when those inputs are
relevant to the bounded task.

**Why this priority**: This is where Canon and external tool ecosystems add
value, but they must stay subordinate to Boundline's bounded execution model.
They should improve domain specificity without becoming the owner of template
selection or project customization.

**Independent Test**: Run a bounded frontend or mobile task once with approved
governed standards and relevant external context inputs available, and once
without them. Boundline must surface the extra inputs when present and continue or
stop explicitly when a required input is unavailable.

**Acceptance Scenarios**:

1. **Given** a workspace with governance enabled and approved standards
  artifacts relevant to a frontend task,
  **When** Boundline plans or runs that bounded task,
  **Then** it reuses those governed artifacts as supporting inputs without
  letting Canon replace the active domain template or workspace rules.
2. **Given** a workspace with external context inputs bound to a relevant
  domain expert,
  **When** Boundline assembles the bounded task context,
  **Then** it surfaces whether those inputs were used, unavailable, stale, or
  skipped instead of silently assuming they were present.

### Edge Cases

- What happens when one bounded step matches both a language expert and a
  framework expert, but the project-specific rules contradict one of them?
- How does the system behave when a workspace was initialized for one stack,
  but the repository later grows into a multi-stack project that needs more
  active domain experts?
- What happens when a task touches generated, vendor, or unsupported files that
  should not inherit the same rules as first-party repository code?
- What happens when a design, design-system, or token context source is bound
  to a domain expert but is unavailable, stale, or inconsistent with the local
  project rules?
- How does the system surface the chosen domain expert on the primary
  session-native path versus an explicit compatibility follow-up that later
  becomes the authoritative continuity source?

## Requirements *(mandatory)*

### Supported Domain Catalog

- **Template Family 1 - Systems Expert**: Rust, Go, C, C++, and Zig delivery
  work.
- **Template Family 2 - JVM Service Expert**: Java, Kotlin, and Spring-based
  service work.
- **Template Family 3 - .NET Service Expert**: C# and ASP.NET Core service
  work.
- **Template Family 4 - Python Service Expert**: Python, Django, and FastAPI
  service work.
- **Template Family 5 - Node Service Expert**: JavaScript and TypeScript
  service work, including Express- and Nest-style application patterns.
- **Template Family 6 - Web UI Expert**: JavaScript and TypeScript client-side
  application work.
- **Template Family 7 - React Expert**: React and Next.js user-interface and
  full-stack web work.
- **Template Family 8 - Vue Expert**: Vue and Nuxt-style application work.
- **Template Family 9 - Angular Expert**: Angular application work.
- **Template Family 10 - Ruby Expert**: Ruby and Rails application work.
- **Template Family 11 - PHP Expert**: PHP and Laravel application work.
- **Template Family 12 - Data Expert**: Python data and machine-learning
  workflows plus SQL-centric analytics work.
- **Template Family 13 - Mobile Expert**: Swift, SwiftUI, Kotlin Android,
  Flutter, and React Native application work.

### Functional Requirements

- **FR-001**: System MUST provide a declared first-party catalog of domain
  experts for every template family listed in the Supported Domain Catalog.
- **FR-002**: System MUST let one bounded task inherit both general
  language-level guidance and more specific framework-level guidance when both
  are relevant to the same target.
- **FR-003**: System MUST let maintainers define reusable cross-workspace
  standards for any supported domain expert so multiple repositories can start
  from one shared baseline.
- **FR-004**: System MUST let operators choose the supported domain experts
  that are active for a workspace during project initialization.
- **FR-005**: System MUST let operators define workspace-specific coding,
  formatting, style, and architectural standards for any active domain expert
  during initialization.
- **FR-006**: System MUST let operators revise workspace-specific standards
  after initialization and apply those revisions to later bounded tasks.
- **FR-007**: System MUST apply domain guidance through an explicit precedence
  order: Boundline default template, reusable shared standards, and then
  workspace-specific standards, with the narrower scope winning on conflicts.
- **FR-008**: System MUST give workspace-specific standards precedence over
  broader defaults while preserving broader guidance for uncovered areas.
- **FR-009**: System MUST evaluate the bounded task context and select the most
  relevant active domain expert for planning, execution, review, and
  verification on the primary session-native route.
- **FR-010**: System MUST support mixed-stack repositories by allowing
  different bounded steps in the same session to use different domain experts
  as task targets change.
- **FR-011**: System MUST preserve which domain experts, shared standards, and
  workspace-specific standards shaped the current bounded step so operators can
  inspect that decision later.
- **FR-012**: System MUST expose the supported domain catalog, the workspace's
  active selections, and the currently applied guidance sources through the
  standard configuration and execution-inspection surfaces.
- **FR-013**: System MUST let Canon-approved governed artifacts augment the
  current bounded task as optional supporting inputs when governance is active,
  without making Canon the owner of template selection or workspace
  customization.
- **FR-014**: System MUST let operators bind external context inputs to
  relevant domain experts so bounded tasks can reuse repository-adjacent
  references such as design, design-system, platform, or token guidance when
  those inputs are available.
- **FR-015**: System MUST let a workspace declare whether a bound external
  context input is optional or required for a given domain or task class.
- **FR-016**: System MUST preserve and surface whether governed artifacts and
  bound external context inputs were used, unavailable, stale, or skipped for
  the current bounded step.
- **FR-017**: System MUST stop, block, or downgrade explicitly when no
  credible active domain expert can be selected for the current bounded task,
  or when a required supporting input is unavailable, instead of silently
  applying mismatched guidance.
- **FR-018**: System MUST keep domain-expert selection bounded by the captured
  goal, current plan, targeted repository evidence, and route ownership instead
  of turning the feature into open-ended persona switching.
- **FR-019**: System MUST keep explicit compatibility follow-up secondary to
  the session-native route while reusing the same domain-guidance summary when
  compatibility state later becomes authoritative.
- **FR-020**: System MUST let maintainers update the declared support catalog
  without forcing existing workspaces to discard their shared or
  workspace-specific standards.
- **FR-021**: System MUST include validation that covers supported-domain
  selection, standards precedence, post-initialization customization,
  mixed-stack task switching, governed artifact reuse, external-context-input
  handling, unsupported-domain handling, and operator inspectability.

### Scope Boundaries *(mandatory)*

- **In Scope**: a first-party catalog of domain experts for the listed major
  language and framework families; reusable shared standards across
  repositories; project initialization for active domain selection;
  workspace-specific overrides for coding, formatting, style, and architectural
  rules; later customization of those rules; optional Canon-governed standards
  reuse; optional external context inputs for relevant domains; bounded domain
  selection during planning and execution; inspectable domain-guidance
  projection on the existing operator surfaces.
- **Out of Scope**: operating-system package distribution and bundled
  installation; third-party template marketplaces; niche ecosystem coverage
  outside the declared catalog; making Canon mandatory for template selection;
  authoring or operating every possible external context source;
  autonomous background multi-agent fan-out; review councils and voting; UI
  redesign; deployment pipeline automation.

### Key Entities *(include if feature involves data)*

- **Domain Template Family**: Boundline's default expert guidance for one supported
  language or framework family, including the standards and bounded delivery
  expectations associated with that family.
- **Shared Standards Overlay**: the reusable cross-workspace standards pack
  that lets an organization apply one baseline to multiple repositories.
- **Project Standards Overlay**: the workspace-specific set of company or team
  rules that refine or override the default guidance and any shared standards
  for active domain experts.
- **Active Domain Selection**: the workspace-owned declaration of which domain
  template families are enabled for the current project.
- **Applied Domain Context**: the inspectable record of which domain experts
  and guidance sources shaped the current bounded task or step.
- **Governed Standards Artifact**: the optional Canon-approved artifact that
  can contribute bounded guidance or constraints to the active task when
  governance is enabled.
- **External Context Input Binding**: the workspace-declared association
  between a domain expert and an external source of bounded context that may be
  optional or required for certain task categories.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In representative repositories across every supported template
  family, operators can initialize the relevant domain support and capture
  project-specific standards in under 15 minutes without writing a full expert
  guide from scratch.
- **SC-002**: In representative supported tasks, at least 95% of bounded
  sessions surface the expected domain expert before the first code-modifying
  step.
- **SC-003**: In representative mixed-stack tasks, operators can identify the
  active language, framework, and project-rule context for the current bounded
  step in under 2 minutes using standard Boundline output.
- **SC-004**: In representative override-conflict scenarios, 100% of
  project-specific standards take precedence over default domain guidance while
  uncovered areas still retain the matching default guidance.
- **SC-005**: In unsupported or ambiguous tasks, 100% of sessions stop or
  surface an explicit fallback explanation rather than silently applying an
  unrelated domain expert.
- **SC-006**: In representative governed or design-sensitive tasks, operators
  can identify in under 2 minutes whether approved governed artifacts and bound
  external context inputs influenced the active bounded step.

## Assumptions

- The primary operator path remains the session-native route; explicit
  compatibility follow-up stays secondary but must reuse the same domain
  guidance summary when it owns the latest authoritative state.
- Canon remains optional and downstream in this slice; when governance is
  enabled it contributes approved supporting artifacts, but Boundline still owns
  template selection, precedence, and bounded task routing.
- Boundline-maintained default guidance covers the declared first-party catalog in
  this slice; adding new external or community-maintained template families is
  a future decision, not part of the initial macrofeature.
- Teams are willing to select active domain experts during setup and to keep
  shared and workspace-specific rules up to date as repository conventions
  evolve.
- Distribution work such as Homebrew, winget, or bundled Canon installation is
  handled by the next roadmap macrofeature rather than this slice.
