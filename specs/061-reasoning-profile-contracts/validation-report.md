# Validation Report: Governed Reasoning Profile Contracts

## Status

This feature is complete. This report records the implemented and validated `061-reasoning-profile-contracts` surface so spec, code, release metadata, and the paired Canon companion remain aligned.

`061` is closed as the first release reasoning-profile contract slice, not as the final closure of every conceptual first-wave reasoning profile. The follow-through that completed that closure now lives in [`062-reasoning-profile-closure`](../062-reasoning-profile-closure/spec.md) so the completed implementation can stay explicit without reopening this feature.

## Implemented And Validated

- Boundline `0.61.0` and Canon `0.57.0` release-pair alignment across active runtime constants, distribution metadata, assistant package manifests, and active product docs.
- Typed reasoning-profile activation records, posture windows, and confidence contributions in the Boundline runtime and domain model.
- Session-native reasoning-profile persistence through session status, run output, and trace inspection.
- Governance and session projections now fold reasoning confidence, admission effect, summary, and next action into the existing governance path.
- Routing and governance types now preserve typed reasoning-profile attachment on stage policy selection and expose typed confidence handoff data.
- Local reasoning posture fixtures and deterministic reasoning-profile scenarios are available in `src/fixture.rs` so Boundline stays independently testable without the sibling Canon repository.
- Canon posture projection with current-version compatibility windows derived from the active Boundline and Canon release pair.
- Runtime contract coverage now locks the supported reasoning profile ids, status values, outcome kinds, and participant-role vocabulary for the first release.
- Reasoning-domain helpers now centralize blocking activation states and explicit-reason outcome semantics, with family coverage across self-consistency, heterogeneous review, reflexion, and debate-enabled budgets.
- Verification-stage integration coverage now proves both reasoning-profile activation on `bug-fix:verify` and the unchanged operator path when no reasoning profile is configured.
- Degradation integration coverage now proves insufficient-independence fail-closed behavior and explicit `Interrupted` reasoning-profile projection when Canon approval pauses an interruptible profile.
- Verification-stage integration coverage now also proves `status` and `inspect` surface reasoning participant topology, Canon posture contract, and confidence summary lines for an active profile.
- Reasoning trace contract coverage now locks the additive reasoning event-family vocabulary required for activation, participant lifecycle, disagreement, debate, reflexion, adjudication, confidence, and blocked or escalated trace conditions.
- Reasoning trace unit coverage now proves bounded reflexion exhaustion, bounded debate stagnation, and confidence-contribution projection survive through `summarize_trace` and `render_trace_summary` when driven by the additive reasoning event family.
- Trace-domain helpers now publish explicit reasoning event variants plus a reasoning-family classifier so operator-facing projections can recognize additive reasoning traces without overloading governance or review event groups.
- Runtime reasoning-trace emission now records activation, confidence, blocked, interrupted, and additive iteration lifecycle events through the real `session_runtime.rs` gate and the focused `review_trace.rs` serializer; the planned `src/orchestrator/reasoning_profile.rs` file was not needed.
- The planned `src/orchestrator/reasoning_profile.rs` surface was not needed: reasoning-profile selection and stage attachment are implemented and validated in `src/orchestrator/session_runtime.rs`.
- Reasoning-profile confidence, posture, independence, and next-action rendering on `run`, `status`, and `inspect`-backed trace summaries.
- Contract-drift blocked outcomes are now covered end-to-end in integration by seeding a typed blocked reasoning-profile state; the existing `output.rs` and `inspect.rs` reasoning surfaces already project the mismatch contract line, disagreement summary, remediation next action, and status-level `next_best_action` without additional renderer branching.
- Boundline-side Canon posture contract validation now falls back to a local provider snapshot when the sibling Canon repository is unavailable, preserving independent contract-test execution.
- Canon-side contract coverage now verifies the published provider vocabulary and the Boundline consumer window in `/Users/rt/workspace/apply-the/canon/tests/contract/governed_reasoning_posture_contract.rs`.
- Canon challenge-posture domain validation now fails closed on unsupported contract lines and on compatibility windows that exclude the active Boundline and Canon release pair.
- The actual activation surface in `src/orchestrator/session_runtime.rs` now enforces posture validation during Canon reasoning-profile activation; the planned `src/orchestrator/reasoning_profile.rs` file was not needed.
- Release-facing docs and metadata are now aligned across Boundline and Canon, including the Canon `0.57.0` assistant package manifests, shared plugin metadata, and runtime-compatibility references consumed by host-package validation.
- Final workspace quality gates now pass in both repositories: Boundline `clippy` and full-workspace `llvm-cov`, plus Canon `clippy` and full-workspace `llvm-cov` after correcting stale `0.56.0` package-surface metadata.

## Validation Evidence

### Focused reasoning-profile tests

