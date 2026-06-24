# Canon Governed Reasoning Posture Contract Snapshot

This snapshot preserves the Canon provider-side release contract needed for Boundline-local validation when the sibling Canon repository is unavailable.

## Contract Identity

- `owner`: `canon`
- `current_contract_line`: `governed_reasoning_posture_v1`
- `schema_version`: `v1`
- `primary_consumer`: `boundline`
- `supported_boundline_window`: `0.79.x`
- `supported_canon_window`: `0.71.x`

## Producer Shape

```toml
contract_line = "governed_reasoning_posture_v1"
boundline_min = "0.75.0"
boundline_max_exclusive = "0.75.0"
canon_min = "0.72.6"
canon_max_exclusive = "0.68.0"
required_profile_family = "blind_review"
admission_priority = "required_before_acceptance"
confidence_handoff_required = true
provenance_ref = "packet:reasoning-posture-123"
```
