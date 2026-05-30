# Feature Specification: Distribution & Bundling

**Feature Branch**: `039-distribution-bundling`  
**Created**: 2026-05-03  
**Status**: Draft  
**Input**: User description: "Implement distribution and bundling with Homebrew and winget support, bundled Canon installation and unified updates, while restructuring Boundline docs around a brutal quick path and a separate advanced architecture story that keeps Boundline and Canon clearly separated."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install Boundline Without Toolchain Friction (Priority: P1)

An operator on a supported machine can install Boundline from an official system
channel, receive the Canon runtime alignment needed by the documented Boundline
surface, and reach the first bounded session-native command path without having
to clone the repository or assemble the toolchain by hand.

**Why this priority**: Distribution is the whole point of this macrofeature.
If the user still has to build from source and manually coordinate Canon, the
product remains harder to adopt than it should be.

**Independent Test**: On a fresh supported macOS or Windows machine, install
Boundline from the official channel, verify the CLI is runnable, verify the Canon
companion state is acceptable, and complete the first `doctor -> start ->
goal -> plan -> run` flow without source-build steps.

**Acceptance Scenarios**:

1. **Given** a supported macOS machine without Boundline installed, **When** the
   operator installs Boundline from the official macOS channel, **Then** Boundline is
   runnable from the shell and the operator can verify whether the Canon
   companion is already aligned or was installed alongside it.
2. **Given** a supported Windows machine without Boundline installed, **When** the
   operator installs Boundline from the official Windows channel, **Then** Boundline is
   runnable from the shell and the documented first-run path does not require a
   repository clone.
3. **Given** a supported install attempt where the required Canon companion
   cannot be made available, **When** the operator completes installation,
   **Then** Boundline ends in an explicit blocked or repair-needed state instead of
   silently appearing ready.

---

### User Story 2 - Keep Boundline And Canon Aligned Through Updates (Priority: P2)

An operator who already has Boundline installed can update through the same
official channel and keep the Boundline plus Canon pairing coherent, with explicit
repair guidance when a partial upgrade, drift, or unsupported pairing occurs.

**Why this priority**: Distribution is incomplete if installation is easy but
maintenance is fragile. Users need one bounded update story, not a fresh manual
compatibility exercise every release.

**Independent Test**: Start from one older supported install, run the official
update flow, then verify that the resulting Boundline and Canon state is either
fully usable or explicitly repairable from the CLI and docs without guessing.

**Acceptance Scenarios**:

1. **Given** a machine with an older supported Boundline release, **When** the
   operator updates through the official channel, **Then** Boundline lands in one
   bounded state: ready with a supported Canon pairing, or blocked with a clear
   next action.
2. **Given** a machine where Boundline and Canon versions drift outside the
   documented support window, **When** the operator runs the documented
   verification path after update, **Then** the system explains the mismatch
   and the repair route instead of leaving the user to infer compatibility.

---

### User Story 3 - Learn The Product In Two Read Levels (Priority: P3)

An operator opening the docs for the first time can first see a brutal quick
path for installation and first bounded work, then separately read a clearer
advanced architecture explanation that preserves the distinction between Boundline
as the operational runtime and Canon as the governed runtime.

**Why this priority**: The product framing is now strong, but the current docs
can feel intimidating. This slice needs the adoption path and the architecture
story to be equally coherent without forcing new users through the whole model
up front.

**Independent Test**: Ask a new reader to follow the docs from a cold start,
find the install path, find the first-run path, and then explain the Boundline vs
Canon boundary from the advanced material without reading the whole README end
to end.

**Acceptance Scenarios**:

1. **Given** a new operator who wants to get started quickly, **When** they
   open the primary docs, **Then** they can find installation and the first-run
   session-native command path before advanced routing, cluster, governance, or
   workflow details.
2. **Given** a reader who wants architecture context after the quick path,
   **When** they continue into the advanced material, **Then** the docs explain
   that Boundline decides, executes, validates, and owns session state while Canon
   governs, records, approves, and publishes structured artifacts.

---

### User Story 4 - Publish One Coherent Release Surface (Priority: P4)

A maintainer can ship one Boundline release that updates versioning, package
channel metadata, release narrative, and compatibility guidance together,
instead of treating distribution, Canon alignment, and product messaging as
separate cleanup work.

**Why this priority**: The user-facing install story and the maintainer-facing
release story have to land together or the channels drift immediately.

**Independent Test**: Prepare one release candidate, refresh the official
distribution surfaces and release docs, then confirm that the package channels,
version metadata, compatibility guidance, changelog, and roadmap all describe
the same release.

**Acceptance Scenarios**:

1. **Given** a new Boundline release candidate, **When** the maintainer prepares
   the official distribution update, **Then** the release surface names the
   same Boundline version, Canon expectation, and install or update story across
   package channels and docs.
