# Contract: Workflow Routing Projection

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

## Intent

Workflow-facing runtime output must expose the same route, binding, and follow-
through cues that operators already rely on in the direct session-native
surfaces.

## Scenarios

### 1. Workflow output keeps route and binding inspectable

**Given** a named workflow is active on the primary native path  
**When** the operator runs workflow status, resume, or inspect  
**Then** the output must preserve workflow identity, workflow phase, routing,
execution condition, route-config projection, and bounded next-step guidance.

### 2. Assistant capability mismatch fails explicitly

**Given** the active workflow route resolves to an assistant runtime outside the
declared capability list  
**When** the operator starts or resumes the workflow  
**Then** Synod must stop explicitly with a surfaced assistant-binding failure
instead of silently falling back to a different runtime.

### 3. Workflow discovery still signals the primary product path

**Given** workflow discovery succeeds  
**When** the operator reads the discovery output  
**Then** the reported invocation guidance must clearly point back to Synod's
workflow command family and not to a separate provider-owned surface.

## Acceptance Notes

- Workflow routing projection should reuse the existing session-status rendering
  vocabulary where possible.
- Route ownership and execution-path cues must stay consistent with direct
  native and compatibility follow-up output.
- Contract coverage should validate both successful native workflow execution
  and explicit non-success cases such as invalid definitions or unsupported
  assistant bindings.