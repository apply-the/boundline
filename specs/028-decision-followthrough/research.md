# Research: Decision Continuity And Guided Follow-Through

**Feature**: 028-decision-followthrough  
**Date**: 2026-05-01

## R1: Project guided follow-through through existing status, next, and inspect surfaces

**Decision**: Represent the next bounded action and its supporting evidence on
the existing `status`, `next`, and `inspect` surfaces instead of creating a new
continuity command or a new persistence file.

**Rationale**: Operators already use those three commands to understand what
Synod will do next. Reusing them delivers immediate value without splitting the
read-side workflow or introducing a second authority for follow-up guidance.

**Alternatives Considered**:
- Add a dedicated continuity-inspection command: rejected because it would make
  operators learn another path for information that directly shapes current
  follow-up decisions.
- Persist a new standalone next-action file under `.synod/`: rejected because it
  would duplicate information that should stay derivable from session and trace
  state.

## R2: Prefer explicit evidence precedence between persisted session state and authoritative traces

**Decision**: Use one explicit continuity-evidence precedence rule: when the
native session remains authoritative, project its persisted decision continuity;
when continuity has shifted to an explicit compatibility trace, reuse the latest
authoritative trace evidence and make that authority visible.

**Rationale**: The core gap is not lack of evidence but unclear ownership of
which evidence should guide follow-up. Making the winning evidence source
explicit keeps Synod inspectable and avoids silent merging of stale session
state with newer trace facts.

**Alternatives Considered**:
- Always prefer session state: rejected because explicit compatibility follow-up
  can legitimately own the latest authoritative story.
- Always re-read only the trace: rejected because active native sessions already
  carry relevant continuity state and should not be demoted unnecessarily.

## R3: Reuse a compact follow-through projection instead of broadening the session model ad hoc

**Decision**: Introduce one compact follow-through projection that summarizes
the latest decision continuity, winning evidence source, guidance headline, and
bounded next action or stop condition for reuse across session and trace output.

**Rationale**: Current status and inspect surfaces already expose many narrow
fields. A compact shared projection is the smallest way to keep the next-action
story aligned without scattering more one-off render logic across session and
trace code paths.

**Alternatives Considered**:
- Add several new unrelated status fields: rejected because it would increase
  drift between session and trace rendering.
- Keep using only the generic lifecycle state machine: rejected because it does
  not explain why a particular follow-up action is credible.

## R4: Keep compatibility and clustered continuity boundaries explicit

**Decision**: When guided follow-through is projected for compatibility or
cluster-aware follow-up, preserve the existing continuity-authority and route-
ownership cues instead of synthesizing a native-session story.

**Rationale**: This feature should deepen follow-through, not blur ownership.
Guidance is only trustworthy if it still tells the operator whether the next
action comes from a live native session or an explicit compatibility trace.

**Alternatives Considered**:
- Normalize compatibility output to look fully native: rejected because it hides
  the real continuity boundary.
- Duplicate continuity ownership into cluster members: rejected because primary-
  workspace authority is already the bounded cluster model.

## R5: Close the slice as 0.28.0 with release-aligned validation

**Decision**: Treat version bump, impacted docs, assistant guidance, changelog,
touched-Rust coverage refresh, clippy cleanup, and formatting as first-class
tasks for the feature rather than post-hoc cleanup.

**Rationale**: The slice changes how operators interpret what Synod should do
next. The release must ship as one coherent story across runtime output,
assistant packs, documentation, and validation evidence.

**Alternatives Considered**:
- Defer release closeout until after implementation: rejected because it risks a
  mismatch between runtime behavior and operator guidance.
- Skip touched-file coverage refresh: rejected because the requested delivery
  discipline explicitly includes coverage for modified or created Rust files.