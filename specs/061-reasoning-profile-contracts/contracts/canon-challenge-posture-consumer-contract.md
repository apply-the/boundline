# Canon Challenge Posture Consumer Contract

- **Consumer**: Boundline
- **Canonical Source Of Truth**: Canon provider-side contract at
  `docs/integration/governed-reasoning-posture-contract.md`
- **Canonical Source Identifier**: `canon:docs/integration/governed-reasoning-posture-contract.md`
- **Supported Contract Line**: `governed_reasoning_posture_v1`
- **Supported Compatibility Window**: Boundline `0.64.x` consuming Canon
  `0.61.x` posture inputs only

## Boundline Consumes

- `contract_line`
- `compatibility_window`
- `required_profile_family` or explicit `required_profile_id`
- `minimum_independence`
- `admission_priority`
- `confidence_handoff_required`
- `provenance_ref`
- optional additive fields explicitly marked compatible with the same contract
  line

## Boundline Does Not Own

- Canon authority-zone semantics
- Canon approval semantics
- Canon evidence semantics
- Canon admission posture authoring
- Canon contract-line deprecation policy

## Consumer Rules

- Boundline MUST reject unsupported contract lines before profile activation.
- Boundline MUST reject posture inputs whose compatibility window excludes the
  active Boundline or Canon version.
- Boundline MUST treat missing required posture fields as explicit blocked or
  incompatible input, not as a silent fallback.
- Boundline MUST keep Canon posture advisory to runtime selection, not to final
  acceptance authority.
- Boundline MAY use local policy or local fixtures when Canon posture is absent,
  but it MUST surface that absence explicitly.
- Boundline MUST NOT redefine Canon posture terms under new local names when the
  original Canon meaning can be preserved.

## Preferred Producer Shape

```toml
contract_line = "governed_reasoning_posture_v1"
boundline_min = "0.64.0"
boundline_max_exclusive = "0.65.0"
canon_min = "0.61.0"
canon_max_exclusive = "0.62.0"
required_profile_family = "blind_review"
admission_priority = "required_before_acceptance"
confidence_handoff_required = true
provenance_ref = "packet:reasoning-posture-123"

[minimum_independence]
route_distinct = true
provider_distinct = true
context_distinct = false
prompt_pattern_distinct = true
minimum_participants = 2
```

## Explicit Exclusions

- Canon does not choose Boundline routes, runtime participants, or final review
  adjudication implementation.
- Canon does not execute debate, reflexion, or self-consistency loops.
- Boundline does not author or mutate Canon posture documents through this
  contract.
