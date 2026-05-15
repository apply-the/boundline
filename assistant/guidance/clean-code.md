# Clean Code

Favor code that stays readable, testable, and low-risk after the first edit.

- Make intent, ownership, side effects, error flow, and invariants obvious in names and control flow.
- Prefer one clear responsibility per function or module; split when responsibilities diverge, not to chase arbitrary line counts.
- Keep domain language consistent across types, functions, logs, and UI or API surfaces.
- Replace magic numbers, magic strings, and implicit policy with named constants, enums, or typed value objects.
- Separate parsing, validation, domain decisions, persistence, notifications, and transport glue instead of mixing them in one handler.
- Make common mistakes hard through types, constructors, discriminated states, and narrow APIs.
- Keep diffs bounded and reviewable; do not broaden scope with incidental rewrites.
- Preserve stable contracts unless the bounded task explicitly requires a contract change.
