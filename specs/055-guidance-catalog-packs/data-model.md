# Data Model: Guidance Catalog And Guardian Rule Packs

## CatalogManifest

Represents the catalog-level defaults and layout metadata declared in `catalog/catalog-manifest.toml`.

Fields:
- `catalog_id`: stable catalog identifier.
- `version`: catalog version string.
- `kind`: catalog kind, expected to identify a guidance catalog.
- `status`: draft, active, deprecated, or equivalent release state.
- `description`: operator-facing summary.
- `compatibility.boundline`: supported Boundline version range.
- `compatibility.canon_contract`: optional Canon contract version range.
- `authority.default_source`: default authority source for entries that do not override it.
- `authority.default_strength`: default strength for guidance entries that do not override it.
- `authority.canon_promotable`: whether the catalog can be promoted into Canon-governed standards.
- `authority.workspace_override_allowed`: whether workspace-local content may override this catalog.
- `layout`: declared directories for guidance, guardians, schemas, and examples.
- `included_pillars`: ordered set of pillar identifiers included by the catalog.
- `trace_policy`: booleans describing whether resolution, authority, strength, and findings should be recorded.

Validation rules:
- `catalog_id`, `version`, `kind`, and `description` must be non-empty.
- `authority.default_strength` must be one of `mandatory`, `recommended`, `legacy-warning`, `target-excellence`, `anti-pattern`, or `deprecated` when present.
- `included_pillars` must contain only supported pillar identifiers.
- declared layout directories must be relative paths inside the pack root.

## GuidanceEntry

Represents one guidance item declared in `catalog/guidance-index.toml`.

Fields:
- `entry_id`: stable guidance identifier.
- `path`: relative path to the guidance Markdown file.
- `pillar`: canonical pillar classification.
- `language`: optional language classifier.
- `framework`: optional framework classifier.
- `strength`: guidance strength classification.
- `applies_to`: ordered lifecycle phases where the guidance may influence work.
- `roles`: ordered expert or reviewer roles that consume the entry.
- `authority_source`: optional per-entry authority override.
- `canon_artifact_kind`: optional Canon-facing artifact classifier for promotion compatibility.
- `owner`: optional pack or team owner.
- `version`: optional entry version.
- `deprecated`: whether the entry has been superseded.
- `replaced_by`: optional successor entry identifier.

Validation rules:
- `entry_id`, `path`, `pillar`, `strength`, `applies_to`, and `roles` must be present.
- `pillar` must match the canonical pillar taxonomy.
- `strength` must be one of `mandatory`, `recommended`, `legacy-warning`, `target-excellence`, `anti-pattern`, or `deprecated`.
- `path` must resolve to a file within the pack root.

## GuardianRuleSeed

Represents one guardian rule seed declared in `catalog/guardian-index.toml`.

Fields:
- `seed_id`: stable guardian identifier.
- `pillar`: canonical pillar classification.
- `kind`: `deterministic`, `llm`, or `hybrid`.
- `rules`: ordered rule identifiers covered by the seed.
- `applies_to`: ordered lifecycle phases where the seed may run.
- `default_disposition`: default finding disposition.
- `language`: optional language classifier.
- `framework`: optional framework classifier.
- `requires_guidance`: optional guidance entry IDs this seed expects.
- `requires_tools`: optional deterministic tools required by the seed.
- `timeout_seconds`: optional execution budget hint.
- `max_findings`: optional upper bound for emitted findings.
- `authority_source`: optional per-entry authority override.
- `owner`: optional pack or team owner.
- `version`: optional entry version.

Validation rules:
- `seed_id`, `pillar`, `kind`, `rules`, `applies_to`, and `default_disposition` must be present.
- `kind` must be one of `deterministic`, `llm`, or `hybrid`.
- `default_disposition` must be one of `info`, `observation`, `concern`, `warning`, `risk`, `blocker`, or `error`.
- `rules` must not be empty.

## CatalogValidationFinding

Represents one explicit warning or error emitted while loading or validating a catalog pack.

Fields:
- `finding_id`: stable validation finding identifier.
- `severity`: `warning` or `error`.
- `scope`: manifest, guidance-entry, guardian-entry, layout, or compatibility.
- `entry_id`: optional affected guidance or guardian identifier.
- `field`: optional field name associated with the issue.
- `message`: operator-facing explanation.
- `recovery_action`: suggested remediation.

Validation rules:
- `finding_id`, `severity`, `scope`, `message`, and `recovery_action` must be non-empty.
- `severity` must be either `warning` or `error`.

## CatalogResolutionRecord

Represents the ordered outcome of discovering and validating catalog packs for a bounded planning or execution context.

Fields:
- `target_ref`: bounded workspace target or workspace-level scope.
- `phase`: lifecycle phase for which catalog-backed guidance was resolved.
- `loaded_packs`: ordered pack identifiers that contributed winning entries.
- `loaded_guidance_entries`: ordered selected guidance entry IDs.
- `loaded_guardian_seeds`: ordered selected guardian seed IDs.
- `skipped_packs`: ordered pack identifiers that were unavailable, invalid, shadowed, or incompatible.
- `validation_findings`: ordered validation finding IDs emitted during catalog processing.
- `authority_decisions`: ordered lines explaining precedence decisions.
- `summary`: operator-facing headline for the catalog load result.

Validation rules:
- `target_ref`, `phase`, and `summary` must be non-empty.
- every skipped pack must have a matching reason in `authority_decisions` or `validation_findings`.

Relationships:
- `CatalogManifest` owns catalog defaults and layout.
- `GuidanceEntry` and `GuardianRuleSeed` may inherit defaults from `CatalogManifest`.
- `CatalogValidationFinding` can refer to either a `CatalogManifest`, `GuidanceEntry`, or `GuardianRuleSeed` problem.
- `CatalogResolutionRecord` references loaded and skipped packs plus emitted `CatalogValidationFinding` records.
- runtime consumer code must normalize `GuidanceEntry.strength` and `GuardianRuleSeed.default_disposition` into the structured finding and trace model used by S2.1.