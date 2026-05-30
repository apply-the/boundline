# Project Memory and Evidence Structure

Boundline uses two repo-visible document roots to keep reusable inputs and durable outputs separate from runtime state:

- `docs/project/`: stable project memory that planning and governed delivery can reuse across multiple sessions.
- `docs/evidence/`: consolidated feature outputs and evidence bundles that should remain readable after a bounded delivery cycle completes.

These roots are different from runtime-owned storage:

- `.boundline/` keeps session state, traces, checkpoints, and transient governance artifacts.
- `.boundline/context-intelligence/` keeps the derived retrieval DB,
  `manifest.json`, and any SQLite WAL/SHM sidecars used by local semantic
  retrieval.
- `.canon/` keeps raw Canon run packets and Canon-owned runtime payloads.

## Default Layout

```text
docs/
  project/
    README.md
    architecture.md
    domain-language.md
    operating-constraints.md
  evidence/
    README.md
    feature-slug/
      summary.md
      decisions.md
      validation.md
```

Use the layout as a convention, not as a hard schema. The important contract is ownership:

- keep curated, reusable inputs under `docs/project/`
- keep durable, shareable outputs under `docs/evidence/`
- keep the derived semantic index and its sidecars under `.boundline/`
- keep transient runtime and packet state out of both folders

## What Belongs in docs/project

Use `docs/project/` for maintained repository context such as:

- architecture maps
- domain terminology
- operating constraints
- service boundaries
- delivery rules that should survive one feature

This folder is the repo-visible input surface. Boundline and Canon can reference it during planning without treating chat history or transient traces as the long-term source of truth.

## What Belongs in docs/evidence

Use `docs/evidence/` for durable outputs produced during or after a bounded delivery slice, such as:

- feature summaries
- decision records that explain tradeoffs
- validation writeups
- links or references to promoted governed artifacts
- handoff notes that should remain visible in the repository

A common pattern is `docs/evidence/<feature-slug>/...` so each delivery slice has one readable evidence bundle.

## Bootstrap Behavior

`boundline init` creates both roots and seeds a small README in each one. `boundline update --apply` recreates them when they are missing so the repo-visible contract stays discoverable.

If your workspace config remaps project-memory roots, Boundline resolves those configured paths instead of the defaults. The default contract remains `docs/project/` and `docs/evidence/`.
