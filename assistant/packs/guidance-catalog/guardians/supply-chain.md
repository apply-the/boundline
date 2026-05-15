# Supply Chain Guardian

Review dependency and lockfile changes together with vulnerability and license signals before finalizing the delivery.

## Rules

### dependency-manifest-without-lockfile-update
Changing a dependency manifest (Cargo.toml, package.json, pyproject.toml, go.mod) without updating the corresponding lockfile creates reproducibility risk. Both must change together.

Triggers: manifest changes without lockfile in the same commit, lockfile conflicts resolved by regeneration without review.

### license-unknown
Every dependency must have a known, acceptable license. Unknown or unreviewed licenses create legal risk.

Triggers: new dependencies without license verification, transitive dependencies with unclear licensing, license changes in dependency updates.

### vulnerability-untriaged
Known vulnerabilities in dependencies must be triaged: mitigated, accepted with justification, or resolved by update. Unacknowledged vulnerabilities are not acceptable.

Triggers: `cargo deny`, `npm audit`, or equivalent reporting findings without corresponding triage documentation, dependency updates that introduce new advisories.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages with package managers. Most relevant during dependency changes and supply-chain analysis.
