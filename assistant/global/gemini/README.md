# Boundline Global Fallback For Gemini

Support mode: `manual-fallback`.

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

Gemini guidance should stay CLI-first and must not imply repo-local packaged
parity. Use exact Boundline CLI output for status, inspect, explain-plan, and
follow-up routing.

Fallback CLI:
- `boundline init --assistant gemini`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
