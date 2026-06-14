# Traces And Inspectability

Boundline 0.79.0 traces make delivery explainable. They preserve what the runtime decided, what it used as evidence, what it ran, what it skipped, what failed, and what should happen next.

## Where Traces Live

Workspace traces live under:

```text
.boundline/traces/
```

The active session state lives at:

```text
.boundline/session.json
```

When CLI output reports `trace_location`, treat it as the authoritative trace reference for that run or inspection.

## What Traces Contain

Depending on the command and lifecycle phase, traces can include:

- command name and exit status
- rendered operator output
- session status
- context summary and credibility
- context primary inputs and provenance
- plan state and planning rationale
- verification strategy
- plan-quality state, findings, assumptions, and the emitted `phase_request`
- backlog-quality state, findings, task count, MVP scope, unmapped items, and
  any withheld execution handoff
- planning-analysis state, source-attributed findings, coverage metrics, and
  any execution-handoff withholding caused by cross-artifact contradictions or
  producer contract gaps
- capability-provider identity, activation state, validation disposition,
  failure class, accepted evidence refs, rejected evidence refs, and declared
  limitations when a provider-backed path ran or was blocked
- route owner and route config projection
- selected guidance and guardian sources
- loaded and skipped packs
- catalog validation findings
- guardian timeline and findings
- changed files and validation status
- completion-verification state, claim, blocked claims, evidence refs, and
  stale or failed proof findings when closeout is gated
- checkpoint refs
- Canon packet or project-memory refs when governed delivery is active
- next command, corrected command, or stop reason

## Inspect Decisions

Use:

```bash
boundline inspect --workspace .
```

Use JSON when an assistant or script needs structured fields:

```bash
boundline inspect --workspace . --json
```

Ask these questions while reading inspect output:

- Why did Boundline select this context?
- Was context credible enough to continue?
- Which guidance shaped the plan?
- Which guardians ran, skipped, degraded, or blocked?
- Which route owned the current step?
- What evidence supports the next action?
- Which planning artifact or governed document blocked execution readiness?
- Is closeout still blocked on missing, stale, failed, or mismatched proof?
- Which proving command or rerun action is required before completion can proceed?

## Inspect Guidance And Guardians

Look for:

- `guidance_resolution_summary`
- `loaded_guidance_sources`
- `loaded_guardian_sources`
- `loaded_packs`
- `skipped_packs`
- `catalog_validation_findings`
- `guardian_timeline`
- `guardian_findings_summary`
- `guardian_blocking_outcome`

These fields explain capability selection and validation behavior.

## Inspect Context

Look for:

- `context_summary`
- `context_credibility`
- `context_primary_inputs`
- `context_provenance`
- `context_staleness_reason`
- `retrieval_index_state`
- `semantic_capability_state`
- `semantic_fallback_reason`
- `retrieval_recovery_guidance`

If context is weak, repair it by narrowing the goal, adding a brief, producing validation evidence, or pointing at relevant files.

## Inspect Recovery

After a failed or blocked run:

```bash
boundline status --workspace .
boundline next --workspace .
boundline inspect --workspace .
```

Preserve reported checkpoint fields:

- `latest_checkpoint_id`
- `latest_checkpoint_scope`
- `latest_checkpoint_restore_command`

Use the restore command only when intentionally rewinding the bounded workspace slice.

## Operator Rule

Do not infer continuation from chat history. Continue from `status`, `next`, `inspect`, and trace-backed evidence.
