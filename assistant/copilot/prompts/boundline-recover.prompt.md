---
description: "Recover a Boundline session from runtime state"
---

# Command: /boundline-recover

Shared guidance: `assistant/README.md`

Recover this Boundline session using the real runtime state.

Ask the user for the workspace if it is missing, then run or ask them to run:

```bash
cargo run --bin boundline -- status --workspace <workspace> --json
```

Use `.boundline/session.json` and the pasted CLI output as authoritative. Preserve `next_command`, `latest_checkpoint_restore_command`, `corrected_command`, and any blocked, clarification-required, failed, exhausted, or terminal state. Do not infer recovery from chat history.
