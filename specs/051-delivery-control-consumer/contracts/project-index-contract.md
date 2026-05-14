# Project Index Contract

## Purpose

Define the minimal repo-visible shape for `project.boundline.toml`.

## Shape

```toml
[project]
name = "example"
primary_domains = ["commerce"]

[docs]
project_memory = "docs/project"
evidence = "docs/evidence"

[systems.checkout]
workspace = "web-app"
paths = ["apps/checkout", "packages/payment"]
owner = "checkout-team"
domain = "commerce"
criticality = "high"
```

## Workflow Delivery Path Entry

V1 delivery paths live inside `.boundline/workflows.toml` using a dedicated
section that extends the existing workflow registry rather than replacing it.

```toml
[delivery_paths.idea_to_code]
description = "Move from idea intake to verified code through bounded stages."
stages = [
  "discovery",
  "requirements",
  "domain-language",
  "domain-model",
  "system-shaping",
  "architecture",
  "backlog",
  "implementation",
  "verification",
  "pr-review"
]
adaptive = true
```

## Delivery Path Semantics

- `stages` is an ordered list of registered delivery stage identifiers.
- Supported V1 stage identifiers are `discovery`, `requirements`,
  `domain-language`, `domain-model`, `system-shaping`, `architecture`,
  `backlog`, `implementation`, `verification`, `pr-review`,
  `system-assessment`, `change`, `migration`, `security-assessment`,
  `incident`, `supply-chain-analysis`, and `refactor`.
- Unknown stage identifiers are `unsupported stage or mode` hard stops in V1.
- Existing workflow entries continue to own execution detail; `delivery_paths`
  is only the higher-level stage map.

## Semantics

- `project.boundline.toml` owns project semantics such as systems, domains,
  owners, paths, docs locations, and criticality.
- `.boundline/cluster.toml` owns workspace topology and membership.
- The project index may reference workspace IDs from the cluster, but it does
  not replace cluster topology.
- `docs.project_memory` and `docs.evidence` override the V1 defaults of
  `docs/project/` and `docs/evidence/` when present; when absent, Boundline
  falls back to those defaults.