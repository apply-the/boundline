# Contract: Applied Domain Context

**Feature**: 038-domain-agent-templates  
**Date**: 2026-05-03

## Purpose

Define how Boundline selects, persists, and projects the active domain guidance for
bounded planning and execution.

## Required Surface

- `plan` must either produce a credible applied domain context for the bounded
  task or stop explicitly because domain guidance is insufficient.
- `run`, `status`, `next`, and `inspect` must be able to name the active domain
  family or family combination, the winning standards source, and the bounded
  target that triggered the selection.
- Replanning or task-target changes in a mixed-stack repository must be able to
  update the applied domain context without creating a second planning system.

## Explicit Boundaries

- Domain selection must remain bounded by repository evidence, selected targets,
  and the captured goal rather than open-ended persona switching.
- The surface must not silently fall back to an unrelated domain family when no
  credible match exists.
- Compatibility follow-up may reuse the same summary vocabulary, but it must not
  become the primary owner of domain-template selection.