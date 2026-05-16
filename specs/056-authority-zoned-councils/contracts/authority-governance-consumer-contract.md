# Authority Governance Consumer Contract

## Purpose

Define the minimum Canon `authority-governance-v1` surface that Boundline must
consume before it resolves a local control class, bounded council profile, or
stop posture.

## Required Canon Fields

Boundline must be able to recover these required control inputs from a
compatible Canon `authority_governance` object:

- `contract_line`
- `authority_zone`
- `change_class`
- `intended_persona`
- `approval_state`
- `packet_readiness`
- `risk`

## Optional Canon Provenance

The same Canon contract line may also include optional additive metadata that
Boundline may inspect and project without treating it as runtime control:

- `persona_anti_behaviors`
- `primary_artifact`
- `artifact_order`
- `promotion_refs`
- `stage_role_hints`

Missing optional provenance does not invalidate an otherwise compatible
contract.

## Consumer Rules

- Boundline only consumes `authority-governance-v1` in this first slice.
- Required Canon fields may influence local control resolution; optional
  provenance fields may not change the control outcome by themselves.
- Canon `stage_role_hints` are advisory only and must not directly assign local
  runtime roles, councils, provider routes, model routes, retry policy, or
  final decision authority.
- If `authority_governance` is absent, Boundline may still use the stable flat
  lifecycle fields from the Canon governance runtime when governance is not
  required.

## Fail-Closed Rules

Boundline must treat the authority contract as unavailable when:

- `contract_line` is unsupported
- any required `authority-governance-v1` field is missing
- required field values are incompatible with the supported first-slice
  vocabulary

When governance is required for the current boundary, an unavailable authority
contract must produce an explicit blocked or hard-stop posture rather than a
silent downgrade to an ungoverned path.

## Explicit Exclusions

Canon does not choose, and this consumer contract does not accept, any of the
following as Canon-owned runtime directives:

- council profile selection
- reviewer assignment
- provider route selection
- model route selection
- retry policy
- stop-transition policy
- final adjudication authority inside Boundline