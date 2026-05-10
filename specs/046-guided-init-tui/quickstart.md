# Quickstart: Guided Init TUI and Runtime Catalog

## Goal

Validate the guided `init` redesign by exercising version output, interactive
bootstrap, non-interactive parity, custom routing, and progress feedback.

## Scenario 1: Print Version Without a Subcommand

1. Run:

   ```bash
   cargo run --bin boundline -- --version
   cargo run --bin boundline -- -V
   ```

2. Verify both invocations exit successfully and print the current Boundline version.

## Scenario 2: Complete Guided Init With Defaults and One Route Edit

1. Create an empty temporary workspace.
2. Run:

   ```bash
   cargo run --bin boundline -- init --workspace <workspace>
   ```

3. In the guided flow:
   - choose a Canon approval mode from the select prompt
   - choose one or more assistant surfaces from the multi-select prompt
   - review the proposed route table
   - edit one route slot
   - accept the remaining defaults
   - confirm the final summary
4. Verify:
   - no comma-separated freeform route entry is required
   - no raw escape sequences appear during interaction
   - `.boundline/execution.json` and `.boundline/config.toml` are written
   - assistant assets are written when surfaces are selected

## Scenario 3: Recover From Invalid Custom Route Input

1. Start guided init in an empty workspace.
2. Open one route slot editor.
3. Choose a custom model identifier and provide an invalid or blank value.
4. Verify the prompt remains on the current step, shows a contextual correction message, and allows a valid replacement without restarting the command.

## Scenario 4: Run Non-Interactive Init With Explicit Flags

1. Run:

   ```bash
   cargo run --bin boundline -- init \
     --workspace <workspace> \
     --non-interactive \
     --canon-mode-selection auto-confirm \
     --assistant copilot \
     --assistant codex \
     --route planning=copilot:gpt-5.4 \
     --route implementation=codex:gpt-5.4-codex \
     --route verification=copilot:gpt-5.4 \
     --route review=copilot:gpt-5.4
   ```

2. Verify the command completes without prompts and produces the same stored route selections that the guided flow would have written.

## Scenario 5: Observe Progress Feedback Safely

1. Run a bootstrap scenario that performs enough file and assistant-asset work to trigger progress feedback.
2. Verify:
   - interactive terminals show spinner-style or equivalent single-line progress during long steps
   - redirected output does not contain spinner frames or raw cursor-control artifacts
   - completion, cancellation, and failure states end the progress activity cleanly

## Validation Commands

Use focused validation while implementing the slice:

```bash
cargo test --test integration init_bootstrap_flow
cargo test --test contract init_cli_contract
cargo test -p boundline-cli init::tests
cargo test --no-run --all-targets --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
```
