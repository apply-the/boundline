# Data Model: Multi-Workspace Orchestration

## WorkspaceCluster

- Purpose: Represents the bounded multi-workspace delivery context anchored in
  one primary workspace.
- Fields:
  - `cluster_id`: Stable human-readable identifier for the cluster.
  - `primary_workspace_ref`: Canonical path to the workspace that owns
    `.synod/cluster.toml`.
  - `members`: Ordered list of `ClusterMemberRegistration` values.
  - `routing`: Optional cluster-scoped routing defaults.
  - `created_at`: Millisecond timestamp for cluster creation.
  - `updated_at`: Millisecond timestamp for the latest cluster mutation.
- Validation rules:
  - `cluster_id` must be non-empty.
  - `members` must contain at least two distinct canonical workspace paths.
  - `primary_workspace_ref` must also appear in `members`.
  - Member paths must be unique after canonicalization.

## ClusterMemberRegistration

- Purpose: Identifies one workspace that belongs to the cluster.
- Fields:
  - `workspace_ref`: Canonical workspace path.
  - `display_name`: Optional user-facing label.
  - `role`: `primary` or `member`.
- Validation rules:
  - Exactly one member must use `role = primary`.
  - `display_name`, when present, must be non-empty after trimming.

## ClusterSessionProjection

- Purpose: Captures the cluster identity that an active primary-workspace
  session is operating under.
- Fields:
  - `cluster_id`: Referenced cluster identifier.
  - `primary_workspace_ref`: Canonical path to the primary workspace.
  - `member_workspace_refs`: Ordered list of canonical member paths.
  - `started_from_command`: The command that created or reused the cluster
    projection.
  - `updated_at`: Millisecond timestamp.
- Validation rules:
  - `member_workspace_refs` must match the current cluster membership order.
  - `cluster_id` and `primary_workspace_ref` must point to a valid
    `WorkspaceCluster`.

## ClusterMemberStatus

- Purpose: Summarizes the current state of one cluster member for status and
  inspection output.
- Fields:
  - `workspace_ref`: Canonical workspace path.
  - `session_state`: `healthy`, `missing_session`, `blocked`, `mismatched`, or
    `invalid`.
  - `latest_status`: Optional projection of the member’s latest session status.
  - `latest_trace_ref`: Optional latest relevant trace path.
  - `headline`: User-facing summary of the member state.
- Validation rules:
  - `session_state = healthy` requires a non-empty `headline`.
  - `session_state = mismatched` requires a mismatch reason.

## ClusterRoutingConfig

- Purpose: Stores inherited defaults that apply across cluster members unless a
  higher-precedence source overrides them.
- Fields:
  - `planning`: Optional route.
  - `implementation`: Optional route.
  - `verification`: Optional route.
  - `review`: Optional route.
  - `adjudication`: Optional route.
  - `reviewer_roles`: Ordered reviewer-role overrides.
- Validation rules:
  - The same route validation rules as workspace/global config apply.
  - Duplicate reviewer role identifiers are invalid.

## ClusterInspectReport

- Purpose: The aggregated inspectable output for a clustered view.
- Fields:
  - `cluster_id`: Referenced cluster identifier.
  - `primary_workspace_ref`: Canonical primary workspace path.
  - `members`: Ordered list of `ClusterMemberStatus` values.
  - `active_cluster_session`: Optional `ClusterSessionProjection`.
  - `generated_at`: Millisecond timestamp.
- Validation rules:
  - Every current cluster member must appear exactly once in `members`.
  - `active_cluster_session`, when present, must reference the same cluster.

## Relationships

- One `WorkspaceCluster` contains two or more `ClusterMemberRegistration`
  values.
- One `ClusterSessionProjection` points back to exactly one `WorkspaceCluster`.
- One `ClusterInspectReport` is derived from one `WorkspaceCluster` plus the
  latest per-member session and trace state.
- One `WorkspaceCluster` may contain one optional `ClusterRoutingConfig`.

## Persistence Notes

- `WorkspaceCluster` and `ClusterRoutingConfig` persist together in
  `.synod/cluster.toml` under the primary workspace.
- `ClusterSessionProjection` persists inside the primary workspace active
  session record so clustered flows remain discoverable through existing session
  surfaces.
- `ClusterMemberStatus` and `ClusterInspectReport` are derived views and do not
  require their own file format.