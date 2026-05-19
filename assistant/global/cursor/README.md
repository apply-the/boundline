# Boundline Global Commands For Cursor

Support mode: `copy-ready-assets`.

Cursor support is represented as copy-ready command and rule assets because
Cursor installations can differ by environment.

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

Cursor guidance should not imply repo-local full parity or self-registering
installation. The generated assets are copy-ready and the CLI remains the
authoritative runtime surface for status, inspect, explain-plan, and follow-up
guidance.

Fallback CLI:
- `boundline init`
- `boundline doctor`
- `boundline doctor --workspace <workspace>`
- `boundline status`
- `boundline continue`
