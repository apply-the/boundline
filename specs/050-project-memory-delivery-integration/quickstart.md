# Quickstart: Project Memory Delivery Integration

## Scenario 1: Stage planner uses stable Canon project memory

Given `docs/project/architecture-map.md` exists with a lineage sidecar:

```json
// docs/project/architecture-map.lineage.json
{
  "contract_version": "1.0.0",
  "source_run": "019738a4-...",
  "mode": "architecture",
  "profile": "project-memory",
  "promotion_state": "auto",
  "readiness": "stable"
}
```

When Boundline plans the next stage:

```bash
boundline next
```

**Expected result**: The stage planner reads the Canon output, resolves
`PromotionStateView::Stable`, and uses the architecture map as credible
context for delivery decisions. The trace records the Canon ref and promotion
state.

## Scenario 2: Pending Canon output is visible but non-authoritative

Given `docs/project/pending-decisions.md` with lineage showing
`promotion_state: "pending-index"`:

```bash
boundline status
```

**Expected result**: Status shows the pending Canon ref with a
`PendingOrIndex` marker. The stage planner does not treat the pending content
as accepted project truth.

## Scenario 3: No Canon output available

Given no `docs/project/` or `docs/evidence/` directories:

```bash
boundline next
```

**Expected result**: `ProjectMemoryContext.status` is `Absent`. Boundline
continues delivery using other available context. No error, no synthetic
Canon output.

## Scenario 4: Unsupported contract version

Given Canon output with `contract_version: "2.0.0"` but Boundline supports
only `1.x`:

```bash
boundline next
```

**Expected result**: Boundline detects `CompatibilityOutcome::Unsupported`,
stops with explicit guidance: "Canon project-memory contract version 2.0.0
is not supported. Update Boundline to consume the newer contract."

## Scenario 5: Evidence refs in governed stage

Given a governed stage whose assurance profile references Canon evidence, and
`docs/evidence/security-assessment.lineage.json` exists:

```bash
boundline run
```

**Expected result**: The assurance evaluator reads Canon evidence refs and
lineage metadata. The trace records which Canon evidence was consumed. The
stage proceeds using Boundline-owned orchestration logic.

## Scenario 6: Inspect shows Canon refs

```bash
boundline inspect
```

**Expected result**: Inspect output includes a "Canon Project Memory" section
listing discovered surfaces, their promotion states, lineage refs, and
contract compatibility status.
