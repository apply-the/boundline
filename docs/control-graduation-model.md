# Control Graduation Model In Boundline 0.56.0

Boundline `0.56.0` resolves one explicit runtime governance posture for each
governed boundary. Canon supplies semantic inputs. Boundline decides what the
runtime does with them.

This S4 slice keeps the primary operator path unchanged:

```text
start -> capture -> plan -> run -> status -> next -> inspect
```

Control graduation is therefore not a second orchestration system. It is a
runtime-owned decision layer on top of the existing session-native workflow.

## Runtime-Owned Outputs

Boundline owns these outcomes even when Canon governance is active:

- effective runtime governance state: `advisory`, `catch`, `rule`, `hook`
- rollout profile: `minimal`, `guided`, `governed`, `strict`
- confidence and trust evolution
- degradation and escalation choices
- council assembly and stop semantics
- override handling and follow-through

Canon does not choose those outcomes for Boundline. Canon provides semantic
posture, readiness, approval, lineage, and promotion metadata that Boundline
may consume while still keeping local runtime authority.

## Canon Inputs At The Boundary

Current S4 consumption uses two Canon envelopes:

- required baseline: `authority-governance-v1`
- optional additive companion: `adaptive-governance-v1`

The required baseline classifies the packet posture. The optional companion adds
semantic maturity labels that Boundline may project and reuse as input to local
explanation.

## Stage Policy Gates

Stage policy controls whether a Canon boundary can continue without a given
contract.

- `required: true` on a Canon stage means the required authority baseline must
  be present and compatible, otherwise the stage fails closed.
- `require_adaptive_companion: true` on a Canon stage means the adaptive
  companion must also be present and compatible before a `governed_ready`
  Canon response may continue.

This yields three distinct compatibility paths:

| Boundary state | Result |
|---|---|
| authority baseline compatible, companion absent, companion optional | continue with local runtime logic |
| authority baseline compatible, companion compatible, companion optional or required | continue and project both envelopes |
| authority baseline compatible, companion unavailable, companion required | block with `adaptive_contract_unavailable` |

## Fail-Closed Behavior

Boundline blocks a required Canon stage when:

- `authority-governance-v1` is missing or unsupported
- the required authority posture implies a hard stop
- `adaptive-governance-v1` is required by stage policy but is missing,
  unavailable, or unsupported

For the adaptive companion, the explicit reason code is:

- `adaptive_contract_unavailable`

That failure does not rewrite Canon semantics into a local approximation. It
remains an explicit compatibility stop.

## Projection Model

Boundline preserves Canon baseline lines and adaptive companion lines
separately in compacted Canon memory and reuses them through session and trace
surfaces.

Examples:

- `authority_contract_line: authority-governance-v1`
- `authority_control_class: council_review`
- `adaptive_contract_line: adaptive-governance-v1`
- `adaptive_governance_state: rule`
- `adaptive_rollout_profile: governed`

When the companion is optional and absent, Boundline still keeps that fact
visible:

- `adaptive_contract_line: unavailable`

This matters because operators can distinguish three cases clearly:

- no companion was supplied
- a compatible companion was supplied
- a required companion was unavailable and blocked execution

## Relationship To S3

S3 still owns the authority-zone, council, and stop-semantics vocabulary that
Boundline maps onto runtime outcomes. S4 adds adaptive governance maturity and
runtime follow-through without moving the source of runtime authority out of
Boundline.

For the current authority-zone matrix and stop posture details, see
`docs/authority-zones-and-stop-semantics.md`.