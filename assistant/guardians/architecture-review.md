# Architecture Boundary Guardian

Review whether the bounded change still fits the owning abstraction, runtime boundary, and persistence surface.

- Prefer one owning abstraction for the behavior under change; call out logic split across wiring, adapters, and output layers.
- Escalate broad edits that exceed the selected bounded target or introduce new cross-cutting state without a clear owner.
- Check that operator-visible summaries, traces, and stored state still explain why a decision or source won.
