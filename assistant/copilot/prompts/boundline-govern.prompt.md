---
description: "Use optional Canon governance through Boundline"
---

# Command: /boundline-govern

Shared guidance: `assistant/README.md`

Use Canon governance through Boundline's governed stage surface.

Ask the user for the workspace and desired mode if either is missing, then run or ask them to run:

```bash
cargo run --bin boundline -- govern --workspace <workspace> --mode <mode> --json
```

Canon produces governed packets while Boundline owns orchestration. Preserve `.boundline/session.json`, governed stage refs, packet refs, approval or missing-input state, `next_command`, and any blocked, clarification-required, failed, exhausted, or terminal state before suggesting follow-up actions.
