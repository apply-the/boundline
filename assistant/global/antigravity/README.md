# Boundline Global Fallback For Antigravity

Support mode: `manual-fallback`.

Antigravity support is fallback guidance. Boundline does not claim native
user-scoped global command installation for this host.

Bootstrap remains a raw CLI step: run `boundline init --workspace <workspace>
--assistant antigravity` once, then use `boundline update --workspace
<workspace> --target assistant --apply` to refresh the repo-local Antigravity package surface.

This global bootstrap note only covers pre-init fallback guidance.

Commands to expose manually where supported:
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

Antigravity global guidance should stay CLI-first and must not imply a native
global plugin API. Use exact Boundline CLI output for status, inspect,
explain-plan, and follow-up routing.

Fallback CLI:
- `boundline assistant install --host antigravity --scope user`
- `boundline init --workspace <workspace> --assistant antigravity`
- `boundline update --workspace <workspace> --target assistant --apply`
- `boundline doctor --install`
- `boundline doctor --workspace <workspace>`
- `boundline status --workspace <workspace>`
- `boundline continue --workspace <workspace>`