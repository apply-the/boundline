# Guidance And Guardian Trace Contract

## Purpose

Define the operator-visible projection that Boundline must persist and surface
for capability resolution and guardian execution.

## Required Projection Fields

The persisted projection must be able to expose:
- `capability_resolution_summary`
- `loaded_guidance_sources`
- `skipped_guidance_sources`
- `loaded_guardian_sources`
- `skipped_guardian_sources`
- `guardian_timeline`
- `guardian_findings_summary`
- `guardian_findings`
- `guardian_degradations`
- `guardian_blocking_outcome`

## Projection Rules

- Projection must distinguish workspace overrides, Canon-governed standards, shared packs, and built-ins.
- Projection must preserve the ordered guardian execution timeline for the active phase.
- Projection must make deterministic-before-LLM ordering visible.
- Projection must preserve skipped-source and skipped-guardian reasons.
- Projection must show route-unavailable degradation explicitly rather than implying a hidden fallback.
- Session-native read surfaces and trace summaries must read from the same persisted capability and finding state rather than recomputing a second evaluation path.
