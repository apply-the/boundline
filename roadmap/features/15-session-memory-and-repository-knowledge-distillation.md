# S21 - Session Memory And Repository Knowledge Distillation

## Owner

Boundline, with Canon project memory integration

## Status

B-level, after traces and project memory are stable

## Strategic Role

This feature prevents repeated rediscovery without creating ungoverned memory blobs.

Memory should be trace-linked, reviewable, and scoped.

## Problem

Long-running AI work loses knowledge across sessions:

- build commands
- test commands
- repo conventions
- known pitfalls
- accepted patterns
- previous failures
- environment assumptions
- useful trace conclusions

But uncontrolled memory is dangerous:

- transient errors become permanent lore
- stale assumptions persist
- secrets may be stored
- incorrect conclusions get reused

## Core Scope

- Confirmation-first memory writes
- Trace-linked memory entries
- Workspace-local repository knowledge notes
- No transient error logs as memory
- Memory classification
- Expiry or review status
- Canon promotion path for governed knowledge
- Inspect memory source and confidence

## Memory Types

### Operational Memory

Examples:

- build command
- test command
- formatting command
- local setup caveat

### Repository Convention

Examples:

- architecture pattern
- folder ownership
- test fixture style
- API error convention

### Known Pitfall

Examples:

- flaky test condition
- migration hazard
- generated file warning

### Candidate Canon Knowledge

Examples:

- domain term
- invariant
- architecture decision
- project standard

## Write Protocol

Before saving memory:

1. Summarize proposed memory.
2. Cite trace or evidence source.
3. Ask for confirmation unless policy allows auto-capture.
4. Classify memory type.
5. Set authority and expiry/review status.
6. Redact secrets.
7. Store with stable ID.

## Algorithms And Techniques

### Distillation

Use bounded summarization over traces:

- decisions
- commands
- failures
- fixes
- unresolved findings
- next actions

### Memory Hygiene

Reject:

- raw logs
- raw secrets
- one-off transient errors
- unverified assumptions
- emotional or conversational context

### Retrieval

Retrieve memory by:

- command intent
- lifecycle phase
- file path
- guidance pillar
- previous finding
- Canon project memory link

## Acceptance Criteria

- Boundline can propose memory entries from trace.
- User can accept/reject/edit before write.
- Memory entries cite trace refs.
- Memory does not store raw transient logs by default.
- Memory can be promoted or linked to Canon when governed.
- Inspect shows memory provenance and authority.
- Stale memory can be marked deprecated.

## Risks

- Memory becomes stale.
- Incorrect heuristics become "truth".
- Users skip review.
- Memory overlaps poorly with Canon project memory.

## Hard Rule

Memory is not governance. Canon governs knowledge.
