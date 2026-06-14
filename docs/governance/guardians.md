# Guidance And Guardians

Guidance and guardians are how Boundline makes quality expectations explicit during delivery.

## Guidance

Guidance shapes work before and during action. It can cover:

- clean-code expectations
- architecture boundaries
- testing strategy
- language and framework practices
- API contract stability
- migration safety
- security boundaries
- observability and operations readiness
- domain standards

Guidance helps answer:

```text
Which rules should shape this implementation before code changes start?
```

## Guardians

Guardians validate work after action or at quality boundaries. They emit structured findings and can block continuation when risk demands it.

Guardian kinds:

- **rule-based**: checks derived from files, manifests, schemas, or command output
- **model-assisted**: review where judgment is needed and the runtime projects the result as a finding
- **hybrid**: deterministic evidence plus model-assisted assessment

Guardians help answer:

```text
What must be checked before this bounded run can be treated as credible?
```

## Structured Findings

Findings should preserve:

- source
- rule or guardian id
- severity or disposition
- affected scope
- evidence
- blocking outcome
- recommended next action when available

Inspect them with:

```bash
boundline status --workspace . --json
boundline inspect --workspace . --json
```

## Source Precedence

Boundline can resolve guidance and guardian sources from:

1. workspace overrides such as `.boundline/guidance/` and `.boundline/guardians/`
2. Canon-governed inputs when configured and compatible
3. bundled assistant packs under `assistant/packs/`
4. built-in fallback capability content

The runtime should project loaded sources, skipped sources, and catalog validation findings so operators can see why a rule participated or did not.

## Workspace Overrides

Use workspace overrides when a repository has local standards that should shape delivery:

```text
.boundline/guidance/
.boundline/guardians/
```

Examples:

- require repository-specific test commands
- define local architecture boundaries
- add migration safety rules
- align security checks with the local threat model

## Shared Expert Packs

Bundled packs provide reusable guidance by language, stack, and delivery role. They keep common expectations available even when a workspace has no custom rules.

See [Core Concepts](/guide/core-concepts#expert-packs).

## Canon-Governed Standards

When Canon is active, governed standards can outrank local or bundled defaults because Canon owns governed knowledge, approval, evidence, and lineage.

Boundline consumes those standards as delivery inputs. It does not become Canon, and Canon does not become the runtime orchestrator.

## Example

For a security-sensitive migration:

```bash
boundline goal --workspace . \
  --goal "Migrate account token storage to a hashed representation" \
  --brief docs/security/migration-brief.md
boundline plan --workspace .
boundline inspect --workspace .
```

Expect the plan and inspect output to show relevant context, guidance sources, validation strategy, and guardian findings or stop conditions.

## Source Reference

See [Extending the Guidance Catalog](https://github.com/apply-the/boundline/blob/0.78.0/tech-docs/guides/extending-guidance-catalog.md) for pack authoring details.
