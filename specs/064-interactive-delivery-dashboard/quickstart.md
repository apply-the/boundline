# Quickstart: Interactive Delivery Dashboard

## Purpose

Validate that the dashboard gives operators a faithful, actionable view of Boundline delivery state while normal commands remain authoritative.

## Prerequisites

- A local checkout on branch `064-interactive-delivery-dashboard`.
- A built Boundline binary and dashboard entrypoint.
- Representative workspaces with:
  - no active session
  - proposed plan waiting for confirmation
  - confirmed plan ready to run
  - blocked or waiting governed state
  - failed or exhausted state with recovery options
  - optional governed artifact references

## 1. Open A Workspace With No Session

```bash
boundline-dashboard --workspace <empty-workspace>
```

Expected:

- Dashboard starts or emits a degraded state.
- The first screen reports no active session.
- The suggested action matches the normal command path for starting or capturing work.

Cross-check:

```bash
boundline status --workspace <empty-workspace>
```

The dashboard summary and normal status output must agree on the next command.

## 2. Inspect A Proposed Plan

Prepare a workspace with a captured goal and proposed plan.

```bash
boundline-dashboard --workspace <planned-workspace>
```

Expected:

- Goal, plan state, current stage, current step, next action, and confirmation requirement are visible.
- Goal plan and evidence panels show plan revision, targets, selected evidence, and verification strategy.
- The only mutating actions are valid for the current plan state.

Cross-check:

```bash
boundline status --workspace <planned-workspace>
boundline inspect --workspace <planned-workspace>
```

The dashboard must not show a different route owner, next action, or blocking reason.

## 3. Confirm And Continue Through The Dashboard

From the proposed-plan dashboard state, select confirm.

Expected:

- The dashboard applies the same confirmation semantics as the normal command path.
- The refreshed snapshot shows the plan as confirmed.
- The next valid action becomes continue or run.

Cross-check:

```bash
boundline status --workspace <planned-workspace>
```

The confirmed state and next command must match the dashboard.

## 4. Reject Or Replan A Proposed Direction

Prepare a workspace where rejection or replanning is allowed, then select the action and provide a bounded operator reason.

Expected:

- The operator reason is preserved in Boundline-owned state or trace evidence.
- The prior plan and evidence remain inspectable.
- The resulting state is replan requested or explicitly stopped.

Invalid case:

- Repeat the same action from a stale dashboard view after changing the session from another terminal.
- Expected result: the action is refused with a stale-state explanation and a refreshed next action.

## 5. Inspect Findings, Checkpoints, And Governed References

Open a workspace with guidance or guardian findings, checkpoints, and optional governed references.

Expected:

- Findings show status, severity, evidence refs, and unresolved follow-up.
- Checkpoints show available restore refs or explain why no restore path is credible.
- Governed references are read-only.
- Missing governed references are shown as unavailable or degraded, not as dashboard failure.

## 6. Validate Degraded Rendering

Run in a constrained or non-interactive environment.

```bash
boundline-dashboard --workspace <workspace> --snapshot-json
```

Expected:

- Snapshot JSON contains the same summary facts as the interactive screen would use.
- If interactive rendering cannot start, the degraded output names the reason and valid fallback command.

## 7. Validate Branding

Open the dashboard in color and no-color modes.

```bash
boundline-dashboard --workspace <workspace>
boundline-dashboard --workspace <workspace> --no-color
```

Expected:

- The dashboard uses a simple `boundline` terminal wordmark.
- No image, SVG, or wide ANSI banner asset is required.
- No-color mode keeps the same information hierarchy.

## 8. Release Validation

Before release closure, run the repository validation suite:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
cargo nextest run
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected:

- Dashboard contract tests pass.
- Dashboard integration flows cover ready, waiting, blocked, failed, exhausted, degraded, and complete states.
- Modified Rust file coverage meets the repository target.
- Docs, changelog, roadmap, assistant guidance if affected, and version metadata describe the dashboard without roadmap code names.
- The bundled assistant model catalog is reconciled with current provider docs or records a no-change rationale.

