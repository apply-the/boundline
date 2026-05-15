# Capability Manifest Contract

## Purpose

Define the minimum manifest and discovery shapes that Boundline must support for
shared and workspace-local guidance and guardian capabilities.

## Guidance Declaration

Shared-pack or built-in guidance entries must be able to declare:
- `guidance.<id>.title`
- `guidance.<id>.applies_to`
- `guidance.<id>.roles`
- `guidance.<id>.path`
- `guidance.<id>.priority`

Workspace-local guidance overrides are Markdown files discovered under:
- `.boundline/guidance/*.md`

## Guardian Declaration

Shared-pack or built-in guardian entries must be able to declare:
- `guardians.<id>.title`
- `guardians.<id>.kind`
- `guardians.<id>.applies_to`
- `guardians.<id>.rules`
- `guardians.<id>.severity_floor`
- `guardians.<id>.command` for `deterministic` and `hybrid` guardians
- `guardians.<id>.instruction` for `llm` and `hybrid` guardians

Workspace-local guardian overrides are TOML files discovered under:
- `.boundline/guardians/*.toml`

## Source And Precedence Rules

- Boundline must preserve the winning `authority_source` and `source_ref` for every resolved guidance and guardian entry.
- Workspace overrides may shadow Canon-governed, shared-pack, or built-in entries, but the shadowing decision must remain trace-visible.
- Canon-governed standards remain optional and must never become a hard precondition for capability resolution.
- Invalid or unsupported manifest files must produce explicit load errors and skipped-source records rather than crashing the runtime.
- Capability resolution must remain deterministic for the same effective inputs.
