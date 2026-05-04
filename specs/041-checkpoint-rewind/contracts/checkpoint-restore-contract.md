# Contract: Checkpoint Restore

## Purpose

Define the observable behavior for explicit checkpoint restore on the main CLI
surface.

## `boundline checkpoint restore <id>`

### Required behavior

- Restore replays the captured file states for the selected checkpoint scope.
- Restore records whether it succeeded, was refused, or failed.
- Restore keeps trace history append-only by recording a restore event instead
  of deleting later traces.

### Safe-refusal behavior

- If unrelated newer edits would be overwritten, restore must refuse by default.
- A refused restore must name the conflicting paths and show the forced restore
  command needed to proceed.

### Forced behavior

- When the operator passes the explicit override, restore may proceed even when
  unrelated newer edits are present.
- Forced restore must still record that the restore was forced.