Observed on 2026-05-20:

- `cargo test --no-run --all-targets` compiled all workspace targets successfully. In this environment the bare `cargo test` harness still stalls after compilation, so deterministic dashboard validation used targeted suites plus `cargo nextest run`.
- `cargo test --test contract dashboard_` passed with 23 dashboard contract tests.
- `cargo test --test integration dashboard_` passed earlier in the slice for dashboard integration flows; the expanded projection coverage rerun `cargo test --test integration dashboard_snapshot_` passed with 5 snapshot-flow tests.
- `cargo test --test unit dashboard_` passed with the dashboard file-size guardrail test.
- `cargo test -p boundline-dashboard --test ui_behaviors` passed with 7 crate-local UI and CLI entrypoint tests.
- `cargo nextest run` exited `0` for the full workspace.
- `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` generated LCOV coverage. The patch-intersection helper timed out against the new dashboard paths, so file-level LCOV was used as the practical proxy for new-file patch coverage: `crates/boundline-dashboard/src/app.rs` reached `36/37` lines (`97.30%`) and `src/adapters/dashboard_state.rs` reached `382/385` lines (`99.22%`).
- `cargo deny check licenses advisories bans sources` passed after allowing the transitive `Zlib` license and ignoring the archived `paste` advisory that arrives transitively from `ratatui`; the remaining output is duplicate-version warnings only.

Scenario evidence captured on 2026-05-20:

- Empty workspace: `target/debug/boundline-dashboard --workspace /tmp/boundline-dashboard-empty.5VVUuG --snapshot-json` emitted a degraded snapshot with `authority = "degraded"`, no active session, and empty panels.
- Goal-captured workspace: `target/debug/boundline-dashboard --workspace /tmp/boundline-dashboard-planned.EYVEZ9` rendered `mode: interactive`, `condition: Waiting`, and the expected confirm command.
- No-color workspace render: `target/debug/boundline-dashboard --workspace /tmp/boundline-dashboard-planned.EYVEZ9 --no-color` rendered `mode: monochrome` while preserving the same summary facts and proposed-plan guidance.
- Proposed-plan, failed, exhausted, stale-trace, externally changed, governed-reference, diagnostics, degraded, and multiple-session states were exercised by the dashboard contract and integration suites above, including the refusal and refresh flows.

Performance evidence captured on 2026-05-20:

- On the pre-built `target/debug/boundline-dashboard` binary with the prepared `/tmp/boundline-dashboard-planned.EYVEZ9` workspace, first render completed in `real 0.00` seconds.
- An immediate second invocation against the same workspace also completed in `real 0.00` seconds.
- These timings exclude workspace preparation and underlying planning commands; they measure only dashboard process startup, snapshot assembly, and render output.

## Model Catalog Reconciliation

Checked on 2026-05-20 against official provider documentation:

- OpenAI API model docs list `gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, and `gpt-5.4-nano` as current frontier choices, and list `gpt-5.3-codex` as the current capable Codex model.
- GitHub Copilot supported-model docs list `GPT-5.3-Codex`, `GPT-5.4`, `GPT-5.4 mini`, `GPT-5.4 nano`, `GPT-5.5`, Claude 4.5/4.6/4.7 choices, and Gemini 3/3.1/3.5 choices.
- Anthropic model docs list Claude Opus 4.7, Sonnet 4.6, and Haiku 4.5 as the current first-party model family.
- Google Gemini model docs list Gemini 3.1 Pro Preview, Gemini 3.5 Flash, Gemini 3 Flash Preview, Gemini 3.1 Flash-Lite, and continuing Gemini 2.5 models.

Applied catalog delta:

- Updated `/Users/rt/workspace/apply-the/boundline/assistant/catalog/model-catalog.toml` to `catalog_version = "0.64.0"` and `updated_at = "2026-05-20"`.
- Switched Codex default planning and implementation routes from `gpt-5-codex` to `gpt-5.3-codex`.
- Added current Copilot and provider-visible entries while removing stale default-route reliance on deprecated Codex entries.
