# Contract: Authored Input Shortcuts

## Scope

The same authored-input normalization path used by session-native capture/run and explicit compatibility entry points.

## Inputs

- Inline text such as `Fix the failing parser`
- One Markdown path such as `./docs/prd.md`
- An ordered Markdown-path array such as `[./docs/prd.md, ./docs/adr.md]`

## Required Behavior

- Pure inline text remains `primary_goal_text` and produces a `direct_text` source.
- A single Markdown path produces no `primary_goal_text` and yields one `referenced_markdown` source.
- An ordered Markdown-path array produces no `primary_goal_text` and yields ordered `referenced_markdown` sources after deterministic deduplication.
- Mixed text plus Markdown references keeps the text as `primary_goal_text` and records the referenced sources separately.
- Invalid, missing, unsupported, or out-of-workspace paths fail explicitly during normalization.

## Persisted Evidence

- `authored_input_summary`
- `authored_input_sources`
- `authored_input_deduplicated_sources` when applicable
- `primary_goal_text` only when bounded direct text remains after normalization