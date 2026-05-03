# Contract: Workflow Assistant Surface

**Feature**: 032-workflow-surface-closure  
**Date**: 2026-05-02

## Intent

Named workflows must be available through the shipped assistant surfaces as
first-class Boundline guidance, not as raw undocumented escape hatches.

## Scenarios

### 1. Shipped assistant families expose the same bounded workflow actions

**Given** a workspace with valid workflow definitions  
**When** an operator asks Claude, Codex, or Copilot how to use workflows  
**Then** the assistant surface must expose workflow discovery, run, status,
resume, and inspect through the same bounded Boundline command vocabulary.

### 2. Gemini remains CLI-first but not conceptually separate

**Given** the current Gemini release artifact is documentation rather than a
chat-native command pack  
**When** the operator follows Gemini guidance  
**Then** the guidance must use the same workflow and routing vocabulary as the
other assistant families and must not imply a separate Gemini-owned runtime.

### 3. Workflow follow-through stays actionable

**Given** a workflow pauses for capture, clarification, governance, review, or
terminal inspect follow-through  
**When** an assistant surfaces the next action  
**Then** it must preserve the CLI-reported bounded next command instead of
inventing provider-specific continuation steps.

## Acceptance Notes

- Assistant workflow guidance must stay thin over the local Boundline CLI.
- Missing workflow guidance in a shipped assistant family is a contract failure.
- The contract should be validated by assistant asset tests that look for the
  required sections and exact workflow command snippets.