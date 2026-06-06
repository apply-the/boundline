# help-next

Inspect the current workspace and recommend the next action.

## Synopsis

```bash
boundline help-next [--json] [--all]
```

## Description

`help-next` is a read-only diagnostic command. It inspects the workspace state
across six lifecycle phases and returns the next recommended action with an
exact command, reason, and documentation link. It does not mutate any session
state, configuration, or trace files.

## States

| State | Meaning |
|-------|---------|
| `uninitialized` | No `.boundline/` directory exists |
| `initialized` | `.boundline/` exists but no active session |
| `active` | Active session, no blockers |
| `blocked` | Planning analysis or execution gate is blocking |
| `failed` | Session in a terminal failure (or corrupt session file) |
| `ready` | Healthy session — next command available |

## Flags

| Flag | Description |
|------|-------------|
| `--json` | Output as structured JSON for CI/automation |
| `--all` | List all detected issues ordered by priority instead of just the top blocking issue |

## Output

**Human-readable (default)**:

```
State: ready
No blockers found.
Next action: continue execution
Command: boundline run
Why: healthy session — no blockers detected
Docs: wiki/Daily-Operating-Guide
```

**JSON** (`--json`):

```json
{
  "state": "ready",
  "blockers_found": false,
  "primary_issue": null,
  "additional_issues": [],
  "additional_count": 0,
  "recommended_action": "continue execution",
  "recommended_command": "boundline run",
  "reason": "healthy session — no blockers detected",
  "docs_link": "wiki/Daily-Operating-Guide",
  "output_format": "json"
}
```

## Documentation Links

Documentation links are resolved from `.boundline/help-links.toml`, a versioned
TOML file mapping diagnostic keys to wiki paths. Missing keys produce a
non-blocking warning with a generic fallback.

## Structured Event

Every invocation emits a `boundline.help_next.requested` event to the runtime
trace, recording the diagnosed state, diagnostics count, recommended action,
and output format.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Successful inspection (any state, including blocked) |
| 1 | Internal error (corrupt session file, unreadable link map) |

## Examples

```bash
# Check what to do next (human-readable)
boundline help-next

# Machine-readable output for CI
boundline help-next --json | jq '.state'

# List all issues when multiple blockers exist
boundline help-next --all
```
