# Guidance Entry Schema

## Purpose

This document defines the expected fields for guidance catalog entries.

## Required Fields

```toml
[guidance.<id>]
path = "guidance/file.md"
pillar = "string"
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
replaced_by = "guidance.new_id"
```

## Field Definitions

### path

Relative path to the guidance markdown.

### pillar

High-level classification.

Examples:

```text
clean-code
architecture
testing
language
framework
security
domain
operations
supply-chain
data-ai
```

### strength

One of:

```text
mandatory
recommended
legacy-warning
target-excellence
anti-pattern
deprecated
```

### applies_to

Lifecycle phases where this guidance may be active.

### roles

Runtime roles likely to consume this guidance.

## Validation Rules

- `path` must exist in the package unless entry is external.
- `strength` must use supported values.
- `applies_to` must use supported lifecycle labels.
- deprecated entries should declare `replaced_by` when possible.
