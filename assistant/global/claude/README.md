# Boundline Global Commands For Claude

User-scoped package for readiness and status before a repository has repo-local
command files. Bootstrap itself remains a raw CLI step.

Commands:
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
- `boundline update --workspace <workspace> --target assistant --apply`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
