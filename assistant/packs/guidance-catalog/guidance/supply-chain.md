# Supply Chain

Dependency updates are code changes: review lockfiles, licensing impact, and known vulnerabilities together with the implementation diff. Prefer reproducible builds and explicit provenance over convenience shortcuts.

## Core Principles

### Dependency Introduction

Adding a dependency is an architecture decision. Before adding:
- Is the functionality needed, or is the implementation trivial?
- Is the library actively maintained?
- What is the license?
- What is the transitive dependency footprint?
- Does it introduce native code or build scripts?
- Is there a known vulnerability history?
- Does it conflict with existing dependencies?

### Version Pinning And Lockfiles

All production dependencies must be pinned through a lockfile. Lock the exact resolved versions, not just the declared ranges. Commit lockfile changes alongside code changes. Review lockfile diffs for unexpected transitive additions.

### Vulnerability Management

Known vulnerabilities must be triaged, not ignored. Establish:
- Scanning cadence (CI-integrated preferred)
- Severity thresholds for blocking merge
- Triage workflow for false positives
- Upgrade path for affected dependencies
- Workaround documentation when upgrade is blocked

### License Risk

Every dependency carries license obligations. Track:
- Copyleft licenses in distributed binaries
- Network-triggered copyleft for server-side use
- License compatibility with project license
- License changes across version upgrades

### Build Scripts And Install Hooks

Dependencies with build scripts or install hooks can execute arbitrary code. Treat with higher scrutiny:
- Native compilation steps
- Post-install scripts
- Code generation during build
- Network access during build

### Generated Code And Provenance

AI-generated code should be treated as an unverified dependency. Track:
- Which tool generated it
- What prompt or context was used
- Whether it was reviewed
- Whether it introduces hidden dependencies

### Container And CI Supply Chain

Container images and CI pipelines carry supply chain risk. Pin:
- Base image digests
- CI action versions
- Tool versions in CI scripts
- Registry sources

### SBOM

Software Bill of Materials should be producible for released artifacts. Maintain enough metadata to reconstruct what went into a build.

## Anti-Patterns

- Adding dependencies without reviewing transitive tree
- Unpinned versions or missing lockfile
- Ignoring vulnerability scan results without triage
- Unknown or incompatible licenses in production dependencies
- Build scripts that download from network without verification
- AI-generated code treated as trusted without review
- Container images based on unpinned `latest` tags
- CI pipelines using unversioned third-party actions

## Guardian Hooks

Guardians that apply to this guidance:
- `supply_chain`: dependency-manifest-without-lockfile-update, license-unknown, vulnerability-untriaged
- `security_boundary`: build scripts with network access or arbitrary execution
- `operations_readiness`: reproducibility and SBOM coverage for released artifacts
