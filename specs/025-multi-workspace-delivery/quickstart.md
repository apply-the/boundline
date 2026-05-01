# Quickstart: Expand Multi-Workspace Delivery

## Goal

Exercise the complete `0.25.0` operator story: one bounded clustered delivery
run, explicit workspace participation, and cluster-aware follow-up authority.

## Prerequisites

- A primary workspace and at least one member workspace already registered in a
  Synod cluster.
- Local `.synod/` state available in each participating workspace.
- Use `cargo run --bin synod -- ...` from the repository root when validating
  locally.
- Treat direct single-workspace and explicit compatibility execution as
  separate paths; this quickstart focuses on the cluster-aware session-native
  path.

## Flow

1. Register or verify the cluster.
2. Start one clustered session-native delivery task from the primary workspace.
3. Run the bounded task so it can read or mutate more than one member
   workspace.
4. Inspect cluster-aware follow-up status and authority.
5. Validate release docs and repository checks.

## Example Validation Sequence

```text
cargo run --bin synod -- cluster init --workspace <primary-workspace> --cluster-id delivery-b --member <primary-workspace> --member <secondary-workspace>
cargo run --bin synod -- start --cluster <primary-workspace>
cargo run --bin synod -- capture --cluster <primary-workspace> --goal "Implement the shared API and update the dependent client"
cargo run --bin synod -- plan --cluster <primary-workspace> --flow change
cargo run --bin synod -- run --cluster <primary-workspace>
cargo run --bin synod -- status --cluster <primary-workspace>
cargo run --bin synod -- next --cluster <primary-workspace>
cargo run --bin synod -- inspect --cluster <primary-workspace>
cargo run --bin synod -- cluster inspect --workspace <primary-workspace>
```

## Expected CLI Behavior

### Clustered delivery execution

- The clustered run keeps one explicit route owner and one authoritative
  workspace context at a time.
- The reported delivery story makes clear which member workspaces participated
  and whether they were read, mutated, blocked, or skipped.

### Cluster-aware follow-up

- `status`, `next`, and `inspect` keep the same bounded summary family as the
  single-workspace path while naming the authoritative cluster and workspace
  context explicitly.
- If clustered work is blocked or inspect-only, follow-up surfaces identify the
  blocking or authoritative workspace instead of implying another workspace is
  resumable.

### Cluster inspection

- `cluster inspect` remains a cluster-wide summary surface for the member
  workspaces and their latest relevant traces.
- Cluster follow-up does not hide which member workspace produced the current
  authoritative trace or terminal state.

## Validation Checklist

- One clustered run can touch more than one member workspace without creating
  unrelated orchestration owners.
- Follow-up surfaces expose authoritative route, authoritative workspace
  context, workspace participation, execution condition, and recommended next
  action.
- Blocked and inspect-only clustered scenarios remain explicit about which
  workspace is authoritative next.
- Docs, assistant guidance, version metadata, and changelog describe the same
  `0.25.0` clustered delivery behavior as the runtime output.