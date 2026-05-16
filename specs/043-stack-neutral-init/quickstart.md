# Quickstart: Stack-Neutral Workspace Entry

## Scenario 1: Empty Repository, Stack Chosen Later

1. Create an empty repository directory and initialize Git.
2. Run `boundline init --assistant copilot`.
3. Confirm `.boundline/config.toml` exists and shows Copilot plus auto-seeded route defaults.
4. Run `boundline doctor`.
5. Run `boundline start`, `capture`, and `plan` on the same workspace.
6. Expected result: the workspace is accepted without a Rust-specific manifest; planning either proposes a credible path or stops with explicit clarification.

## Scenario 2: Non-Rust Repository Uses Direct Native Run

1. Create a repository with Python or Node source files and no `Cargo.toml`.
2. Run `boundline doctor`.
3. Run `boundline run --goal "Bootstrap the repository"`.
4. Expected result: readiness passes, native direct-run enters the same primary flow, and any stop condition is about task credibility rather than a missing Rust manifest.

## Scenario 3: Assistant Defaults Seeded Automatically

1. Run `boundline init --assistant claude` in a fresh workspace.
2. Inspect `.boundline/config.toml`.
3. Expected result: route slots are filled with Claude's default model unless explicit routes were provided.
4. Override one route explicitly and rerun with `--force`.
5. Expected result: the explicit route wins and the remaining slots keep deterministic defaults.

## Scenario 4: Domain-Driven Hygiene Defaults

1. Create a repository with `package.json`, `Dockerfile`, and frontend source files.
2. Run `boundline init --domain react --assistant codex`.
3. Inspect `.gitignore` and `.dockerignore`.
4. Expected result: universal, web, and Docker patterns are added; unrelated Rust-only patterns are absent.
5. Rerun `init --force` after adding a custom ignore line.
6. Expected result: Boundline preserves the custom line while adding any newly credible defaults.
