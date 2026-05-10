# Data Model: Catalog Currency, Independent Voting, and File-Backed Inputs

## Bundled Catalog Entry

- **Location**: `assistant/catalog/model-catalog.toml`
- **Role**: Curated route-capable model option for one runtime.
- **Key fields**:
  - `runtime`: provider/runtime owner such as `copilot`, `claude`, `codex`, or `gemini`
  - `model`: stable model identifier stored in config
  - `label`: operator-visible bundled choice
  - `channel`: release posture such as stable or preview
- **Invariants**:
  - Only route-capable general-purpose models belong in the standard catalog.
  - Catalog metadata must carry a refreshed version/date when the bundle changes.

## Authored Brief Bundle

- **Location**: `src/domain/brief.rs`
- **Role**: Normalized human input used to seed planning and execution.
- **Key fields affected by this feature**:
  - `primary_goal_text: Option<String>`
  - `sources: Vec<InputSourceReference>`
  - `deduplicated_sources: Vec<String>`
- **New behavior**:
  - When the supplied direct input is only a Markdown path or an ordered array of Markdown paths, `primary_goal_text` becomes `None` and the paths are recorded only as referenced Markdown sources.
  - Mixed inline text plus referenced paths still keeps the text in `primary_goal_text` and records the referenced sources separately.
- **Invariants**:
  - Accepted Markdown sources must remain within the active workspace.
  - Source order must follow declared input order after deterministic deduplication.

## Input Source Reference

- **Location**: `src/domain/brief.rs`
- **Role**: Provenance record for one normalized authored input source.
- **Relevant fields**:
  - `kind`: `direct_text`, `attached_markdown`, or `referenced_markdown`
  - `workspace_path: Option<String>`
  - `precedence: usize`
  - `content: String`
- **Invariants**:
  - A path-only prompt shorthand must produce `referenced_markdown` records, not a redundant `direct_text` record.

## Reviewer Participation

- **Location**: `src/domain/review.rs`
- **Role**: Persisted evidence for one reviewer's participation in a resolved vote.
- **Relevant fields**:
  - `reviewer_id: String`
  - `status: ReviewerParticipationStatus`
  - `reason: Option<String>`
  - `effective_route: Option<String>`
- **New behavior**:
  - Every completed participant must resolve to an `effective_route` before the vote can count.
  - Collapsed councils are represented by explicit terminal failure rather than a normal vote decision.

## Routing Projection in Task State

- **Location**: `src/fixture.rs` task initial context and existing routing projection model in `src/domain/routing_decision.rs`
- **Role**: Shared resolved routing evidence used by downstream review logic.
- **Relevant fields**:
  - `effective_routing: Vec<String>` entries such as `review=claude/sonnet-4.6 [workspace]` or `reviewer:safety=copilot/gpt-5.5 [workspace]`
- **New behavior**:
  - The projection is copied into task state at task creation so vote resolution can inspect the effective review routes.

## Vote Resolution Failure States

- **Location**: `src/domain/review.rs` and `src/fixture.rs`
- **Role**: Explicit bounded failure reasons for invalid councils.
- **New states introduced in behavior**:
  - missing effective route for a completed reviewer
  - duplicate effective route across completed reviewers
- **Resulting terminal code**:
  - `non_independent_review_council`