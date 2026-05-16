# Adaptive Governance Consumer Contract

## Purpose

Define the Canon S4 surface that Boundline may consume before it resolves a
local governance state, rollout profile, confidence posture, degradation, or
escalation outcome.

The same Canon envelope may arrive inline on the governance response or through
the runtime packet metadata sidecar. Boundline consumes one compatible shape
and preserves the Canon-owned semantics separately from Boundline-owned runtime
decisions.

## Required Canon Baseline

For first-slice S4 runtime behavior, Boundline still relies on Canon
`authority-governance-v1` as the required posture baseline when governance is
required for the current boundary.

Boundline must be able to recover these required control inputs from a
compatible Canon `authority_governance` object:

- `contract_line`
- `authority_zone`
- `change_class`
- `intended_persona`
- `approval_state`
- `packet_readiness`
- `risk`

## Optional Canon Companion

Canon may also publish an optional additive `adaptive_governance` object with
the `adaptive-governance-v1` contract line.

Required companion fields when the object is present:

- `contract_line`
- `governance_state`
- `rollout_profile`

Optional companion metadata:

- `state_rationale`
- `profile_rationale`

Example shape:

```json
{
  "authority_governance": {
    "contract_line": "authority-governance-v1",
    "authority_zone": "yellow",
    "change_class": "bounded-impact",
    "intended_persona": "system-architect",
    "approval_state": "not_needed",
    "packet_readiness": "reusable",
    "risk": "bounded-impact"
  },
  "adaptive_governance": {
    "contract_line": "adaptive-governance-v1",
    "governance_state": "advisory",
    "rollout_profile": "guided"
  }
}
```

## Consumer Rules

- `authority-governance-v1` remains the required Canon posture baseline for this slice.
- `adaptive-governance-v1` is optional overall unless a local stage policy explicitly requires a compatible companion contract.
- Local stage policies use `require_adaptive_companion` to opt one Canon stage into fail-closed companion enforcement.
- Companion semantics may influence local explanation or initial maturity seeding, but they do not own runtime confidence, trust evolution, council assembly, degradation choice, escalation choice, provider routes, model routes, or stop transitions.
- Missing optional companion metadata does not invalidate an otherwise compatible authority baseline.
- Malformed, incomplete, or unsupported companion payloads are treated as unavailable rather than partially merged into local runtime state.
- An adaptive companion contract cannot repair a missing or incompatible required authority baseline.

## Fail-Closed Rules

Boundline must treat the required authority contract as unavailable when:

- `authority_governance` is absent and governance is required
- `contract_line` is unsupported
- any required `authority-governance-v1` field is missing
- required field values are incompatible with the supported baseline vocabulary

Boundline must treat the adaptive companion as unavailable when:

- `adaptive_governance` is present but `contract_line` is unsupported
- required companion fields are missing
- companion values are incompatible with the supported S4 vocabulary

Runtime consequence rules:

- unavailable required baseline + governance required: explicit blocked or hard-stop posture
- unavailable companion + companion optional: record companion unavailability and continue with baseline plus local runtime logic
- unavailable companion + companion required by stage policy: explicit compatibility failure with visible rationale

## Projection Rules

Boundline must keep Canon baseline and companion projection separate on compact
surfaces and trace surfaces.

Minimum visible lines for a compatible governed packet:

- `authority_contract_line: authority-governance-v1`
- `adaptive_contract_line: adaptive-governance-v1` when a compatible companion is present
- `adaptive_contract_line: unavailable` when the companion is absent but the baseline remains compatible
- `adaptive_governance_state: <state>` and `adaptive_rollout_profile: <profile>` when the companion is present
- `adaptive_state_rationale: <text|unavailable>` and `adaptive_profile_rationale: <text|unavailable>` when the companion is present

Minimum visible failure projection for a required companion mismatch:

- `reason_code: adaptive_contract_unavailable`
- a blocked message that explains whether the companion was missing, unavailable, or unsupported for the required stage

## Explicit Exclusions

Canon does not choose, and this consumer contract does not accept, any of the
following as Canon-owned runtime directives:

- confidence score calculation
- trust evolution
- council profile selection
- reviewer assignment
- provider route selection
- model route selection
- degradation mode selection
- escalation target selection
- stop-transition policy
- final adjudication or override authority inside Boundline