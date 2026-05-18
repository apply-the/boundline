# Boundline Global Commands For Claude

User-scoped package for bootstrapping Boundline before a repository has
repo-local command files.

Commands:
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
- `boundline init --assistant claude`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
