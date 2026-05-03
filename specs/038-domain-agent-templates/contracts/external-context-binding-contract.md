# Contract: External Context Binding

**Feature**: 038-domain-agent-templates  
**Date**: 2026-05-03

## Purpose

Define how Canon-governed artifacts and bound external context inputs augment
the active domain guidance without taking ownership of template selection.

## Required Surface

- The effective applied domain context must surface Canon-governed artifacts as
  optional supporting inputs when governance is active.
- Bound external inputs must declare whether they are optional or required for a
  relevant domain family or task class.
- Planning and inspection surfaces must show whether each supporting input was
  used, skipped, unavailable, or stale.

## Explicit Boundaries

- Canon may augment the bounded context, but it must not decide the active
  domain template or standards precedence.
- Boundline must not claim to execute every possible MCP or external provider
  protocol in this slice; bindings are bounded references and status surfaces.
- A required supporting input must trigger an explicit blocked or downgraded
  outcome when it is unavailable rather than silently disappearing.