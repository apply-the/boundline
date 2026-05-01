# Contract: Workflow Registry Guidance

## Purpose

Define the minimum documentation and guidance behavior for authoring workspace-local workflow registries in the workflow follow-through slice.

## Requirements

### 1. Supported authored shape stays explicit

The shipped guidance MUST explain the supported workflow registry shape for workflows that include review and govern.

### 2. Boundaries stay explicit

The guidance MUST name the unsupported behaviors explicitly, including:

- branching
- loops
- fan-out or fan-in
- hidden background progression
- Canon-owned workflow control

### 3. Route relationships stay clear

The guidance MUST explain how named workflows relate to:

- the primary direct session-native path
- the explicit compatibility path
- assistant-driven invocation guidance

### 4. Examples stay usable

At least one shipped example MUST be sufficient for a maintainer to author or update a representative workflow that includes review and govern without relying on undocumented behavior.

## Acceptance Examples

### Supported authored workflow

```text
workflow name: governed-delivery
phases: capture -> plan -> run -> review -> govern -> inspect
result: accepted as a bounded sequential workflow example
```

### Unsupported authored workflow

```text
workflow name: auto-escalate
phases: run -> review -> govern -> run
extra rule: repeat until approval
result: documented as unsupported and rejected before execution
```