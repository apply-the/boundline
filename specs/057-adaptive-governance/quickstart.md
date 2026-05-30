# Quickstart: Control Graduation And Adaptive Governance

## 1. Prepare A Governed Boundary

Start from a workspace where the governance runtime can return compatible Canon
`authority-governance-v1` metadata. If Canon also returns
`adaptive-governance-v1`, treat it as optional companion semantics.

Expected result:
- Boundline can recover the required authority posture baseline.
- Boundline may also recover optional adaptive-governance state and rollout-profile semantics.
- Missing optional companion semantics do not block first-slice runtime behavior by themselves.

## 2. Capture Or Reuse A Governed Goal

Run:

```bash
boundline goal --goal "stabilize the governed delivery boundary"
```

Expected result:
- the session stores a bounded goal and remains eligible for the normal session-native workflow.

## 3. Build The Plan

Run:

```bash
boundline plan
```

Expected result:
- Boundline can surface the consumed Canon contract lines before execution continues.
- newly enabled or low-trust governance starts from an explicit advisory posture instead of silently assuming stronger enforcement.

## 4. Evaluate The Boundary

Run:

```bash
boundline run
```

Expected result:
- Boundline resolves one explicit runtime governance state and one explicit rollout profile for the current boundary.
- low-confidence or unsupported conditions produce explicit degradation, escalation, or stop outcomes instead of silent weakening.
- a missing required `authority-governance-v1` baseline still fails closed when governance is required.

## 5. Inspect The Runtime Projection

Run:

```bash
boundline status
boundline next
boundline inspect
```

Expected result:
- `status` and `next` surface the current governance state, rollout profile, and next required action.
- `inspect` shows the contract lines consumed, confidence rationale, trust posture, and any degradation or escalation outcome.
- projection keeps the required baseline and optional companion distinct, including `adaptive_contract_line: unavailable` when the companion is optional and absent.
- if the optional adaptive companion contract is absent or unsupported, that fact remains visible separately from the required baseline compatibility state.