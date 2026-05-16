# Boundline Global Commands For Cursor

Cursor support is represented as copy-ready command and rule assets because
Cursor installations can differ by environment.

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
- `boundline init`
- `boundline doctor`
- `boundline status`
- `boundline continue`