2. **Given** one official channel is not yet ready for publication, **When**
   the maintainer closes the release, **Then** the release story makes that
   state explicit instead of implying that every supported install route is
   already available.

### Edge Cases

- What happens when the operator is on an unsupported platform or unsupported
  machine architecture for the official channels?
- How does the system behave when Boundline installs successfully but the Canon
  companion cannot be installed, discovered, or upgraded into the documented
  support window?
- What happens when a user follows the quick path docs but later needs routing,
  workflow, cluster, or governance detail that belongs in the advanced layer?
- How does the release story surface partial channel readiness so the roadmap,
  changelog, and install docs do not overclaim availability?
- How does the primary install and quick-run path stay visibly session-native
  while still acknowledging explicit compatibility behavior as secondary?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide official end-user distribution through the
  roadmap-promised macOS and Windows package channels.
- **FR-002**: System MUST provide a documented install path that leaves the
  operator with a runnable Boundline CLI and an explicit Canon companion state:
  ready, already satisfied, blocked, or repair-needed.
- **FR-003**: System MUST let operators verify the installed Boundline version,
  the documented Canon support expectation, and whether the current machine is
  inside that bounded support story.
- **FR-004**: System MUST provide one bounded update story for installed users
  so Boundline and Canon can move forward together or surface explicit repair
  guidance when they cannot.
- **FR-005**: System MUST make installation and update failures explicit rather
  than silently leaving the user in an apparently usable but unsupported state.
- **FR-006**: System MUST keep the session-native path as the primary operator
  journey after installation, with explicit compatibility behavior remaining
  secondary in the docs and release story.
- **FR-007**: System MUST restructure the primary documentation into at least
  two readable levels: a quick path for install plus first-run bounded work,
  and a separate advanced architecture explanation.
- **FR-008**: System MUST explain Boundline and Canon responsibilities consistently
  across README, practical getting-started material, assistant guidance, and
  release narrative.
- **FR-009**: System MUST let maintainers refresh versioning, official channel
  metadata, compatibility guidance, and release notes as one bounded release
  activity.
- **FR-010**: System MUST make channel availability explicit in the release
  story so unsupported or not-yet-published routes are not implied to be ready.
- **FR-011**: System MUST preserve a source-based fallback path for users who
  are outside the official distribution channels.
- **FR-012**: System MUST include validation that covers fresh install,
  upgrade, version or compatibility verification, non-ready bundle states, and
  documentation coherence for the quick path plus advanced path split.

### Scope Boundaries *(mandatory)*

- **In Scope**: official Homebrew and Windows Package Manager distribution,
  Canon companion alignment during install and update, explicit install or
  repair verification, documentation restructuring into quick and advanced
  paths, version bump and release narrative alignment, roadmap closure, and
  validation of the resulting distribution story.
- **Out of Scope**: Linux package-manager coverage beyond existing source-based
  installation, a standalone GUI installer, remote hosted service deployment,
  full Canon mode expansion, package channels beyond the roadmap-promised set,
  or a generalized plugin marketplace for external runtimes.

### Key Entities *(include if feature involves data)*

- **Distribution Channel**: an official user-facing install and update route
  for Boundline on one supported platform, including its published package
  metadata, version, and availability status.
- **Runtime Pairing State**: the bounded statement of whether the installed
  Boundline and Canon combination is ready, already satisfied, blocked, or
  repair-needed for the documented support window.
- **Release Surface**: the combined user-facing story for one Boundline release,
  including version metadata, official package updates, compatibility guidance,
  roadmap state, and changelog narrative.
- **Documentation Path**: one of the two intended read levels for users,
  consisting of the quick install plus first-run path and the advanced
  architecture explanation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On supported macOS and Windows machines, operators can install
  Boundline from an official channel and reach a runnable CLI plus explicit Canon
  pairing state in under 10 minutes without cloning the repository.
- **SC-002**: 100% of representative fresh-install and update runs on
  supported channels end in an explicit ready, blocked, or repair-needed state;
  none fail silently.
- **SC-003**: New readers can find the install plus first-run quick path in
  under 2 minutes without first reading advanced routing or governance detail.
- **SC-004**: In documentation review, readers can correctly explain the Boundline
  versus Canon boundary after reading the advanced architecture section in under
  5 minutes.
- **SC-005**: Maintainers can prepare one release update that keeps package
  metadata, roadmap state, changelog, and compatibility guidance aligned to the
  same Boundline version with no unresolved public mismatch at release close.

## Assumptions

- The official distribution slice is limited to macOS and Windows because those
  are the roadmap-promised channels for this release.
- Canon remains a separately versioned runtime, but this slice may install it
  alongside Boundline or verify an already installed compatible version.
- Source-based installation remains available as the fallback path for users
  outside the official package channels.
- Users have network access to reach the official package channels during fresh
  install or update.
- The primary product story after installation remains the session-native Boundline
  route, not a compatibility-first or Canon-first entrypoint.
