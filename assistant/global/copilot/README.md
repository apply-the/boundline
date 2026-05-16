# Boundline Global Prompt Pack For Copilot

Copilot support is represented as prompt-pack guidance. Boundline does not claim
that every Copilot environment supports user-scoped slash command installation.

Prompts:
- `/boundline:init`
- `/boundline:doctor`
- `/boundline:help`
- `/boundline:status`
- `/boundline:continue`

These prompts must read Boundline state through the CLI when shell execution is
available, or provide exact fallback commands when it is not. Chat history is
not authoritative; `.boundline/session.json` is the active session source of
truth after initialization.

Fallback CLI:
- `boundline init --assistant copilot`
- `boundline doctor`
- `boundline status`
- `boundline continue`
