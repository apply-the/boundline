# Tasks: Catalog Currency, Independent Voting, and File-Backed Inputs

**Input**: Design documents from `/specs/047-catalog-voting-inputs/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: This slice changes executable behavior in authored-input normalization, review vote resolution, routing evidence persistence, and bundled routing defaults. Focused unit and fixture coverage is required, and modified Rust files should stay at or above 95% line coverage.

## Phase 1: Setup

- [x] T001 Bump Boundline release version to `0.48.0` in `/Users/rt/workspace/apply-the/boundline/Cargo.toml`, release metadata, and `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`
- [x] T002 Record the catalog refresh outcome and aligned defaults in `/Users/rt/workspace/apply-the/boundline/assistant/catalog/model-catalog.toml` and `/Users/rt/workspace/apply-the/boundline/src/domain/configuration.rs`

## Phase 2: Foundational

- [x] T003 Persist routing projection into initial task context in `/Users/rt/workspace/apply-the/boundline/src/fixture.rs`
- [x] T004 Extend review participation evidence with `effective_route` in `/Users/rt/workspace/apply-the/boundline/src/domain/review.rs`

## Phase 3: User Story 1 - Choose From A Current Catalog (P1)

**Goal**: Surface current mainstream bundled route-capable models and aligned defaults.

- [x] T005 [P] [US1] Refresh bundled model entries and metadata in `/Users/rt/workspace/apply-the/boundline/assistant/catalog/model-catalog.toml`
- [x] T006 [P] [US1] Align built-in default routes and init expectations in `/Users/rt/workspace/apply-the/boundline/src/domain/configuration.rs` and `/Users/rt/workspace/apply-the/boundline/src/cli/init.rs`

## Phase 4: User Story 2 - Start From File-Backed Authored Input (P2)

**Goal**: Treat one Markdown path or an ordered Markdown-path array as file-backed authored input instead of literal goal text.

- [x] T007 [P] [US2] Add failing and then passing unit coverage for path-only and array-based shorthand input in `/Users/rt/workspace/apply-the/boundline/src/domain/brief.rs`
- [x] T008 [US2] Normalize pure Markdown-path shorthand into referenced Markdown sources in `/Users/rt/workspace/apply-the/boundline/src/domain/brief.rs`

## Phase 5: User Story 3 - Keep Review Voting Independent (P3)

**Goal**: Reject review councils that collapse onto the same effective runtime/model route.

- [x] T009 [P] [US3] Add core vote-resolution coverage for effective-route persistence and duplicate-route rejection in `/Users/rt/workspace/apply-the/boundline/src/domain/review.rs`
- [x] T010 [P] [US3] Add fixture-runtime coverage for routing-projection propagation and explicit collapsed-council failure in `/Users/rt/workspace/apply-the/boundline/src/fixture.rs`
- [x] T011 [US3] Resolve reviewer effective routes from task-state routing projection and reject non-independent councils in `/Users/rt/workspace/apply-the/boundline/src/fixture.rs` and `/Users/rt/workspace/apply-the/boundline/src/domain/review.rs`

## Phase 6: Polish

- [x] T012 [P] Refresh feature artifacts and assistant/runtime guidance in `/Users/rt/workspace/apply-the/boundline/specs/047-catalog-voting-inputs/`, `/Users/rt/workspace/apply-the/boundline/assistant/README.md`, and related release notes as needed
- [x] T013 Run focused validation plus `cargo test --no-run --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --check`

## Dependencies & Order

- T001-T004 establish release alignment and shared evidence.
- US1 is independent once setup is complete.
- US2 depends only on the existing authored-input normalizer and can ship independently after T007-T008.
- US3 depends on T003-T004 because vote resolution needs persisted routing evidence and per-participant effective routes.
- Polish runs after the selected stories are complete.