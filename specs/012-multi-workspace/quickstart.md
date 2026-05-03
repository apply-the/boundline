# Quickstart: Multi-Workspace Orchestration

## Prerequisites

- Work from the repository root.
- Prepare at least two repositories that already contain Boundline workspace state
  or are ready to be initialized.
- Use `cargo run --bin boundline -- ...` when validating locally.
- Treat the first slice as cluster bootstrap, status/inspection, and inherited
  defaults rather than automatic cross-repository execution planning.

## Scenario 1: Register a cluster

```bash
cargo run --bin boundline -- cluster init \
  --workspace <primary-workspace> \
  --cluster-id delivery-a \
  --member <primary-workspace> \
  --member <secondary-workspace>
```

Expected outcome:

- Boundline creates `.boundline/cluster.toml` in the primary workspace.
- The cluster summary shows the cluster id, primary workspace, and canonical
  member list.
- If a member path is invalid or duplicated, no partial cluster file is left
  behind.

## Scenario 2: Inspect cluster status

```bash
cargo run --bin boundline -- cluster status \
  --workspace <primary-workspace>
```

Expected outcome:

- Boundline lists every member workspace in the cluster.
- Each member is marked explicitly as healthy, missing-session, blocked,
  mismatched, or invalid.
- The report makes it clear which member requires operator action.

## Scenario 3: Inspect cluster traces

```bash
cargo run --bin boundline -- cluster inspect \
  --workspace <primary-workspace>
```

Expected outcome:

- Boundline shows the latest relevant trace reference for each member workspace.
- Members with no trace are surfaced explicitly as a gap.
- The operator can move from the cluster view to the relevant workspace trace
  without guessing paths.

## Scenario 4: Save a cluster-level default and verify precedence

```bash
cargo run --bin boundline -- config set \
  --cluster <primary-workspace> \
  --scope cluster \
  --slot planning \
  --runtime codex \
  --model gpt-5-codex

cargo run --bin boundline -- config show \
  --workspace <secondary-workspace> \
  --cluster <primary-workspace> \
  --scope effective
```

Expected outcome:

- Boundline saves the cluster-level default into `.boundline/cluster.toml`.
- Effective config for the member workspace shows cluster as the source when no
  local override exists.
- Workspace-local values still override the cluster default when present.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Expected outcome:

- Cluster bootstrap, inspection, and config precedence work without breaking
  single-workspace flows.
- Cluster-aware CLI output remains explicit about missing or mismatched member
  state.