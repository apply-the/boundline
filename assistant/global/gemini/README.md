# Boundline Global Fallback For Gemini

Gemini support is fallback guidance. Boundline does not claim native user-scoped
global command installation for Gemini hosts.

Commands to expose manually where supported:
- `/boundline:init`
- `/boundline:doctor`
- `/boundline:help`
- `/boundline:status`
- `/boundline:continue`

Contextual follow-ups after bootstrap:
- `/boundline:explain-plan`
- `/boundline:doctor-context`

These commands must read Boundline state through the CLI. Chat history is not
authoritative; `.boundline/session.json` is the active session source of truth
after initialization.

Fallback CLI:
- `boundline init --assistant gemini`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
