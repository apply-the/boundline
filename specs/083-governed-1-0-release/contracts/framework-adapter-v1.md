# FrameworkAdapterV1 Contract

## Transport

- One local subprocess per request.
- One JSON request on stdin.
- Exactly one framed JSON response on stdout.
- Diagnostics only on stderr.
- Process tree must terminate before the invocation is complete.

Stable operations:

```text
describe
preflight
execute_stage
emit_hook
```

`preflight` is non-mutating. `emit_hook` is advisory and non-authoritative;
hook failure is non-blocking unless the invocation explicitly declares a
blocking hook admitted by Boundline.

## Authority

Every result is a proposal. The adapter cannot:

- Approve its own output.
- Publish or commit.
- Update Boundline state directly.
- Satisfy completion through self-reporting.
- Waive challenge, proof, or authority requirements.

## Capability admission

Before invocation, Boundline binds the adapter to:

- Read/write path allowlists.
- Executable and command allowlists.
- Explicit environment and named-secret allowlists.
- Denied-by-default network access.
- Bounded child-process policy.
- Time, memory, process, stdout, stderr, and file-output limits.
- An invocation identifier, transaction revision, and fencing token.

Access to the authoritative worktree, state root, credentials, unrelated user
directories, direct Git ref updates, detached/background children, and nested
delegation is denied.

## Speckit qualification

Stable-stage candidates:

```text
requirements
planning
implementation proposal
```

Preview:

```text
clarification
checklist generation
task decomposition
```

Speckit remains prerelease if any stable-stage candidate fails protocol,
security, failure-path, evidence, or semantic-projection qualification.
