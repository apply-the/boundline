# Contract: Governed Stage Refresh And Lineage

## Purpose

Define the minimum behavior for approval refresh and packet-lineage visibility across the governed `bug-fix:investigate` to governed `verify` path.

## Requirements

### 1. Refresh happens before resumed progression

When a governed session is resumed or inspected through later commands, Boundline MUST refresh approval and packet-readiness state before allowing the next governed stage to continue.

### 2. Packet lineage stays bounded and explicit

When later governed work reuses a prior packet, the surfaces MUST expose:

- the reused packet reference
- the upstream source stage
- the packet binding reason

### 3. Refresh outcomes do not hide non-success transitions

When refreshed state changes from waiting to blocked or from waiting to reusable, Boundline MUST report the updated state explicitly and MUST NOT advance hidden work.

## Acceptance Examples

### Later status refreshes approval

```text
latest_governance_stage: bug-fix:investigate
latest_governance_state: governed_ready
latest_governance_approval: approved
headline: refreshed governance approval state for the active workspace session
```

### Verify reuses investigate lineage

```text
latest_governance_stage: bug-fix:verify
latest_governance_packet_ref: .canon/runs/canon-run-investigate
latest_governance_packet_source_stage: bug-fix:investigate
latest_governance_packet_binding_reason: upstream_stage_context
```