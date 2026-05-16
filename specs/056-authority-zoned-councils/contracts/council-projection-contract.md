# Council Projection Contract

## Purpose

Define the minimum operator-visible projection that Boundline must persist and
surface after resolving one governed boundary under the authority-zoned council
slice.

## Required Projection Fields

The persisted council projection must be able to surface:

- `canon_contract_line`: consumed Canon contract line or explicit absence.
- `authority_zone`: Canon authority zone used for the decision.
- `change_class`: Canon change class used for the decision.
- `intended_persona`: Canon intended persona carried into the decision trace.
- `effective_control_class`: local control result derived from Canon plus local evidence posture.
- `council_profile`: one of `none`, `light_single`, `yellow_pair`, `red_five`, or `restricted_manual`.
- `mandatory_roles`: required runtime roles for the selected profile.
- `selected_roles`: ordered local runtime roles actually assigned.
- `independence_state`: whether reviewer independence passed, degraded, or failed.
- `stop_semantics`: explicit progression result such as `proceed`, `council_required`, `human_gate_required`, or `hard_stop`.
- `findings_summary`: concise summary of current findings and their severities.
- `producer_response_summary`: concise summary of recorded producer responses.
- `adjudication_summary`: explicit adjudication outcome when mixed or blocking findings exist.
- `next_action`: required operator or runtime follow-up.
- `optional_canon_provenance`: any optional Canon provenance surfaced separately from required control inputs.

## Projection Rules

- The same resolved projection must be reusable by `plan`, `run`, `status`, `next`, and `inspect` rather than recomputed independently per command.
- Projection must remain deterministic for the same persisted session state and Canon input.
- Optional Canon provenance must stay distinguishable from the required fields that actually drove the control decision.
- Reviewer independence failures, missing mandatory roles, unresolved blocking findings, and required human gates must be projected explicitly rather than summarized as generic failure.

## Failure Projection Rules

The council projection must make the blocking reason explicit when any of these
conditions hold:

- unsupported Canon contract line
- missing required Canon control metadata
- failed reviewer independence
- missing mandatory runtime roles
- unresolved blocking findings
- restricted action waiting on a required human gate

In those cases the projection must preserve the explicit blocked or hard-stop
posture and the required next action for the operator.