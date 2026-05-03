# Contract: Flow Selection Command

## Command Surface

```text
boundline flow <name>
```

## Supported Names

- `bug-fix`
- `change`
- `delivery`

## Preconditions

- An active session must exist for the current workspace.
- The requested flow name must resolve to a built-in flow definition.
- The session must not already contain an in-progress planned task that would be silently invalidated by replacing the flow.

## State Mutation

On success, the command must:

1. Bind the selected flow to the active session.
2. Initialize the current stage to the first stage of that flow.
3. Reset any stale terminal reason or trace reference that would misrepresent the newly selected flow.
4. Preserve the captured goal already stored in the session.

## Success Output Requirements

- Exit successfully.
- Render the current session summary.
- Include the selected flow name.
- Include the current stage label and stage progress.
- Recommend the next valid command, normally `boundline plan` when a goal is already present.

## Error Cases

### Missing active session

- Command fails with a clear message that a session must be started first.
- Command recommends `boundline start`.

### Unknown flow name

- Command fails with a clear message listing or referencing supported flows.
- Session state is unchanged.

### Flow replacement would invalidate active work

- Command fails if the current session already contains planned or running work for a different flow and the user has not explicitly reset the session.
- Failure explains that flow selection is deterministic and must not silently replace in-flight work.

## Trace Requirements

- Successful flow selection records a visible trace or session event that captures the selected flow and initialized stage.
- Failed flow selection must not create a misleading stage transition event.