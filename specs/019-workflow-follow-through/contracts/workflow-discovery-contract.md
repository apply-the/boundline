# Contract: Workflow Discovery Surface

## Purpose

Define the minimum operator-facing behavior for discovering available named workflows and selecting the correct invocation path.

## Requirements

### 1. Available workflows stay visible

When a workspace defines one or more named workflows, the workflow discovery surface MUST expose:

- the available workflow names
- a short summary or fallback description for each workflow
- the declared phase chain for each workflow

### 2. Invocation guidance stays actionable

For each discoverable workflow, the surface MUST expose enough invocation guidance for an operator or assistant to start the correct workflow without reading the raw registry file.

### 3. Invalid or unsupported workflows stay explicit

When the registry is missing, invalid, or contains unsupported workflow shapes, the discovery surface MUST explain that state explicitly rather than failing silently.

### 4. Discovery does not silently start work

The discovery surface MUST remain read-only and MUST NOT activate a workflow or override the direct session-native path on the operator's behalf.

## Acceptance Examples

### Workspace with valid workflows

```text
workflow: governed-delivery
summary: use when the task requires review and governance before completion
phases: capture -> plan -> run -> review -> govern -> inspect
invoke_with: synod workflow run governed-delivery --workspace .
```

### Workspace with invalid workflow definitions

```text
workflow registry status: invalid
reason: unsupported workflow shape present in .synod/workflows.toml
next_command: synod workflow inspect --workspace .
```