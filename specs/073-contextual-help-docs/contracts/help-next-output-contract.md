# Help-Next Output Contract

**Feature**: 073-contextual-help-docs
**Date**: 2026-06-06
**Contract type**: CLI output contract (human-readable and JSON)

## Purpose

Define the output format for `boundline help-next` in both human-readable (default) and `--json` modes.

## Human-Readable Output (default)

```
State: <state label>
[Blockers found: <count>]
---
<primary issue description>
Next action: <recommended action>
Command: <exact command>
Why: <reason>
Docs: <url or "unavailable">
[<N> additional issues detected. Run `boundline help-next --all` to list them.]
```

When `state = ready`:

```
State: ready
No blockers found.
Next action: continue execution
Command: boundline run
Why: the workspace is initialized, the session is active, required configuration is present, and no blocking diagnostics were detected.
Docs: wiki/Daily-Operating-Guide
```

## JSON Output (`--json`)

```json
{
  "state": "blocked",
  "blockers_found": true,
  "primary_issue": {
    "key": "session_blocked_planning",
    "severity": "blocking",
    "message": "Planning analysis is blocked: uncovered success criterion 'SC-004'",
    "source": ".boundline/session.json",
    "command": "boundline plan",
    "docs_key": "session_blocked_planning"
  },
  "additional_issues": [
    {
      "key": "config_missing_key",
      "severity": "warning",
      "message": "Missing config key: provider.default_route",
      "source": ".boundline/config.toml",
      "command": "boundline config set provider.default_route",
      "docs_key": "config_missing_key"
    }
  ],
  "additional_count": 1,
  "recommended_action": "Repair planning coherence then re-run plan",
  "recommended_command": "boundline plan",
  "reason": "Planning analysis blocked with 1 uncovered success criterion",
  "docs_link": "wiki/Troubleshooting#planning-blocked",
  "output_format": "json"
}
```

Healthy-state JSON:

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

## `--all` Flag

When `--all` is specified, the human-readable output lists every detected issue ordered by priority (blocking first, then warnings, then info). The JSON output already includes all issues in `additional_issues` regardless of `--all`.

## Structured Event (`boundline.help_next.requested`)

```json
{
  "event_type": "boundline.help_next.requested",
  "schema_version": "1.0",
  "payload": {
    "state": "blocked",
    "lifecycle_phase": "plan",
    "blocked_category": "planning",
    "diagnostics_count": 2,
    "recommended_action_id": "repair_planning_coherence",
    "recommended_command": "boundline plan",
    "docs_link": "wiki/Troubleshooting#planning-blocked",
    "output_format": "human"
  }
}
```

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | Successful inspection (any state, including blocked) |
| 1 | Internal error (corrupt session file, unreadable link map, etc.) |

`help-next` does NOT use exit code to signal blocked vs healthy — the state is in the output, not the exit code.
