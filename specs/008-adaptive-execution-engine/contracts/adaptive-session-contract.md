# Contract: Adaptive Session Projection

## Purpose

Define the minimum adaptive evidence surfaced through `boundline status` and `boundline next`.

## Required status fields

When an adaptive delivery task has started, `boundline status` SHOULD surface:

- `latest_workspace_slice`
- `latest_selection_headline`
- `latest_attempt_lineage` when more than one adaptive attempt has occurred
- `latest_validation_status`
- existing session fields such as `latest_status`, `latest_trace_ref`, and `next_command`

## Required next guidance behavior

When adaptive execution remains in progress or ends non-successfully, `boundline next` MUST preserve:

- the latest adaptive slice summary
- the latest validation outcome
- one explicit next command or recovery action

## Omission behavior

When no adaptive evidence exists yet:

- adaptive-specific session fields MUST be omitted cleanly
- existing session-native output remains valid
