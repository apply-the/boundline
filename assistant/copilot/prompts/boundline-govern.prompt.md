---
description: "Use optional Canon governance when configured"
---

# Command: /boundline-govern

Shared guidance: `assistant/README.md`

Use Canon governance only when this Boundline workspace is configured for it.

Ask the user for the workspace if it is missing, then run or ask them to run:

```bash
cargo run --bin boundline -- config show --workspace <workspace> --scope workspace --json
```

Canon governance is conditional and must not appear as the normal delivery path when governance is not configured. Preserve `.boundline/session.json`, `next_command`, and any blocked, clarification-required, failed, exhausted, or terminal state before suggesting governed actions.
