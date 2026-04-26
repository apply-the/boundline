# Contract: Adaptive Trace And Inspection Surface

## Purpose

Define the minimum trace and inspection evidence required for adaptive execution.

## Required trace evidence

Each adaptive delivery run MUST persist inspectable evidence for:

- the selected workspace slice for each adaptive attempt
- the candidate signature or equivalent stable identity for the current attempt
- any attempt-lineage transition such as `initial`, `narrowed`, `broadened`, `replaced`, or `terminated`
- validation outcomes per attempt
- final terminal outcome

The trace MAY continue to include the existing delivery and review lifecycle events before or after these adaptive details.

## Inspect output requirements

`synod inspect` MUST make the following visible after an adaptive run:

- the workspace slice chosen for each adaptive attempt
- the sequence of adaptive attempts in execution order
- how later attempts differed from earlier ones
- the validation result attached to each attempt
- the final terminal reason

## Failure visibility requirements

When adaptive execution stops because no credible next path exists, inspection output MUST show:

- the last attempted workspace slice
- the last validation result when available
- the rule or condition that caused termination instead of another retry or replan
