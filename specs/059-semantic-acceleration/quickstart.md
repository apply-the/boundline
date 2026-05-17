# Quickstart: Advanced Context Intelligence Semantic Acceleration

## 1. Prepare A Workspace With The S5 V1 Baseline

Start from a workspace where the S5 V1 advanced-context baseline is already
operational and the workspace-local retrieval index exists.

Expected result:
- Boundline already has authoritative structured runtime context.
- The workspace can build plans on the normal session-native path without any
  semantic acceleration requirement.
- Compatible Canon artifacts are optional enrichment, not a startup
  prerequisite.

## 2. Confirm The Effective Advanced-Context Posture

Run:

```bash
boundline config show --scope effective --workspace <workspace>
```

Expected result:
- the effective configuration shows the V1 advanced-context baseline plus the
  dedicated `semantic_acceleration` policy for the active workspace
- semantic acceleration is visible as `disabled` or `local`
- the authority order still reads as structured runtime context, Canon-governed
  memory, workspace overrides, then semantic expansion

## 3. Capture A Bounded Goal

Run:

```bash
boundline capture --goal "recover the most relevant local evidence for a bounded change" --workspace <workspace>
```

Expected result:
- the active session stores a bounded goal that can exercise hybrid retrieval
- the session remains on the normal session-native path

## 4. Verify The Baseline Path With Semantic Acceleration Disabled

Run:

```bash
boundline plan --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:
- Boundline builds the plan with the existing V1 FTS and structured path only
- `status` and `inspect` show that semantic acceleration is disabled or not in
  use
- no hidden semantic dependency is required for successful bounded planning

## 5. Re-Run With Local Semantic Acceleration Enabled

Enable local semantic acceleration in the workspace configuration, then run:

```bash
boundline config set-semantic-acceleration --scope workspace --policy local --workspace <workspace>
boundline plan --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:
- the workspace enablement path changes only the dedicated
  `semantic_acceleration` policy and does not redefine the V1
  `advanced_context` baseline
- the runtime still starts from the V1 candidate set and authority order
- semantic acceleration is surfaced as a local additive layer, not a second
  retrieval architecture
- `status` and `inspect` show whether semantic similarity expanded or reranked
  the V1 set and why each semantic candidate was selected, downgraded, rejected,
  or skipped
- Canon-backed evidence preserves artifact class, semantic contract line, and
  provenance reference when it participates

## 6. Force And Observe Explicit Fallback

Re-run the same bounded plan in a workspace where semantic acceleration is
configured for local use but the semantic capability is unavailable,
unsupported, or degraded.

Run:

```bash
boundline plan --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

Expected result:
- Boundline falls back explicitly to the V1 path instead of failing silently
- `status` and `inspect` surface the semantic capability state and fallback
  reason
- the core delivery loop remains usable even when `sqlite-vec` or local
  embeddings cannot participate

Walkthrough note recorded on 2026-05-17:
- On a fresh local Rust workspace, `config show --scope effective` reported
  `advanced_context: mode=local, remote_policy=local_only, ...` plus
  `semantic_acceleration: policy=disabled [built-in]` before workspace opt-in.
- After `boundline config set-semantic-acceleration --scope workspace --policy local`,
  `status` reported `semantic_policy_state: local`,
  `semantic_capability_state: unavailable`, `hybrid_outcome: skipped`, and
  `retrieval_terminal_reason: bounded context remains insufficient after local retrieval; semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval`.
- The ready semantic-expansion path was validated through focused adapter-local
  tests in this repo; the end-to-end CLI walkthrough on this machine exercised
  the explicit fallback branch instead of a ready semantic capability.

## 7. Verify Canon Compatibility Handling

Repeat the enabled local-semantic run in a workspace that contains both
compatible and incompatible Canon semantic artifacts.

Expected result:
- compatible Canon artifacts may participate in semantic expansion only through
  the documented Canon semantic contract
- incompatible or incomplete Canon artifacts are skipped with explicit reasons
- no Canon artifact can override the structured runtime context or hide its
  compatibility outcome
