# Data Model: Checkpoint Rewind

## Checkpoint Manifest

- **Purpose**: Represents one persisted checkpoint created before a bounded
  mutating `run` or `step` action.
- **Fields**:
  - `checkpoint_id`
  - `workspace_ref`
  - `trigger_command`
  - `session_id`
  - `task_id`
  - `step_id`
  - `created_at`
  - `authority_scope`
  - `group_id`
  - `captured_files`
  - `restore_history`
- **Validation rules**:
  - `checkpoint_id`, `workspace_ref`, and `trigger_command` must be non-empty.
  - Each manifest must belong to exactly one owning workspace.
  - A clustered manifest may belong to a group but still has exactly one local
    owning workspace.

## Checkpoint File Record

- **Purpose**: Represents one captured workspace-relative file path inside a
  checkpoint.
- **Fields**:
  - `path`
  - `workspace_ref`
  - `capture_state`
  - `snapshot_ref`
  - `content_hash`
- **Capture states**:
  - `pre_existing`
  - `newly_created`
  - `deleted`
  - `already_modified`
- **Validation rules**:
  - `path` must remain workspace-relative.
  - `snapshot_ref` must exist when restore requires file contents.
  - `deleted` records do not require a live file at restore time.

## Checkpoint Restore Record

- **Purpose**: Represents one explicit restore attempt against a checkpoint.
- **Fields**:
  - `restore_id`
  - `requested_at`
  - `mode`
  - `outcome`
  - `conflicting_paths`
  - `restored_paths`
- **Modes**:
  - `safe`
  - `forced`
- **Outcomes**:
  - `succeeded`
  - `refused`
  - `failed`
- **Validation rules**:
  - `safe` restore may not produce `succeeded` if unrelated conflicts remain.
  - Refused restore must name at least one conflicting path.

## Checkpoint Group

- **Purpose**: Links the primary-workspace checkpoint with any member-workspace
  checkpoints created for the same clustered mutating action.
- **Fields**:
  - `group_id`
  - `primary_workspace_ref`
  - `member_checkpoints`
  - `created_at`
- **Validation rules**:
  - Every member checkpoint must belong to one registered workspace in the
    current cluster projection.
  - The primary workspace remains the authoritative cluster owner.

## Checkpoint Projection

- **Purpose**: The operator-facing summary surfaced through `run`, `status`,
  `next`, `inspect`, and `checkpoint list`.
- **Derived fields**:
  - `latest_checkpoint_id`
  - `latest_checkpoint_workspace`
  - `latest_checkpoint_scope`
  - `restore_command`
  - `checkpoint_conflict_summary`
- **Validation rules**:
  - Projection must not blur native, clustered, or compatibility authority.
  - A failed or blocked mutating run with a checkpoint must provide a restore
    cue.

## Workspace Layout

- **Purpose**: Captures the repository refoundation needed to support the slice.
- **Members**:
  - `boundline-core`
  - `boundline-adapters`
  - `boundline-cli`
- **Validation rules**:
  - Repo-root cargo commands remain valid.
  - Core must not depend on adapters or CLI.
  - Adapters may depend on core but not CLI.
  - CLI may depend on both core and adapters.