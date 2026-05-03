# Quickstart: Bounded Delegated Execution

## Scenario 1: Declare route capability and effort policy before planning

1. Start or resume a session.
2. Configure routed slots and runtime capability declarations for the workspace.
3. Configure effort policy for the slots that should prefer lighter or heavier
   reasoning.
4. Run `synod config show --workspace <repo> --scope effective`.
5. Run `synod capture --workspace <repo> --goal "<goal>"`.
6. Run `synod plan --workspace <repo>`.

Expected result: the effective configuration reports route capability and effort
policy, and the proposed plan shows when those declarations changed the bounded
next action or prevented a direct route from owning execution.

## Scenario 2: Generate a handoff packet on a blocked native step

1. Use a workspace whose current route cannot continue a bounded step credibly
   but another declared route can.
2. Run `synod run --workspace <repo>`.
3. Read `synod status --workspace <repo>` and `synod next --workspace <repo>`.

Expected result: the run stops on an explicit handoff boundary, the session
persists an active handoff packet with decisive evidence and a recommended next
command, and `status` plus `next` show the same continuity story.

## Scenario 3: Generate an escalation packet when no credible continuation exists

1. Use a workspace whose declared routes cannot continue the current bounded
   step inside the configured limits.
2. Run `synod run --workspace <repo>`.
3. Read `synod inspect --workspace <repo>`.

Expected result: the run stops with an escalation-required continuity state,
the active packet names the blocking reason and decisive evidence, and `inspect`
shows that no direct continuation path remained credible.

## Scenario 4: Resolve or supersede stale delegated continuity

1. Produce an active delegation packet from a previous blocked run.
2. Change the route declaration, add new validation evidence, or replan the
   session.
3. Run `synod run --workspace <repo>` or `synod plan --workspace <repo> --replan`.
4. Read `synod status --workspace <repo>` and `synod inspect --workspace <repo>`.

Expected result: the prior packet is resolved or superseded explicitly, the new
continuity state names why authority changed, and history remains inspectable.

## Scenario 5: Detect a stuck delegation loop

1. Produce a blocked delegated continuity state.
2. Repeat the same continuation attempt until the configured repeated-block
   threshold is reached without new decisive evidence.
3. Run `synod next --workspace <repo>` and `synod inspect --workspace <repo>`.

Expected result: Synod reports a stuck continuity state, recommends a bounded
recovery action such as replan or escalation resolution, and does not keep
repeating the same blocked action silently.

## Scenario 6: Release validation for 0.37.0

1. Run `cargo fmt --all`.
2. Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Run targeted unit, integration, and contract coverage for the changed slices.
4. Run `cargo test --no-run --all-targets`.
5. Run `cargo nextest run --workspace --all-features`.
6. Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
7. Verify modified and new Rust files remain above 95% coverage.

Expected result: the release ships as `0.37.0` with docs, roadmap, assistant
guidance, changelog, and validation outputs aligned to delegated execution.