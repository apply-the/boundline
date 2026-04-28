# Quickstart: Human-Friendly Init and Model Routing

## Prerequisites

- Work from the repository root.
- Start with a workspace that either has no `.synod` directory yet or with a
  workspace you are willing to reconfigure.
- Use `cargo run --bin synod -- ...` when validating locally.
- Treat Claude, Codex, Copilot, and Gemini as the only supported runtime names
  in this slice.

## Scenario 1: Initialize a fresh bug-fix workspace

```bash
cargo run --bin synod -- init \
  --workspace <workspace> \
  --template bug-fix \
  --assistant claude \
  --assistant copilot \
  --yes

cargo run --bin synod -- doctor --workspace <workspace>
```

Expected outcome:

- Synod creates `.synod/execution.json` and `.synod/config.toml` for the workspace.
- The init summary shows which files were created and which assistant surfaces were enabled.
- `doctor` reports the workspace as ready for the bounded execution flow.

## Scenario 2: Set global defaults and override one workspace route

```bash
cargo run --bin synod -- config set \
  --scope global \
  --slot planning \
  --runtime claude \
  --model sonnet-4

cargo run --bin synod -- config set \
  --workspace <workspace> \
  --scope workspace \
  --slot planning \
  --runtime codex \
  --model gpt-5-codex

cargo run --bin synod -- config show \
  --workspace <workspace> \
  --scope effective
```

Expected outcome:

- The global config persists a planning default for future repositories.
- The workspace config overrides only the planning route for this repository.
- Effective config output shows that planning comes from workspace scope while
  unrelated slots still inherit from global or built-in defaults.

## Scenario 3: Configure distinct review and adjudication routes

```bash
cargo run --bin synod -- config set \
  --workspace <workspace> \
  --scope workspace \
  --reviewer safety \
  --runtime copilot \
  --model gpt-5.4

cargo run --bin synod -- config set \
  --workspace <workspace> \
  --scope workspace \
  --reviewer maintainability \
  --runtime claude \
  --model sonnet-4

cargo run --bin synod -- config set \
  --workspace <workspace> \
  --scope workspace \
  --adjudicator \
  --runtime codex \
  --model gpt-5-codex

cargo run --bin synod -- config show \
  --workspace <workspace> \
  --scope effective
```

Expected outcome:

- Synod stores separate routing for named reviewer roles and the adjudicator.
- Effective config output shows reviewer-specific routes overriding the review default.
- The adjudicator route remains distinct from both reviewer roles.

## Scenario 4: Rerun init safely on an existing workspace

```bash
cargo run --bin synod -- init \
  --workspace <workspace> \
  --template change
```

Expected outcome:

- Synod previews which files already exist and what would change.
- If the rerun would overwrite existing config or assistant assets, Synod asks
  for explicit confirmation or exits without changing the repository.
- No existing file is silently replaced.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Expected outcome:

- Init and config commands validate runtime capability, config precedence, and
  review-role routing.
- Existing manifest-driven workflows still work for advanced automation.
- Documentation and assistant guidance reflect the same init-first operator path.