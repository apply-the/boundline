# Boundline Copilot Command And Prompt Pack

Copilot support is documented as a prompt and command pack. This repository does not claim a universal Copilot plugin format for Boundline.

Use these prompts with the Boundline CLI installed in the target repository. `.boundline/session.json` remains authoritative, and pasted CLI output should be treated as the source for `next_command`, blocked, clarification-required, failed, exhausted, and terminal state.

| Task | Prompt | Boundline Surface |
|------|--------|-------------------|
| Start session | Start the active Boundline session for this workspace. | `boundline start --json` |
| Capture goal | Capture this goal or brief into the active Boundline session. | `boundline capture --goal "<goal>" --json` |
| Plan work | I want to turn this idea into a bounded implementation plan. | `boundline plan --json` |
| Run work | Run the next bounded Boundline action. | `boundline run --json` |
| Inspect status | Continue the active Boundline session. | `boundline status --json` |
| Inspect trace | Inspect the latest Boundline trace and tell me the next safe action. | `boundline inspect --json` |
| Recover | Recover this Boundline session using the CLI-reported next_command or checkpoint restore command. | `boundline status --json` |
| Govern | Use Canon governance only if this Boundline workspace is configured for it. | `boundline config show --scope workspace --json` |

Canonical guidance: run or request one CLI command at a time, preserve the runtime output, and do not infer success from chat history. Canon governance is conditional and must not appear as the normal delivery path when governance is not configured.
