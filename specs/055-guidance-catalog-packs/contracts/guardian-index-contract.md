# Contract: Guardian Index

## Purpose

Define the required and optional shape for `catalog/guardian-index.toml` entries.

## Required Fields

```toml
[guardian.clean_code]
pillar = "clean-code"
kind = "llm"
rules = ["intent-revealing-names"]
applies_to = ["implementation", "review"]
default_disposition = "concern"
```

## Optional Fields

```toml
language = "rust"
framework = "react"
requires_guidance = ["guidance.clean_code"]
requires_tools = ["clippy"]
timeout_seconds = 60
max_findings = 20
authority_source = "shared-pack"
owner = "team-or-pack-owner"
version = "1.0.0"
```

## Validation Rules

- entry keys MUST be unique inside the file.
- `pillar`, `kind`, `rules`, `applies_to`, and `default_disposition` MUST be present.
- `pillar` MUST use a canonical pillar identifier.
- `kind` MUST be one of `deterministic`, `llm`, or `hybrid`.
- `rules` MUST not be empty.
- `applies_to` MUST use supported lifecycle labels.
- `default_disposition` MUST be one of:
  - `info`
  - `observation`
  - `concern`
  - `warning`
  - `risk`
  - `blocker`
  - `error`

## Lifecycle Labels

- `planning`
- `system-shaping`
- `architecture`
- `backlog`
- `implementation`
- `testing`
- `verification`
- `review`
- `refactor`
- `migration`
- `incident`
- `supply-chain-analysis`