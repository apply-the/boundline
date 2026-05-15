# Contract: Guidance Index

## Purpose

Define the required and optional shape for `catalog/guidance-index.toml` entries.

## Required Fields

```toml
[guidance.clean_code]
path = "guidance/clean-code.md"
pillar = "clean-code"
strength = "recommended"
applies_to = ["implementation", "review"]
roles = ["implementer", "reviewer"]
```

## Optional Fields

```toml
language = "rust"
framework = "react"
authority_source = "shared-pack"
canon_artifact_kind = "architecture"
owner = "team-or-pack-owner"
version = "1.0.0"
deprecated = false
replaced_by = "guidance.clean_code_v2"
```

## Validation Rules

- entry keys MUST be unique inside the file.
- `path`, `pillar`, `strength`, `applies_to`, and `roles` MUST be present.
- `path` MUST resolve to a Markdown file inside the pack root.
- `pillar` MUST be one of the canonical pillar identifiers.
- `strength` MUST be one of:
  - `mandatory`
  - `recommended`
  - `legacy-warning`
  - `target-excellence`
  - `anti-pattern`
  - `deprecated`
- `applies_to` MUST use supported lifecycle labels.
- `roles` MUST not be empty.

## Canonical Pillar Identifiers

- `clean-code`
- `architecture`
- `testing`
- `language`
- `framework`
- `security`
- `domain-language`
- `domain-modeling`
- `api-contracts`
- `migration`
- `observability`
- `resilience`
- `operations-readiness`
- `supply-chain`
- `data-ai`
- `optional-ecosystem`