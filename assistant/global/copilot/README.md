# Boundline Global Prompt Pack For Copilot

Copilot support is represented as prompt-pack guidance. Boundline does not claim
that every Copilot environment supports user-scoped slash command installation.

Bootstrap itself remains a raw CLI step: run `boundline init --assistant
copilot` once, then use `boundline update --workspace <workspace> --target
assistant --apply` to refresh the repo-local prompt pack.

Prompts:
- `/boundline:doctor`
- `/boundline:help`
- `/boundline:status`
- `/boundline:continue`

Contextual follow-ups after bootstrap:
- `/boundline:explain-plan`
- `/boundline:doctor-context`

These prompts must read Boundline state through the CLI when shell execution is
available, or provide exact fallback commands when it is not. Chat history is
not authoritative; `.boundline/session.json` is the active session source of
truth after initialization.

Fallback CLI:
- `boundline init --assistant copilot`
- `boundline update --workspace <workspace> --target assistant --apply`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
