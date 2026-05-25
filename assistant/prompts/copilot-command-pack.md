# Boundline Copilot Command And Prompt Pack

Copilot support is documented as a prompt and command pack. This repository does not claim a universal Copilot plugin format for Boundline.

Use these prompts with the Boundline CLI installed in the target repository. In Copilot Chat, Boundline still runs through the Rust CLI/runtime; the prompt pack is guidance, not a second orchestration engine. `.boundline/session.json` remains authoritative, and pasted CLI output should be treated as the source for `next_command`, blocked, clarification-required, failed, exhausted, and terminal state.

If a Copilot surface cannot run shell commands directly, fall back to copyable CLI commands plus pasted runtime output. Do not treat chat history alone as execution evidence.

| Task | Prompt | Boundline Surface |
|------|--------|-------------------|
| Open session goal | Open a new Boundline session by stating the goal for this workspace. | `boundline orchestrate --goal "<goal>" --assistant-host copilot --until phase-request --json-stream` |
| Update goal | Refine the active Boundline session goal from new goal text or a brief. | `boundline orchestrate --goal "<goal>" --assistant-host copilot --until phase-request --json-stream` |
| Plan work | I want to turn this idea into a bounded implementation plan. | `boundline plan --json` |
| Run work | Run the next bounded Boundline action. | `boundline run --json` |
| Inspect status | Continue the active Boundline session. | `boundline status --json` |
| Inspect trace | Inspect the latest Boundline trace and tell me the next safe action. | `boundline inspect --json` |
| Diagnose context | Diagnose missing workspace context, provider readiness, indexes, and evidence gaps for this workspace. | `boundline doctor --workspace <workspace>` |
| Recover | Recover this Boundline session using the CLI-reported next_command or checkpoint restore command. | `boundline status --json` |
| Govern | Use Canon governance only if this Boundline workspace is configured for it. | `boundline config show --scope workspace --json` |

Canonical guidance: run or request one CLI command at a time, preserve the runtime output, and do not infer success from chat history. Canon governance is conditional and must not appear as the normal delivery path when governance is not configured.

Interactive contract: when runtime output reports clarification questions,
missing clarification fields, phase requests, approval waits, or other blocked
gates, Copilot should switch to interactive follow-up instead of printing a
static list. Ask concise questions, wait for user answers, then run the
CLI-reported `next_command`, `assistant_resume_command`, or raw `resume_command`.
When a structured goal `phase_request` is present, preserve
`phase_request.request_id`, ask exactly `phase_request.question`, and resume the
orchestrator with the emitted `request_id` plus `--answer "<answer>"`.
