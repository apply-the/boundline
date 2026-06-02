# Daily Operating Guide

Use this page while operating the main session-native Boundline loop.

## The Standard Loop

```text
init → goal → plan → run → status → next → inspect
```

Two optional preflight steps can happen before the loop:

- `boundline models auth ...` when the selected provider route needs a stored credential
- `boundline probe` when you want a read-only readiness answer

## Goal

Record the bounded objective:

```bash
boundline goal --goal "Add email validation to customer import"
```

Add a brief when the goal needs authored context:

```bash
boundline goal --goal "Shape onboarding audit logging requirements" --brief docs/product-brief.md
```

Good goals name the intended behavior, relevant area, and success condition.

## Plan

Ask Boundline to draft the plan:

```bash
boundline plan
```

Read:

- context summary
- context credibility
- primary inputs
- planning rationale
- validation posture
- stop or clarification requirements

Planning can stop explicitly. If the runtime surfaces `goal_quality_state`,
`plan_quality_state`, `backlog_quality_state`, or `planning_analysis_state`,
follow that output literally.

## Run

Execute the next bounded action:

```bash
boundline run
```

For a deliberate fast path after init:

```bash
boundline run --goal "Fix the failing add test"
```

Use `--compatibility` only when you intentionally want the manifest-backed
route.

## Status

Check current state:

```bash
boundline status
```

Use JSON for assistant or automation interpretation:

```bash
boundline status --json
```

Read `next_command`, blocked or degraded states, validation posture,
checkpoint refs, and any assistant-safe follow-up fields literally.

## Next

Ask for the next credible action:

```bash
boundline next
```

Use this after a run, failure, blocked state, or resumed session.

## Inspect

Inspect trace-backed evidence:

```bash
boundline inspect
```

Use this when you need to understand why Boundline selected context, blocked a
run, requested clarification, or recommended recovery.

## Recover

Start from runtime state:

```bash
boundline status
boundline next
boundline inspect
```

If the output includes a checkpoint restore command, use that exact command
only when you intentionally want to rewind the bounded workspace slice.

## Common Workflow Examples

### Small Bug Fix

```bash
boundline goal --goal "Fix the failing parser test in tests/unit/parser.rs"
boundline plan
boundline run
boundline status
```

### Planning Before A Risky Change

```bash
boundline goal --goal "Plan a safe migration for account identifiers" --brief docs/migration-notes.md
boundline plan
boundline inspect
```

### After Validation Fails

```bash
boundline status
boundline inspect
boundline next
```

Do not continue from memory. Use the reported validation, context, trace, and
recovery evidence.

# Examples

These examples show how to shape bounded work. Replace paths and goals with real workspace evidence.

## Small Implementation Task

Goal: fix one failing test or add one narrow behavior.

Why it stays bounded: one behavior and one validation slice are in scope.

```bash
boundline goal --workspace . --goal "Fix the failing email parser test in tests/unit/email_parser.rs"
boundline plan --workspace .
boundline plan --workspace . --confirm
boundline run --workspace .
boundline status --workspace .
```

Success conditions:

- files were changed when implementation ran
- validation passed
- terminal or clear next command
- trace location available

## Refactor Task

Goal: preserve behavior while improving structure.

Why it stays bounded: public behavior stays fixed while structure changes.

```bash
boundline goal --workspace . \
  --goal "Refactor invoice calculation into focused helpers without changing public behavior"
boundline plan --workspace .
boundline inspect --workspace .
```

Good plan evidence includes current tests, affected files, behavior preservation strategy, and verification command.

## Architecture-Guided Change

Goal: change behavior that crosses boundaries.

Why it stays bounded: the brief narrows the boundary-crossing change before execution begins.

```bash
boundline goal --workspace . \
  --goal "Introduce audit events for account lifecycle changes" \
  --brief docs/architecture/account-events.md
boundline plan --workspace .
boundline inspect --workspace .
```

Use `inspect` before confirming the plan. Check that context includes the relevant architecture notes, existing event flow, validation strategy, and risk boundaries.

## Test Or Safety-Net Task

Goal: add coverage before later mutation.

Why it stays bounded: the immediate objective is to lock current behavior before later changes.

```bash
boundline goal --workspace . \
  --goal "Add regression coverage for expired session recovery before changing auth logic"
boundline plan --workspace .
boundline plan --workspace . --confirm
boundline run --workspace .
```

This is useful when current behavior is unclear but needs to be locked before refactor or migration.

## Security-Sensitive Task

Goal: make a security-relevant change with explicit inspection.

Why it stays bounded: inspection happens before execution and the brief supplies the risk boundary.

```bash
boundline goal --workspace . \
  --goal "Replace plaintext API token persistence with hashed token storage" \
  --brief docs/security/token-storage.md
boundline plan --workspace .
boundline inspect --workspace .
```

Before running, check guidance sources, guardian expectations, migration safety, and verification strategy.

## Migration Task

Goal: change schema, storage, or compatibility-sensitive behavior.

Why it stays bounded: migration work is intentionally split into slices.

```bash
boundline goal --workspace . \
  --goal "Plan and implement the first bounded slice of account id migration" \
  --brief docs/migrations/account-id-migration.md
boundline plan --workspace .
boundline inspect --workspace .
```

Split migration work into bounded slices. Do not combine schema design, data migration, API compatibility, and cleanup in one uncontrolled run.

## Canon-Integrated Task

Goal: use governed knowledge or approvals.

Why it stays bounded: governed inputs affect planning and stop semantics, not ownership of runtime control.

```bash
boundline init \
  --workspace . \
  --canon-mode-selection auto-confirm \
  --risk medium \
  --zone engineering \
  --owner platform

boundline goal --workspace . \
  --goal "Implement governed audit logging requirements for onboarding" \
  --brief docs/product/onboarding-audit.md
boundline plan --workspace .
boundline inspect --workspace .
```

Expect Boundline to project Canon compatibility, governed memory, approval state, or stop semantics when those inputs apply.

## High-Risk Governed Task (Authority-Zoned)

Goal: perform a structural change that requires a Delivery Council based on Canon authority semantics.

Why it stays bounded: the council profile and stop semantics are explicit before the boundary is crossed.

```bash
boundline goal --workspace . \
  --goal "Replace the core routing mechanism to support multi-region traffic" \
  --brief docs/architecture/multi-region-routing.md
boundline plan --workspace .
boundline run --workspace .
```

Success conditions:
- Boundline reads Canon `authority-governance-v1` metadata and classifies the change as `red`.
- A **Council Profile** (e.g., `red_five`) is assembled before the boundary is crossed.
- If blocking findings are emitted, the session enters an `adjudication_required` or `hard_stop` state.
- The operator responds to the findings (e.g., `accepted`), which generates explicit remediation work before the stage can proceed.

## Failed Validation Follow-Up

After a failed run:

```bash
boundline status --workspace .
boundline next --workspace .
boundline inspect --workspace .
```

Use the reported next command, failed validation evidence, and checkpoint guidance. Do not continue from chat-only assumptions.
