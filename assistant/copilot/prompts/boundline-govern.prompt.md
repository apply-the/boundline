---
description: "Use optional Canon governance through Boundline"
---

# Command: /boundline-govern

Shared guidance: `assistant/README.md`

Use Canon governance through Boundline's governed stage surface.

Ask the user for the workspace. Ask for a desired mode only if they already want to choose one explicitly. If no mode was supplied, use the CLI choice-rendering path instead of inventing a per-mode alias.

```bash
cargo run --bin boundline -- govern --workspace <workspace> --json
```

Append `--mode <mode>` only when the user already named one.

Canon produces governed packets while Boundline owns orchestration. Preserve `.boundline/session.json`, governed stage refs, packet refs, approval or missing-input state, `next_command`, and any blocked, clarification-required, failed, exhausted, or terminal state before suggesting follow-up actions.
