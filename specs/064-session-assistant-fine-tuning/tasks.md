# Tasks: Provider Auth, Probe Readiness, and Assistant Handoff Fine-Tuning

**Input**: Design documents from `/specs/064-session-assistant-fine-tuning/`  
**Prerequisites**: spec.md (required), plan.md (required)

**Tests**: This retrospective slice relies on focused auth, probe, host-output, planning-gate, and assistant-contract validation rather than one monolithic regression command.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel
- **[Story]**: Related story (`US1`, `US2`, `US3`, `US4`)

## Phase 1: Provider Auth Foundation

- [x] T001 [US1] Add the versioned auth profile domain model in `src/domain/auth_profile.rs`.
- [x] T002 [US1] Add global auth profile persistence in `src/adapters/auth_profile_store.rs`.
- [x] T003 [US1] Add GitHub Copilot device-flow auth support in `src/adapters/github_device_flow.rs`.
- [x] T004 [US1] Implement `models auth login`, `status`, and `remove` in `src/cli/models_auth.rs` and wire the subcommands through `src/cli.rs`.
- [x] T005 [US1] Integrate stored auth-profile lookup into the touched provider-runtime adapters in `src/adapters/provider_runtime.rs` and `src/adapters/provider_runtime/copilot.rs`.
- [x] T006 [US1] Add or refresh auth-profile and runtime coverage in `src/domain/auth_profile.rs`, `tests/unit/session_cli_runtime.rs`, and related touched runtime test surfaces.

## Phase 2: Planning Gates and Assistant-Safe Output Alignment

- [x] T007 [US2] Surface goal-quality, plan-quality, backlog-quality, and planning-analysis projections in `src/domain/goal_plan.rs`, `src/domain/governance.rs`, `src/cli/output_session_status.rs`, and related session output surfaces.
- [x] T008 [US2] Preserve assistant-safe `phase_request`, `assistant_resume_command`, and `assistant_next_command` behavior in `src/cli/orchestrate.rs`, `src/cli/session.rs`, and `src/cli.rs`.
- [x] T009 [US2] Update prompt and template guidance for goal-quality, plan-quality, backlog-quality, and planning-analysis gates in `assistant/prompts/goal-template.md`, `assistant/copilot/prompts/`, and `assistant/*/commands/`.
- [x] T010 [US2] Add or refresh structured host-output coverage in `tests/contract/host_command_output_contract.rs`.
- [x] T011 [US2] Add or refresh planning-gate precedence coverage in `tests/contract/planning_gate_pipeline_contract.rs`.

## Phase 3: Probe Preflight Surface

- [x] T012 [US3] Add the typed probe report models in `src/domain/probe.rs` and register them in `src/domain.rs` and `crates/boundline-core/src/domain.rs`.
- [x] T013 [US3] Implement the read-only probe execution logic in `src/cli/probe.rs`.
- [x] T014 [US3] Wire `boundline probe` into the CLI, host output naming, and workspace resolution flow in `src/cli.rs`, `src/cli/output_host.rs`, and `crates/boundline-cli/src/cli.rs`.
- [x] T015 [US3] Document probe in `README.md` and `CHANGELOG.md` as an assistant helper surface rather than a repo-local command.
- [x] T016 [US3] Add focused contract coverage for probe bootstrap, doctor, goal-ready, and host-envelope behavior in `tests/contract/probe_command_contract.rs`.

## Phase 4: Assistant Handoff and Prompt Contract Closure

- [x] T017 [US4] Update readiness-sensitive goal, plan, status, and recover assets across Copilot, Claude, Codex, and Antigravity to use probe preflight and bootstrap-safe routing.
- [x] T018 [P] [US4] Close Copilot action-prompt routing and command-link gaps in `assistant/copilot/prompts/boundline-goal.prompt.md`, `boundline-plan.prompt.md`, `boundline-step.prompt.md`, `boundline-run.prompt.md`, `boundline-status.prompt.md`, `boundline-next.prompt.md`, `boundline-recover.prompt.md`, and `boundline-inspect.prompt.md`.
- [x] T019 [P] [US4] Close remaining Copilot follow-up prompt section gaps in `assistant/copilot/prompts/boundline-assumptions.prompt.md`, `boundline-challenge.prompt.md`, `boundline-evidence.prompt.md`, `boundline-explain-plan.prompt.md`, `boundline-hidden-impact.prompt.md`, `boundline-next-best.prompt.md`, `boundline-risk.prompt.md`, `boundline-why.prompt.md`, plus touched support prompts such as `boundline-doctor.prompt.md`, `boundline-govern.prompt.md`, and `boundline-update.prompt.md`.
- [x] T020 [US4] Extend assistant pack and definition contract coverage in `tests/contract/assistant_command_pack_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, and `tests/contract.rs`.
- [x] T021 [US4] Align `assistant/README.md` with probe bootstrap behavior, host-native action syntax, and prompt-handoff expectations.

## Phase 5: Validation and Retrospective Closeout

- [x] T022 [US1] Run representative auth and structured-host-output validation across the touched runtime and contract test surfaces.
- [x] T023 [US3] Run focused probe validation with `cargo test -p boundline-cli --lib probe` and `cargo test -p boundline --test contract probe_command_contract`.
- [x] T024 [US4] Run the broader assistant contract modules with `cargo test -p boundline --test contract assistant_command_pack_contract::` and `cargo test -p boundline --test contract assistant_command_definition_contract::`.
- [x] T025 [US1] Run workspace lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] T026 [US4] Update retrospective documentation in `spec.md`, `plan.md`, and `tasks.md` so spec 064 reflects commits `cad1675`, `9ba0b21`, and `6182711`.

## Dependencies & Execution Order

- Provider-auth foundation tasks (`T001-T006`) establish the new persisted credential path before prompt and probe guidance can rely on it.
- Planning-gate and host-output alignment (`T007-T011`) provide the runtime semantics that prompt handoff guidance needs to preserve.
- Probe tasks (`T012-T016`) should land before readiness-sensitive prompt assets start depending on probe outputs.
- Assistant contract-closure tasks (`T017-T021`) depend on the runtime and probe behavior already being stable enough to document and validate.
- Validation and retrospective closeout (`T022-T026`) remain required after all implementation and prompt updates are in place.

## Notes

- This task list documents already-implemented work for release and branch traceability.
- The earlier 064 draft around session-reference and audit-only fine-tuning is superseded here by the actual scope of the last three landed commits.
- Assistant routing remains bounded by runtime-owned handoffs; prompt assets describe those decisions but do not become a second execution authority.
