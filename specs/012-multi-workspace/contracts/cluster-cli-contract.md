# Contract: Cluster CLI

## Purpose

Define the user-facing contract for registering a bounded multi-workspace
cluster and inspecting its current session and trace state.

## Command Surface

### `boundline cluster init`

```text
boundline cluster init \
  --workspace <primary-workspace> \
  --cluster-id <cluster-id> \
  --member <workspace>... 
```

- `--workspace` identifies the primary workspace that will own
  `.boundline/cluster.toml`.
- `--member` must be provided at least twice and must include the primary
  workspace.
- Initialization must validate all members before persisting the cluster.

### `boundline cluster status`

```text
boundline cluster status \
  --workspace <primary-workspace>
```

- Reads the cluster file from the primary workspace.
- Lists every member workspace and a classified summary of its current state.

### `boundline cluster inspect`

```text
boundline cluster inspect \
  --workspace <primary-workspace>
```

- Reads the cluster file from the primary workspace.
- Surfaces the latest relevant trace reference for every member workspace or an
  explicit missing-trace state.

## Required Behavior

- Cluster init MUST canonicalize member paths before validation.
- Cluster init MUST refuse to save partial membership if any member is invalid,
  duplicated, or not a Boundline workspace.
- Cluster status MUST enumerate every member in the saved cluster file.
- Cluster status MUST classify members explicitly rather than collapsing missing
  or mismatched state into generic success.
- Cluster inspect MUST surface one latest relevant trace reference per member or
  an explicit missing-trace result.

## Validation Rules

- The primary workspace must appear in the member list.
- The member list must contain at least two distinct canonical workspaces.
- If the cluster file is missing or malformed, cluster status and inspect must
  fail with actionable guidance.
- If a member workspace reports session data that does not match the active
  cluster identity, the result must be classified as mismatched.

## Compatibility Rules

- Cluster commands must not change behavior of existing single-workspace CLI
  commands when no cluster file is present.
- Cluster output may reuse existing session and trace summaries but must label
  the workspace each summary belongs to.