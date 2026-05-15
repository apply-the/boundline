# Quickstart: Guidance Catalog And Guardian Rule Packs

## 1. Create A Local Catalog Pack

Start from a workspace that already uses the normal Boundline session-native flow.
Create a local pack root:

```bash
mkdir -p .boundline/packs/example-guidance-pack/catalog
mkdir -p .boundline/packs/example-guidance-pack/guidance
mkdir -p .boundline/packs/example-guidance-pack/guardians
```

Expected result:
- the workspace contains a pack root with the required catalog directories.

## 2. Add Pack And Catalog Manifests

Create `pack.toml` and `catalog/catalog-manifest.toml`:

```bash
cat > .boundline/packs/example-guidance-pack/pack.toml <<'EOF'
[pack]
id = "example-guidance-pack"
version = "0.1.0"
kind = "guidance-pack"
description = "Example pack for local catalog validation"
EOF

cat > .boundline/packs/example-guidance-pack/catalog/catalog-manifest.toml <<'EOF'
[catalog]
id = "example-guidance-catalog"
version = "0.1.0"
kind = "guidance-catalog"
status = "draft"
description = "Example catalog"

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
included = ["clean-code"]
EOF
```

Expected result:
- the pack declares identity and the catalog declares the minimum required manifest sections.

## 3. Add Guidance And Guardian Indexes

Create minimal index files:

```bash
cat > .boundline/packs/example-guidance-pack/catalog/guidance-index.toml <<'EOF'
[guidance.clean_code]
path = "guidance/clean-code.md"
pillar = "clean-code"
strength = "recommended"
applies_to = ["implementation", "review"]
roles = ["implementer", "reviewer"]
EOF

cat > .boundline/packs/example-guidance-pack/catalog/guardian-index.toml <<'EOF'
[guardian.clean_code]
pillar = "clean-code"
kind = "llm"
rules = ["intent-revealing-names"]
applies_to = ["implementation", "review"]
default_disposition = "concern"
EOF
```

Expected result:
- the pack now contains one guidance entry and one guardian rule seed.

## 4. Add Referenced Content Files

Create the guidance Markdown referenced by the index:

```bash
printf '%s
' '# Clean Code' 'Prefer names that reveal intent and keep responsibilities narrow.' > .boundline/packs/example-guidance-pack/guidance/clean-code.md
```

Expected result:
- every referenced file exists on disk and can be validated.

## 5. Run Plan-Time Resolution

Run:

```bash
boundline plan
```

Expected result:
- Boundline discovers the local catalog pack.
- the runtime validates pack and catalog metadata before resolution.
- the plan persists loaded packs, selected entries, skipped entries, and any validation findings.

## 6. Inspect The Catalog Resolution Story

Run:

```bash
boundline status
boundline inspect
```

Expected result:
- `status` surfaces loaded packs and a high-level validation summary.
- `inspect` shows why a pack loaded, skipped, or lost precedence, plus the authority source attached to selected entries.

## 7. Verify Validation Behavior

Break the guidance path and re-run:

```bash
rm .boundline/packs/example-guidance-pack/guidance/clean-code.md
boundline plan
```

Expected result:
- the runtime records an explicit load warning for the missing guidance file.
- other valid entries continue to load.
- no catalog failure is hidden or silently ignored.