- `cargo test -p boundline-adapters --lib canon_reasoning_posture_uses_current_release_window`
- `cargo test -p boundline-adapters --lib execute_next_step_reassesses_reasoning_profile_after_routing_changes`
- `cargo test -p boundline-adapters --lib reasoning_route_for_`
- `cargo test governance_confidence_handoff_derives_from_reasoning_profile --workspace`
- `cargo test reviewer_role_routes_can_be_resolved_from_cli_overrides --workspace`
- `cargo test selected_stage_policy_preserves_reasoning_profile_attachment --workspace`
- `cargo test derived_view_projects_governance_confidence_handoff_from_reasoning_profile --workspace`
- `cargo test local_reasoning_posture_fixture_tracks_supported_release_window --workspace`
- `cargo test reasoning_profile_fixture_scenarios_are_deterministic --workspace`
- `cargo test --test contract reasoning_profile_runtime_contract`
- `cargo test activation_status_helpers_cover_blocking_states --workspace`
- `cargo test outcome_kind_helpers_cover_reason_required_states --workspace`
- `cargo test reasoning_budget_accepts_single_participant_self_consistency --workspace`
- `cargo test reasoning_budget_accepts_reflexion_revisions_for_reflexion_family --workspace`
- `cargo test reasoning_budget_rejects_insufficient_participants_for_debate_family --workspace`
- `cargo test independence_floor_rejects_heterogeneous_review_without_distinct_route_or_provider --workspace`
- `cargo test --test integration verification_stage_activation_surfaces_reasoning_profile_lines`
- `cargo test --test integration status_and_inspect_surface_reasoning_profile_summary_details`
- `cargo test --test integration verification_stage_without_reasoning_profile_preserves_unchanged_projection_path`
- `cargo test --test integration insufficient_independence_blocks_reasoning_profile_through_status_and_inspect`
- `cargo test --test integration approval_pending_interrupts_reasoning_profile_through_status_and_inspect`
- `cargo test --test integration governance_autopilot_flow_refreshes_security_assessment_approval_through_status`
- `cargo test --test contract reasoning_trace_contract_lists_required_event_families`
- `cargo test trace_event_type_helpers_cover_reasoning_family --workspace`
- `cargo test --test unit reasoning_profile_trace`
- `cargo test reasoning_trace_records_activation_lifecycle_and_iteration_events --workspace`
- `cargo test --test integration verification_stage_trace_records_reasoning_activation_and_confidence_events`
- `cargo test --test integration contract_drift_blocked_outcome_surfaces_guidance_through_status_and_inspect`
- `cargo test -p boundline-cli --lib reasoning_profile_projection`
- `cargo test -p boundline-cli --lib summarize_trace_rehydrates_reasoning_profile_from_governance_payload`
- `cargo test -p boundline-cli --lib output_helper_functions_cover_review_governance_and_execution_conditions`
- `cargo test compatibility_window_rejects_unsupported_contract_line --workspace`
- `cargo test canon_posture_rejects_incompatible_active_release_pair --workspace`
- `cargo test canon_reasoning_posture_uses_current_release_window --workspace`
- `cd /Users/rt/workspace/apply-the/canon && cargo test --test governed_reasoning_posture_contract`

### Release and contract alignment tests

- `cargo test --test contract distribution_metadata_contract::distribution_metadata_keeps_versions_and_bundle_names_aligned -- --exact`
- `cargo test --test contract distribution_release_surface_contract::release_surface_tracks_current_workspace_version_without_stale_status_heading -- --exact`
- `cargo test --test assistant_plugin_packages metadata_paths_and_versions_are_aligned -- --exact`
- `cargo test --test unit distribution_metadata::supported_distribution_channels_always_include_source_fallback -- --exact`
- `cargo test --test contract canon_reasoning_posture_contract`

### Quality gates

- `cargo test --no-run --all-targets`
- `cargo fmt --all -- --check`
- `sh scripts/check-rust-no-panic.sh`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- `cd /Users/rt/workspace/apply-the/canon && cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cd /Users/rt/workspace/apply-the/canon && cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

### Coverage notes

The workspace-root `cargo llvm-cov` summary reports only root-owned files in this repository layout. On the current tree it reports:

- root summary: `161/161` regions and `103/103` lines covered (`100.00%`)

To capture the member crates touched by this feature, targeted crate summaries were also run:

- `cargo llvm-cov -p boundline-adapters --lib --summary-only`
  - total: `84.62%` regions, `85.75%` lines
  - `orchestrator/session_runtime.rs`: `82.89%` regions, `84.86%` lines
- `cargo llvm-cov -p boundline-cli --lib --summary-only`
  - total: `88.42%` regions, `90.21%` lines
  - `src/cli/output.rs`: `87.71%` regions, `90.40%` lines
  - `src/cli/inspect.rs`: `94.55%` regions, `95.87%` lines
  - `src/cli/session.rs`: `91.66%` regions, `93.57%` lines

## Remaining Open Work

All tasks in `tasks.md` are now complete.

Uniform per-profile closure was completed in [`062-reasoning-profile-closure`](../062-reasoning-profile-closure/spec.md).
