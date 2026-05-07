# Contract: Workspace Hygiene Defaults

## Target Files

- `.gitignore` when the workspace is a Git repository
- `.dockerignore` when Docker cues are present
- `.eslintignore` when legacy ESLint ignore files are relevant
- `.prettierignore` when Prettier cues are present
- `.terraformignore` when Terraform cues are present
- `.helmignore` when Helm cues are present

## Selection Rules

- Universal patterns are always eligible when the target file exists or is credibly needed.
- Technology-specific patterns are eligible only when selected domain families or repository evidence support them.
- Tool-specific patterns are eligible only when repository files or declared domains justify that tool.

## Merge Rules

- Existing file contents are preserved.
- Missing critical patterns are appended.
- Duplicate patterns are not re-added.
- Existing non-empty custom lines remain unchanged.

## Blocking And Skip Rules

- If domain evidence is weak or contradictory, Boundline applies only universal defaults or reports why technology-specific defaults were skipped.
- If the target file is not relevant to the repository, Boundline does not create it.
- If the workspace is read-only, hygiene updates fail with an explicit write error and do not continue silently.
