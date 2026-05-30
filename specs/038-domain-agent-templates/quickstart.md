# Quickstart: Domain Agent Templates

## Scenario 1: Initialize a workspace with active domain families

1. Run `boundline init --workspace <repo> --domain systems --domain react --assistant copilot`.
2. Read the terminal output.
3. Run `boundline config show --workspace <repo> --scope workspace`.

Expected result: the workspace is initialized, the selected domain families are
persisted in the local config, and `config show` reports the active workspace
domain settings.

## Scenario 2: Combine shared standards with workspace overrides

1. Configure a shared standards layer for a domain family in global config.
2. Configure a workspace-specific standards overlay for the same family.
3. Run `boundline config show --workspace <repo> --scope effective`.

Expected result: the effective output reports the built-in template plus shared
and workspace standards in precedence order, with the workspace layer winning
on conflicts.

## Scenario 3: Select the correct domain guidance during planning

1. Use a repository containing both Rust and React code.
2. Enable the relevant domain families for the workspace.
3. Run `boundline goal --workspace <repo> --goal "fix the React dashboard regression"`.
4. Run `boundline plan --workspace <repo>`.

Expected result: the proposed plan keeps the normal bounded planning flow, and
the context projection names the React-facing domain guidance rather than a
generic expert story.

## Scenario 4: Replan onto a different domain family in a mixed-stack repository

1. Capture and plan a task targeting one stack in a mixed-stack repository.
2. Change the goal or bounded target so the next plan should focus on another
   stack.
3. Run `boundline plan --workspace <repo>` again.
4. Read `boundline status --workspace <repo>` or `boundline inspect --workspace <repo>`.

Expected result: the applied domain context changes to the newly selected
family or family combination, and the updated source attribution remains
visible on normal CLI surfaces.

## Scenario 5: Block planning when no credible domain guidance exists

1. Use a workspace with no active matching domain family for the current goal,
   or bind a required supporting input that is unavailable.
2. Run `boundline goal --workspace <repo> --goal "update the unsupported target"`.
3. Run `boundline plan --workspace <repo>`.

Expected result: planning stops explicitly because the bounded context is not
credible, and the blocked-domain reason is visible in the planning output.

## Scenario 6: Reuse governed and external supporting inputs

1. Enable governance for a workspace that already has approved design or
   standards artifacts.
2. Bind a relevant external context input for a frontend or mobile family.
3. Run `boundline plan --workspace <repo>` and `boundline inspect --workspace <repo>`.

Expected result: the bounded context surfaces the governed artifact and bound
external input as supporting evidence, and the output states whether the input
was used, skipped, or unavailable.

## Scenario 7: Release validation for 0.38.0

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run targeted unit, integration, and contract tests for the changed slices.
4. Run `cargo test --no-run --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified and new Rust files remain above 95% coverage.

Expected result: the release ships as `0.38.0` with roadmap, docs, assistant
guidance, changelog, formatting, lint, and coverage aligned to domain agent
templates.