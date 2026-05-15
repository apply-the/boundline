# Architecture Guidance

Keep decisions aligned with the owning boundary instead of letting framework or tooling glue own the behavior.

- Change the code that computes or controls the behavior, not only the wiring that forwards to it.
- Keep presentation, application orchestration, domain logic, and infrastructure adapters separate.
- Invert dependencies toward domain interfaces; keep framework, transport, persistence, and runtime details at the edges.
- Reuse existing orchestration, session, tracing, and persistence surfaces before introducing new cross-cutting state.
- Keep transactions, retries, background work, and external calls explicit and narrowly scoped.
- Prefer bounded contexts and explicit ownership over god services or catch-all utility layers.
- Preserve provenance so operator-visible summaries can explain which source or constraint won.
