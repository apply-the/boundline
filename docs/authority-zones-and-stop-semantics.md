# Authority Zones And Stop Semantics In Boundline 0.63.0

Boundline `0.63.0` consumes Canon `authority-governance-v1` envelopes when a
governed Canon stage returns a reusable packet. Canon remains the semantic
authority for the boundary classification. Boundline remains the runtime owner
for the resulting council profile, stop posture, session projection, and trace
events.

When a stage is marked `required` and runs through the Canon runtime, Boundline
now fails closed if the packet is missing compatible authority metadata. A
required governed boundary does not silently downgrade to an ungoverned
continue path.

## Required Authority Inputs

The current contract slice requires these Canon fields:

- `authority_zone`
- `change_class`
- `intended_persona`
- `approval_state`
- `packet_readiness`
- `risk`

These optional provenance-only fields can be present and are projected without
changing the runtime-owned council decision:

- `persona_anti_behaviors`
- `primary_artifact`
- `artifact_order`
- `promotion_refs`
- `stage_role_hints`

## Authority Zones

- `green`: low-blast-radius bounded work
- `yellow`: elevated coordination or multi-owner work
- `red`: materially risky structural or operational work
- `restricted`: destructive or explicitly human-gated work

`change_class` and the active stage can raise the effective control posture
beyond the raw zone. Boundline keeps the first-slice roadmap matrix as the
runtime contract for the most important green, yellow, red, and restricted
paths.

## Control Resolution Matrix

Current V1 resolution:

| Zone | Risk | Stage | Council Profile | Stop Semantics |
|---|---|---|---|---|
| green | low-impact | discovery, requirements | none | proceed |
| green | low-impact | implementation, refactor | light_single | proceed |
| green | bounded-impact | architecture and other yellow-floor work | yellow_pair | council_required |
| yellow | bounded-impact | implementation, verification, other yellow-band work | yellow_pair | council_required |
| yellow | systemic-impact or critical-operations | structural work | red_five | adjudication_required |
| red | any structural risk | architecture, migration, security | red_five | human_gate_required |
| restricted | any | destructive or unresolved approval state | restricted_manual | hard_stop |

Structural overrides still apply before the generic matrix:

- approval `requested` in restricted space becomes `restricted_manual`
- packet readiness `incomplete` or `rejected` becomes `restricted_manual`
- approval `rejected` or `expired` becomes `restricted_manual`
- unsupported Canon contract lines become blocked
- missing authority metadata on required Canon stages becomes blocked

## Stop Semantics Continuum

- `proceed`: continue without a council gate
- `proceed_with_advisory`: continue but preserve an advisory caution
- `proceed_with_warning`: continue with an elevated warning state
- `degraded_proceed`: continue while preserving degraded credibility
- `council_required`: stop until the bounded council review is satisfied
- `adjudication_required`: stop until adjudication resolves the conflict
- `human_gate_required`: stop until an explicit human gate is granted
- `hard_stop`: do not continue on the current runtime path

The current authority-zoned council slice primarily exercises
`proceed`, `council_required`, `adjudication_required`,
`human_gate_required`, and `hard_stop`.

## What Operators See

The session-native surfaces preserve one authority decision across `run`,
`status`, `next`, and `inspect`:

- consumed Canon contract line
- effective control class
- council profile
- review independence state
- review selection summary
- review stop semantics
- optional authority provenance lines when Canon supplied them

That split matters: Canon describes the semantic posture of the packet, while
Boundline explains the runtime consequence of that posture.

## Failure Modes To Expect

- required Canon stage with no authority envelope: blocked fail-closed response
- unsupported authority contract line: blocked compatibility response
- red boundary: human gate required even when the packet is otherwise reusable
- restricted boundary or unresolved approval state: hard stop
- rejected or incomplete packet: hard stop until the boundary is remediated

Use `boundline status` for the compact posture, `boundline next` for the next
action, and `boundline inspect` when you need the authority provenance and
review-state detail that led to the stop.