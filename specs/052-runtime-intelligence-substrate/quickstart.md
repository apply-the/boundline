# Quickstart: Runtime Intelligence Substrate

## 1. Capture A Narrow Goal

Run:

```bash
boundline capture --goal "repair the bounded failing test"
```

Expected result:
- the session stores the authored goal and any bounded clarification state.

## 2. Build Or Confirm The Plan

Run:

```bash
boundline plan
```

Expected result:
- Boundline assembles a `ContextPack` from local workspace evidence.
- If Canon capability or memory inputs are available and compatible, they are included as optional context enrichment.
- If context credibility is not `credible`, planning stops with an explicit bounded-context summary.

## 3. Inspect Session-Native Projection

Run:

```bash
boundline status
boundline next
```

Expected result:
- `status` surfaces context summary, credibility, primary inputs, and provenance.
- `next` points the operator toward narrowing or refreshing context when credibility is `stale` or `insufficient`.

## 4. Inspect The Trace

Run:

```bash
boundline inspect
```

Expected result:
- the trace summary reconstructs the current substrate from trace payload fields.
- provenance lines now include the input `source` label for `ContextInput`-derived evidence.

## 5. Verify Canon Compatibility Surface

Run:

```bash
boundline doctor --install
```

Expected result:
- Boundline verifies the supported Canon `0.51.0` machine-facing surface for `canon governance start|refresh|capabilities --json`.
