# Research: Catalog Currency, Independent Voting, and File-Backed Inputs

## Decision 1: Keep the bundled catalog curated and offline at runtime

- **Decision**: Continue shipping a repository-managed TOML catalog and refresh it from public provider docs during feature delivery rather than introducing live provider discovery into `init` or config flows.
- **Rationale**: The user asked for all known mainstream route-capable models, but Boundline still needs deterministic offline bootstrap behavior. A curated bundle preserves predictable `init` and config output while letting the repo update quickly when providers publish route-relevant models.
- **Alternatives considered**:
  - Query provider APIs or websites during `init`: rejected because it would add network dependency, format drift risk, and startup latency to the bounded CLI path.
  - Leave the catalog intentionally stale and rely on custom model ids: rejected because it makes the primary route-selection surface untrustworthy.
- **Applied delta**: Added missing mainstream route-capable entries such as `gpt-5.5`, `gpt-5.4-nano`, `sonnet-4.6`, `opus-4.7`, `haiku-4.5`, `gemini-3.1-pro-preview`, `gemini-2.5-flash-lite`, `gemini-3-flash-preview`, `gemini-3.1-flash-lite`, and `gemini-3.1-flash-lite-preview`; also aligned built-in default routes for verification and review.

## Decision 2: Interpret path-only goal text as file-backed authored input in the normalizer

- **Decision**: Detect when the supplied direct goal text is only one Markdown path or an ordered array of Markdown paths, and treat that value as referenced Markdown input instead of persisting it as `primary_goal_text`.
- **Rationale**: Core authored-input ingestion already supports referenced Markdown paths. The bug was local: a value like `./docs/prd.md` or `[./docs/prd.md, ./docs/adr.md]` was both parsed as references and retained as raw direct text. Fixing the normalizer removes duplicate noisy goal text without inventing a new CLI flag or persistence surface.
- **Alternatives considered**:
  - Add a new dedicated CLI flag just for path arrays: rejected because repeated `--brief` flags already exist and the bug is specifically about prompt-like direct input values.
  - Parse the shorthand only in assistant command packs: rejected because it would leave native CLI callers inconsistent with assistant-driven flows.

## Decision 3: Resolve review independence from task-state routing projection first, then fall back to explicit reviewer source

- **Decision**: Carry `routing_projection` into the task's initial context state, derive each reviewer's effective route from reviewer-specific entries or the shared review route, persist that route on review participants, and reject vote resolution when completed reviewers collapse onto the same effective route or cannot resolve one.
- **Rationale**: The user concern was not reviewer labels, but the effective runtime/model behind each counted vote. The existing routing projection already describes the resolved review routes, so using that state keeps the fix aligned with actual routing behavior. Falling back to `reviewer.source` preserves local fixture behavior where explicit source strings already exist.
- **Alternatives considered**:
  - Treat reviewer ids or roles as proof of independence: rejected because different labels can still map to the same effective route.
  - Allow duplicate routes and only annotate them in output: rejected because the spec requires Boundline to block, degrade, or escalate explicitly instead of presenting a misleading multi-vote result.

## Decision 4: Expose effective routes through existing review evidence instead of inventing a new reporting surface

- **Decision**: Add `effective_route` to `ReviewerParticipation` and store it in `latest_review_participants` and `latest_review_vote_resolution`.
- **Rationale**: These fields are already persisted and inspectable. Extending them satisfies the spec requirement to expose the effective route behind each counted reviewer without adding a second review-summary model.
- **Alternatives considered**:
  - Create a separate review-route evidence record: rejected because it duplicates existing persisted review state.