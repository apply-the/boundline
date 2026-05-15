# Packaging Conventions

## Purpose

This document defines conventions for packaging guidance catalogs and guardian rule packs.

## Package Shape

Recommended shape:

```text
boundline-guidance-pack/
  pack.toml
  catalog/
    catalog-manifest.toml
    guidance-index.toml
    guardian-index.toml
  guidance/
    clean-code.md
    architecture.md
    testing-core.md
    language-rust.md
  guardians/
    guardian-rule-seeds.md
  schemas/
    guidance-entry.schema.md
    guardian-entry.schema.md
  examples/
    resolution-trace.example.json
```

## File Naming

Guidance files:

```text
guidance/<pillar>.md
guidance/language-<language>.md
guidance/framework-<framework-or-family>.md
```

Guardian files:

```text
guardians/<pillar>-guardian-rule-seeds.md
guardians/<specific-guardian>.toml
```

## Manifest Requirements

A package manifest must declare:

- pack ID
- version
- compatibility
- authority defaults
- guidance entries
- guardian entries
- lifecycle phases
- optional Canon artifact preferences

## Compatibility

A pack must declare:

```toml
[compatibility]
boundline = ">=0.54"
canon_contract = ">=0.50"
```

## Extension Policy

New guidance can be added without changing S2.1 runtime behavior when it follows the manifest schema.

New guardian kinds require runtime support only if they introduce a new execution type.

Existing kinds:

```text
deterministic
llm
hybrid
```
