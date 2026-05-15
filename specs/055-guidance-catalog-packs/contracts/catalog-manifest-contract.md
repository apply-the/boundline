# Contract: Catalog Manifest

## Purpose

Define the required and optional shape for `catalog/catalog-manifest.toml` in a guidance catalog pack.

## Required Sections

```toml
[catalog]
id = "catalog-id"
version = "0.1.0"
kind = "guidance-catalog"
status = "draft"
description = "Catalog description"

[compatibility]
boundline = ">=0.55"

[authority]
default_source = "shared-pack"
default_strength = "recommended"
canon_promotable = true
workspace_override_allowed = true

[layout]
guidance_dir = "guidance"
guardians_dir = "guardians"
schemas_dir = "schemas"
examples_dir = "examples"

[pillars]
included = ["clean-code", "architecture", "testing"]
```

## Optional Sections

```toml
[runtime]
requires_s2_1 = true
requires_s3 = false
requires_s4 = false

[trace]
record_resolution = true
record_authority_source = true
record_guidance_strength = true
record_guardian_findings = true
```

## Validation Rules

- `catalog.id`, `catalog.version`, `catalog.kind`, and `catalog.description` MUST be present and non-empty.
- `catalog.kind` MUST identify a guidance catalog.
- `compatibility.boundline` MUST be present.
- `authority.default_source` MUST use a supported authority source value.
- `authority.default_strength` MUST use a supported guidance strength value.
- `pillars.included` MUST contain only canonical pillar identifiers.
- layout directories MUST be relative paths under the pack root.

## Supported Authority Source Values

- `runtime-evidence`
- `workspace-override`
- `canon-governed`
- `shared-pack`
- `boundline-built-in`