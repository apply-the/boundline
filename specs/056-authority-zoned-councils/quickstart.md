# Quickstart: Authority-Zoned Delivery Councils

## 1. Prepare A Compatible Governed Packet

Start from a workspace where the governance runtime can return compatible Canon
`authority-governance-v1` metadata with the required fields.

Expected result:
- Boundline can recover `authority_zone`, `change_class`, `intended_persona`, `approval_state`, `packet_readiness`, and `risk` from the Canon boundary.
- Optional provenance such as `stage_role_hints` may be present but is not required.

## 2. Capture Or Reuse A Governed Session Goal

Run:

```bash
boundline goal --goal "finish the governed verification pass"
```

Expected result:
- the session stores a bounded goal and remains eligible for the normal session-native workflow.

## 3. Build The Plan

Run:

```bash
boundline plan
```

Expected result:
- Boundline can surface the consumed Canon contract line and the expected governed boundary context before execution continues.
- if Canon authority semantics are absent and governance is not required, the local-governance compatibility path remains explicit.

## 4. Evaluate A Governed Boundary

Run:

```bash
boundline run
```

Expected result:
- Boundline resolves one explicit effective control class and one bounded council profile for the current boundary.
- the command ends in an explicit proceed, waiting, or stop posture instead of silently downgrading governance.
- missing required Canon control fields or failed reviewer independence produce an explicit blocked or hard-stop result.

## 5. Inspect Session-Native Projection

Run:

```bash
boundline status
boundline next
boundline inspect
```

Expected result:
- `status` and `next` surface the same consumed Canon contract line, control class, council profile, and next action.
- `inspect` shows findings, producer responses, adjudication outcome, stop semantics, and any optional provenance-only Canon metadata separately from the required control decision.
- if the boundary is blocked, the projection makes the blocking reason and required remediation explicit.