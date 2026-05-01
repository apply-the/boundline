# Contract: Governance Profile Guidance

## Purpose

Define the minimum documentation and guidance behavior for authoring the deeper governed bug-fix slice in workspace execution profiles.

## Requirements

### 1. Supported authored shape stays explicit

The shipped guidance MUST show one supported profile shape that governs `bug-fix:investigate` before the existing governed verify story.

### 2. Boundaries stay explicit

The guidance MUST call out that this slice does not introduce:

- Canon-owned orchestration
- hidden background progression
- full Canon artifact exposure
- a generic governance graph

### 3. Route relationships stay clear

The guidance MUST explain how the deeper governed slice relates to:

- the primary session-native path
- the bounded workflow-aware projection
- the explicit compatibility path when operators intentionally choose it

## Acceptance Examples

### Supported authored profile

```text
flow_name: bug-fix
stage_id: investigate
runtime: canon
canon_mode: discovery
result: accepted as a bounded deeper governed stage example
```

### Unsupported authored expectation

```text
flow_name: bug-fix
stage_id: investigate
extra_rule: keep polling Canon until approval changes
result: documented as unsupported hidden background progression
```