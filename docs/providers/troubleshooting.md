# Provider Troubleshooting

Use this page when provider-backed execution does not reach the expected ready
state.

## Registration Succeeds But Activation Does Not

Likely causes:

- missing setup requirements
- provider health failure
- incompatible lifecycle support
- metadata conflict against Boundline runtime policy

Check:

```bash
boundline provider show --workspace <workspace> --json
boundline provider health --workspace <workspace>
boundline status --workspace <workspace> --json
```

## Execution Stops Before `execute`

Boundline intentionally blocks before `execute` when:

- readiness is degraded or unavailable
- required permissions exceed the runtime policy
- `prepare` reports missing required context or evidence
- a specialized profile conflicts with the generic provider contract

This is a bounded stop, not an implicit retry signal.

## Evidence Was Rejected

Inspect the session or latest trace:

```bash
boundline inspect --workspace <workspace> --json
```

Look for:

- provider failure class
- validation disposition
- accepted evidence refs
- rejected evidence refs
- provider limitations

If the runtime rejected a patch proposal or evidence set, repair the provider
or route back to planning instead of forcing execution forward.
