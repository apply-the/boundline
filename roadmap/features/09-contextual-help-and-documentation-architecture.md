# S12 - Contextual Help And Documentation Architecture

## Owner

Canon and Boundline

## Status

A-level product adoption feature

## Strategic Role

This feature makes the system teachable.

BMAD's advantage is not only methodology. It is that the user knows what to do next. Canon and Boundline need the same operational friendliness without losing rigor.

## Problem

Current documentation and command surfaces are too power-user oriented.

Users need answers to:

- where am I?
- what is missing?
- what should I do next?
- why is the system blocked?
- what command should I run?
- which document should I read?
- which mode/workflow applies?

## Core Scope

### Boundline `help-next`

Should inspect:

- workspace initialized?
- assistant package installed?
- active session?
- current lifecycle phase?
- missing config?
- missing provider route?
- missing Context Pack?
- failed guardians?
- active stop rules?
- available recovery?

Should output:

- current state
- next recommended action
- exact command
- why this action
- docs/wiki link
- risks or prerequisites

### Canon `help-next`

Should inspect:

- selected mode?
- packet exists?
- ordered documents present?
- missing required docs?
- evidence present?
- readiness state?
- approval state?
- lineage complete?
- promotion blocked?

Should output:

- current governance state
- missing artifacts
- next recommended action
- exact mode or command
- docs/wiki link
- warning if downstream use is unsafe

## Documentation Architecture

Adopt Diátaxis-inspired structure:

- Tutorials
- How-to guides
- Explanations
- Reference

But keep product identity:

- Boundline docs focus on governed movement.
- Canon docs focus on governed meaning.

## Required Wiki Paths

### Boundline

- Getting Started
- Installation
- Assistant Integrations
- Daily Operating Guide
- Core Concepts
- Guidance And Guardians
- Expert Packs
- Traces And Inspectability
- Canon Integration
- Configuration
- Examples
- Troubleshooting
- Reference

### Canon

- Getting Started
- Installation
- Core Concepts
- Canon Modes
- Packets And Ordered Documents
- Evidence And Approvals
- Lineage And Provenance
- Project Memory
- Publishing And Promotion
- Domain Language
- Domain Model
- Architecture Decisions
- Boundline Integration
- Examples
- Troubleshooting
- Reference

## Style Guide

Create a shared markdown style guide:

- one purpose per page
- practical first
- examples before abstractions where possible
- no duplicate README text
- clear callouts
- no huge nested bullet forests
- diagrams for lifecycle and governance flow
- commands tested or marked conceptual
- glossary consistency across repos

## Acceptance Criteria

- `boundline help-next` works in uninitialized, initialized, active-session, blocked, and failed states.
- `canon help-next` works for at least 5 core modes.
- Wiki structure exists for both repos.
- Each main page has tutorial, how-to, explanation, or reference classification.
- Examples exist for common workflows.
- Troubleshooting maps common failures to next actions.

## Risks

- Docs become too large and stale.
- Help-next duplicates runtime logic.
- Documentation promises behavior not implemented.

## Hard Rule

The safe path must be the easiest path to discover.
