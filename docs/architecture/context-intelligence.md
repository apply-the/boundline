# Project Memory Structure

Boundline uses two repo-visible document roots to keep reusable inputs and durable outputs separate from runtime state:

- `docs/project/` holds stable project memory that planning and governed delivery can reuse.
- `docs/evidence/` holds consolidated feature outputs and evidence bundles that should remain visible after a delivery cycle completes.

These roots are not the same as runtime storage:

- `.boundline/` keeps session state, traces, checkpoints, and transient governance artifacts.
- `.boundline/context-intelligence/` keeps the derived retrieval DB,
  `manifest.json`, and SQLite WAL/SHM sidecars used by local semantic
  retrieval.
- The large-codebase substrate extends that same derived state with
  repository-map readiness, digest-backed context refs, omission findings, and
  freshness-bound snapshot-cache metadata. None of those derived artifacts are
  reviewed memory or authoritative planning truth.
- `.canon/` keeps raw Canon run packets and Canon-owned runtime payloads.

## Default Layout

```text
docs/
  project/
    README.md
    architecture.md
    domain-language.md
  evidence/
    README.md
    feature-slug/
      summary.md
      validation.md
```

The exact file names can vary. The stable rule is ownership:

- curated reusable inputs belong in `docs/project/`
- durable delivery outputs belong in `docs/evidence/`
- derived semantic index state belongs under `.boundline/`
- transient runtime artifacts do not belong in either folder
- large-codebase snapshot cache state must remain disposable and rebuildable

## Bootstrap Behavior

`boundline init` creates both roots and seeds a README in each one. `boundline update --apply` recreates them when they are missing.

## Related Pages

- [Getting Started](../guide/getting-started)
- [Configuration Reference](../reference/configuration)
- [Canon Integration](../governance/guardians)
- [Architecture And Decisions](./runtime-model)
