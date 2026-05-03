# Contract: Assistant Binding Surface

**Feature**: 027-routing-assistant-decoupling  
**Date**: 2026-05-01

## Intent

Assistant-facing guidance and backend binding must follow the effective slot
route instead of relying on a hard-wired default assistant family.

## Scenarios

### 1. Supported assistant family follows the configured route

**Given** the effective route for a bounded slot resolves to `claude`, `codex`,
or `copilot`  
**When** Boundline renders assistant guidance or binds the backend used for that
slot  
**Then** the selected assistant family must match the resolved route and keep
the same CLI command workflow.

### 2. Gemini remains an explicit documented fallback in this slice

**Given** the effective route resolves to `gemini`  
**When** Boundline renders assistant guidance for that slot  
**Then** the binding must surface `gemini` explicitly and use the repository's
Gemini CLI guidance as the supported artifact for this release instead of
silently reverting to a different assistant family.

### 3. Missing binding assets fail loudly

**When** the resolved runtime does not have a supported assistant artifact for
the current slice  
**Then** the system must produce an explicit error or operator-facing correction
that identifies the unsupported binding, and it must not fall back to a
hard-wired assistant family.

## Acceptance Notes

- Binding a different assistant family must not create a second orchestration
  runtime.
- Assistant binding must remain inspectable from the runtime or documentation
  story, not only from internal adapter registration.
- Contract coverage should validate Claude, Codex, Copilot, and the explicit
  Gemini CLI fallback artifact.