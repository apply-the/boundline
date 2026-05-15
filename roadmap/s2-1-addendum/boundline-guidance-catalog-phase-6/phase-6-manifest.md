# Phase 6 Manifest — Optional Ecosystem Guidance

## Included Guidance

```text
guidance/framework-frontend-modern.md
guidance/framework-rails-laravel.md
guidance/framework-mobile.md
guidance/data-ai-systems.md
guardians/optional-ecosystem-guardian-rule-seeds.md
```

## Pillars Covered

- modern frontend framework conventions
- Angular, Vue, Svelte, Next.js-style boundary concerns
- Rails and Laravel conventions
- mobile architecture and release risks
- offline and network handling
- data pipeline reliability
- AI/ML system evaluation
- prompt/model boundary safety
- dataset and feature provenance
- drift, monitoring, and reproducibility

## Suggested Pack Mapping

```toml
[guidance.framework_frontend_modern]
path = "guidance/framework-frontend-modern.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "refactor"]

[guidance.framework_rails_laravel]
path = "guidance/framework-rails-laravel.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "refactor"]

[guidance.framework_mobile]
path = "guidance/framework-mobile.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "release"]

[guidance.data_ai_systems]
path = "guidance/data-ai-systems.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "verification"]
```

## Suggested Guardians

```toml
[guardians.frontend_modern]
rules = ["frontend-reactivity-misuse", "frontend-routing-boundary", "frontend-accessibility-regression"]

[guardians.rails_laravel]
rules = ["active-record-domain-leakage", "callback-hidden-workflow", "queue-job-idempotency"]

[guardians.mobile]
rules = ["offline-state-risk", "platform-permission-risk", "release-compatibility-risk"]

[guardians.data_ai]
rules = ["dataset-provenance-missing", "evaluation-leakage", "model-output-unvalidated", "drift-monitoring-missing"]
```

## Authority Strength

Default strength: recommendation.

These are optional ecosystem capabilities and should not be treated as required unless activated by:

- workspace override
- Canon-governed standard
- organization pack
- repository architecture decision
- explicit expert pack configuration
