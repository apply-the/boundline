# Migration Safety Guidance

## Purpose

This guidance defines expectations for safe migration work.

It applies to data migrations, schema migrations, service migrations, framework migrations, cloud/provider migrations, API migrations, event migrations, storage migrations, and compatibility cutovers.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, migration plan, architecture decision, or Canon-governed migration packet.

## Core Thesis

Migrations fail when they are treated as implementation tasks.

A migration is a coordinated change across time.

Migration guidance exists to protect:

- compatibility
- sequencing
- data integrity
- rollback
- observability
- cutover control
- consumer safety
- operational containment

## Migration Classification

Classify migration type:

- schema migration
- data backfill
- service boundary migration
- API migration
- event schema migration
- storage migration
- identity/auth migration
- infrastructure migration
- framework/runtime migration
- dependency migration

Classify reversibility:

- reversible
- compensatable
- irreversible
- unknown

Unknown reversibility is a risk.

## Expand/Contract

For schema and contract migrations, prefer expand/contract sequencing.

Typical sequence:

1. expand schema/contract compatibly
2. deploy writers that can use new shape
3. deploy readers compatible with both old and new
4. backfill data
5. switch reads
6. stop old writes
7. contract old shape after compatibility window

Skipping phases requires explicit rationale.

## Dual Write And Dual Read

Dual-write and dual-read strategies are powerful but risky.

Check:

- source of truth
- consistency model
- reconciliation
- failure behavior
- idempotency
- monitoring
- cutover criteria
- rollback behavior

Do not add dual writes without discrepancy detection.

## Backfills

Backfills require:

- dry run
- batching
- rate limits
- progress tracking
- pause/resume
- validation queries
- error handling
- idempotency
- monitoring
- stop criteria

Avoid:

- one-shot unbounded update
- no progress reporting
- no validation
- no rollback/compensation
- no owner during execution

## Rollback And Compensation

Every migration must state:

- rollback possible?
- rollback safe?
- compensation required?
- data loss possible?
- compatibility preserved?
- previous version can run?
- feature flag or cutover switch exists?

Some migrations cannot be rolled back. That must be explicit and governed.

## Cutover

Cutover should define:

- trigger condition
- owner
- timing
- monitoring window
- success criteria
- failure criteria
- rollback/compensation path
- communication
- freeze requirements if any

## Observability

Migration observability should include:

- progress
- error count
- latency
- throughput
- discrepancy count
- retry count
- affected records
- source/target comparison
- user impact

No migration should be invisible while running.

## Compatibility

Migrations must consider:

- old code with new data
- new code with old data
- old consumers with new contract
- new consumers with old provider
- replayed events
- partially migrated state
- background jobs during migration

## AI-Assisted Delivery Risks

AI-generated migration plans often:

- skip expand/contract
- assume downtime is acceptable
- ignore old readers/writers
- omit backfill validation
- omit rollback
- create irreversible schema changes too early
- forget queues/workers
- ignore mobile clients or external consumers
- treat local tests as migration evidence

Guardians should challenge these omissions.

## Anti-Patterns

- destructive schema change first
- required column added without default/backfill plan
- backfill without batching
- migration without dry run
- no compatibility window
- no rollback or compensation statement
- no progress monitoring
- dual-write without reconciliation
- cutover without stop criteria
- migration hidden inside application startup
- old clients ignored
- irreversible action without approval

## Guardian Hooks

Recommended guardians:

- migration-sequencing-guardian
- expand-contract-guardian
- rollback-safety-guardian
- backfill-readiness-guardian
- dual-write-safety-guardian
- cutover-readiness-guardian
- migration-observability-guardian
- compatibility-window-guardian
- destructive-change-guardian

## Structured Finding Example

```json
{
  "guardian": "expand-contract",
  "rule": "destructive-schema-change-first",
  "disposition": "blocker",
  "summary": "The migration drops `legacy_customer_id` before readers have been updated and before a compatibility window is documented.",
  "evidence_refs": ["migrations/202605_drop_legacy_customer_id.sql"],
  "recommended_action": "Use expand/contract sequencing: deploy compatible readers, verify adoption, then remove the old field in a later migration."
}
```

## Lifecycle Usage

Planning:
- classify migration type and reversibility

Architecture:
- define compatibility, sequencing, and ownership

Migration:
- produce migration plan, cutover criteria, and fallback

Implementation:
- implement compatible changes and observable backfill

Testing:
- verify old/new compatibility and failure cases

Review:
- challenge destructive changes, rollback gaps, and missing evidence

Verification:
- compare migration claims to dry-run, telemetry, and validation evidence
