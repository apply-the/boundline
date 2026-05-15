# Extending the Guidance Catalog

How to add new languages, guardians, and delivery packs to the guidance catalog.

## Directory Layout

```
assistant/packs/
├── guidance-catalog/          # The reference catalog pack
│   ├── pack.toml              # Pack manifest (id, version, compatibility)
│   ├── catalog/
│   │   ├── guidance-index.toml   # All guidance entries
│   │   └── guardian-index.toml   # All guardian entries
│   ├── guidance/              # Guidance Markdown files
│   └── guardians/             # Guardian Markdown files
├── rust-delivery.toml         # Language-specific delivery binding
├── dart-delivery.toml
└── ...
```

## Adding a New Language

### 1. Create the guidance file

Create `assistant/packs/guidance-catalog/guidance/language-<name>.md` with sections:

- Title and one-line purpose
- Key conventions (error handling, typing, concurrency)
- Domain modeling patterns
- Recommended Ecosystem Libraries (table: Category, Library, Purpose)
- Anti-Patterns
- Guardian Hooks (which guardians apply)

Use existing files like `language-rust.md` or `language-dart.md` as templates.

### 2. Register in guidance-index.toml

Add an entry to `assistant/packs/guidance-catalog/catalog/guidance-index.toml`:

```toml
[guidance.language_<name>]
path = "guidance/language-<name>.md"
pillar = "language"
language = "<name>"
strength = "recommended"
applies_to = ["implementation", "testing", "review", "refactor"]
roles = ["implementer", "reviewer"]
```

Valid `applies_to` values: `planning`, `system-shaping`, `architecture`, `backlog`, `implementation`, `testing`, `verification`, `review`, `refactor`, `migration`, `incident`, `supply-chain-analysis`.

Valid `pillar` values: `clean-code`, `architecture`, `testing`, `language`, `framework`, `security`, `domain-language`, `domain-modeling`, `api-contracts`, `migration`, `observability`, `resilience`, `operations-readiness`, `supply-chain`, `data-ai`, `optional-ecosystem`.

Valid `strength` values: `recommended`, `suggested`, `optional`.

### 3. Create a delivery TOML

Create `assistant/packs/<name>-delivery.toml` that binds guidance for that language stack:

```toml
[guidance.language-best-practices]
title = "<Name> Language Guidance"
applies_to = ["planning", "implementation", "verification", "review"]
roles = ["planner", "implementer", "verifier", "reviewer"]
path = "../guidance-catalog/guidance/language-<name>.md"
priority = "high"

[guidance.testing-best-practices]
title = "Testing Guidance"
applies_to = ["planning", "testing", "verification", "review"]
roles = ["planner", "verifier", "reviewer"]
path = "../guidance-catalog/guidance/testing-<relevant>.md"
priority = "high"
```

Valid `priority` values: `low`, `medium`, `high`.

## Adding a New Guardian

### 1. Create the guardian file

Create `assistant/packs/guidance-catalog/guardians/<guardian-id>.md` with sections:

- Title and one-line purpose
- **Rules**: each rule with a slug, description, and triggers
- **Disposition**: default severity
- **Scope**: which languages or contexts apply

### 2. Register in guardian-index.toml

Add an entry to `assistant/packs/guidance-catalog/catalog/guardian-index.toml`:

```toml
[guardian.<guardian_id>]
pillar = "<pillar>"
kind = "<kind>"
rules = ["rule-slug-1", "rule-slug-2", "rule-slug-3"]
applies_to = ["implementation", "review"]
default_disposition = "<disposition>"
```

For language-specific guardians, add `language = "<name>"`.

Valid `kind` values: `deterministic`, `hybrid`, `llm`.

Valid `default_disposition` values: `info`, `observation`, `concern`, `warning`, `risk`, `blocker`, `error`.

## Adding Rules to an Existing Guardian

1. Add the rule description to the guardian's Markdown file under `## Rules`
2. Add the rule slug to the `rules` array in `guardian-index.toml`

## Extending an Existing Delivery Pack

Edit the relevant `<name>-delivery.toml` to add new `[guidance.<section>]` entries pointing to additional guidance files.

## Validation

After any change, run:

```sh
cargo test --test contract guidance_index
cargo test --test contract guardian_index
```

These contract tests verify:
- All paths in guidance-index.toml resolve to existing Markdown files
- All pillar, lifecycle, kind, and disposition values are valid enum members
- No duplicate entry IDs exist
