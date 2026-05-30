# Quickstart: Advanced Context Intelligence

## 1. Prepare A Workspace With Runtime Context

Start from a workspace where Boundline already has the runtime-intelligence
substrate available and may also have compatible Canon-promoted artifacts.

Expected result:
- Boundline has structured runtime context it can treat as authoritative.
- Canon artifacts are optional enrichment, not a required dependency.
- The workspace can run in `disabled`, `local`, or explicitly enabled `remote`
  retrieval mode without changing the primary session-native workflow.

## 2. Confirm The Effective Retrieval Posture

Run:

```bash
boundline config show --scope effective --workspace <workspace>
```

Expected result:
- the effective configuration shows whether advanced retrieval is disabled,
  local-only, or explicitly permitted to use a remote provider.
- the output makes the authority order visible: structured runtime context,
  Canon-governed memory, workspace overrides, then semantic expansion.
- bounded retrieval limits and remote-disclosure policy are inspectable before
  planning starts.

## 3. Capture Or Reuse A Goal

Run:

```bash
boundline goal --goal "stabilize the delivery context for a bounded change" --workspace <workspace>
```

Expected result:
- the active session stores a bounded goal that later retrieval decisions can
  reference.
- the session remains on the normal session-native path.

## 4. Build A Plan With Expanded Context

Run:

```bash
boundline plan --workspace <workspace>
```

Expected result:
- Boundline retrieves additional repository or Canon-backed evidence only after
  considering structured authoritative inputs.
- the plan surfaces any selected evidence, rejected evidence, relationship
  projections, and impact findings with visible rationale.
- if advanced retrieval is unavailable or blocked, the runtime degrades
  explicitly to structured-only behavior or stops with an explicit reason.

## 5. Inspect Retrieval And Impact Projection

Run:

```bash
boundline status --workspace <workspace>
boundline next --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:
- `status` and `next` show the active retrieval mode, the most relevant impact
  findings, and the next required delivery action.
- `inspect` shows why evidence was selected or rejected, which relationships
  were projected, and whether any Canon artifacts were used or skipped.
- surfaced reasoning stays explainable and attributable to persisted runtime
  state rather than hidden heuristics.

## 6. Verify Remote Mode Stays Explicit

Re-run `boundline plan --workspace <workspace>` in a workspace that does not
permit remote semantic expansion.

Expected result:
- local or structured-only retrieval still works.
- any attempt to use a remote provider is surfaced as blocked, unavailable, or
  not enabled rather than silently transmitting source code or Canon-backed
  artifacts.
- the failure path remains inspectable through `status` or `inspect`.