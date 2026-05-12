# Boundline Global Commands For Codex

User-scoped package for bootstrapping Boundline before a repository has
repo-local command files.

Commands:
- `/boundline:init`
- `/boundline:doctor`
- `/boundline:help`
- `/boundline:status`
- `/boundline:continue`

These commands must read Boundline state through the CLI. Chat history is not
authoritative; `.boundline/session.json` is the active session source of truth
after initialization.

Fallback CLI:
- `boundline init --workspace <workspace> --assistant codex`
- `boundline doctor --workspace <workspace>`
- `boundline status --workspace <workspace>`
- `boundline continue --workspace <workspace>`
