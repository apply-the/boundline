# Guardian Entry Schema

## Purpose

This document defines the expected fields for guardian catalog entries.

## Required Fields

```toml
[guardian.<id>]
pillar = "string"
kind = "hybrid"
rules = ["rule-one", "rule-two"]
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

## Guardian Kinds

Supported kinds:

```text
deterministic
llm
hybrid
```

## Dispositions

Supported default dispositions:

```text
info
observation
concern
warning
risk
blocker
error
```

## Validation Rules

- `kind` must be supported by Boundline runtime.
- `rules` must not be empty.
- `applies_to` must use supported lifecycle labels.
- deterministic guardians should declare tools or commands.
- llm guardians should declare guidance or instructions.
- hybrid guardians should declare both deterministic evidence and semantic evaluation expectations where possible.
