# Contract: Canon Memory Credibility

## Purpose

Define when compacted Canon-grounded memory may be reused across loops and when
it must trigger refresh, replanning, or an explicit stop.

## Preconditions

- A session or active task carries compacted Canon-grounded memory.
- Later planning, execution, or inspection wants to rely on that memory.

## Credible reuse

### Required behavior

- If the compacted Canon-grounded memory remains credible, later bounded
  planning and decision selection may reuse it without replaying the entire
  workspace or full Canon artifact bundle.
- Reused memory must preserve decisive evidence headlines, packet lineage, and
  capability constraints needed for the next bounded action.

## Non-credible states

### Required behavior

- The runtime must mark compacted Canon-grounded memory as non-credible when:
  - required packet lineage no longer matches,
  - required artifact provenance is missing,
  - Canon capability constraints changed materially,
  - later validation contradicts a carried-forward Canon assumption, or
  - the compacted summary no longer contains enough evidence for the next
    bounded action.

### Resulting actions

- `stale` memory must produce an explicit refresh or replan requirement.
- `contradicted` memory must produce an explicit replan or stop requirement.
- `insufficient` memory must produce an explicit clarification, refresh, or stop
  requirement.
- No non-credible state may be treated as silently reusable.

## Traceability requirements

- Traces and read-side surfaces must show:
  - the current credibility state,
  - why the memory lost credibility,
  - what bounded next action is required, and
  - which Canon lineage or capability signal triggered the change.