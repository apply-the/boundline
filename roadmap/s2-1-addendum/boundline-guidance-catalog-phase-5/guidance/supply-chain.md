# Supply Chain Guidance

## Purpose

This guidance defines supply chain expectations for AI-assisted delivery.

It applies to dependencies, package managers, build pipelines, generated code, third-party tools, CI scripts, container images, and release artifacts.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, security policy, supply-chain policy, or Canon-governed standard.

## Core Thesis

A software change does not only modify source code.

It may also modify the trust boundary of the build and runtime environment.

Supply chain guidance asks:

- what dependencies were added?
- who maintains them?
- what license applies?
- are versions pinned?
- is the lockfile updated?
- do install/build scripts execute?
- are vulnerabilities known and triaged?
- does the dependency create runtime or build-time risk?
- does generated code introduce hidden provenance?

## Dependency Introduction

Before adding a dependency, ask:
- is it necessary?
- is it actively maintained?
- is it widely used enough for the risk profile?
- does it have a compatible license?
- does it run install scripts?
- does it pull risky transitive dependencies?
- can standard library or existing dependency solve this?
- who owns future updates?

AI-generated code often adds dependencies without evaluating lifecycle cost.

## Version Pinning And Lockfiles

Package manager lockfiles protect reproducibility.

Required expectations:
- lockfiles updated with dependency changes
- versions pinned according to ecosystem norms
- dependency diffs reviewed
- generated lockfile churn minimized

Guardians should flag:
- dependency file changed without lockfile
- lockfile changed without dependency manifest explanation
- broad version ranges where repo policy requires pinning
- surprising transitive dependency growth

## Vulnerability Management

Vulnerabilities must be triaged, not blindly ignored.

For each material finding:
- severity
- exploitability
- affected path
- runtime exposure
- fix availability
- mitigation
- accepted risk owner
- expiration for waiver

A vulnerability scanner finding without triage is not evidence of safety.

## License Risk

Dependency licenses must be compatible with the project.

Guardians should flag:
- unknown license
- copyleft license where policy forbids it
- dual license requiring decision
- missing license metadata
- generated code with unclear license

## Build Scripts And Install Hooks

Install scripts and build hooks are supply-chain risk.

Examples:
- npm `postinstall`
- build.rs in Rust
- Gradle/Maven plugins
- Python setup hooks
- shell scripts in CI
- Dockerfile curl-pipe-shell patterns

Review:
- what executes?
- when does it execute?
- where does code come from?
- can it access secrets?
- is it pinned?
- is checksum verified?

## Generated Code And Provenance

AI-generated or tool-generated code should preserve provenance where relevant.

Generated artifacts should answer:
- what generated this?
- can it be regenerated?
- is it checked in intentionally?
- is source-of-truth clear?
- is manual editing allowed?

## Container And CI Supply Chain

Check:
- base image pinning
- digest pinning where required
- least privilege builds
- secrets in build logs
- untrusted scripts
- dependency cache poisoning
- CI permissions
- release artifact signing where needed

## SBOM

For higher-governance systems, SBOM generation may be required.

SBOM should support:
- dependency inventory
- vulnerability triage
- license review
- release provenance

SBOM without review process is incomplete.

## Anti-Patterns

- adding dependency for trivial helper
- dependency manifest changed without lockfile
- lockfile churn not reviewed
- unknown licenses
- ignored vulnerability findings
- permanent vulnerability waivers
- curl-pipe-shell in build scripts
- unpinned base images
- install scripts not reviewed
- generated code without provenance
- CI job with broad secrets and untrusted inputs

## Guardian Hooks

Recommended guardians:
- dependency-introduction-guardian
- lockfile-consistency-guardian
- vulnerability-triage-guardian
- license-policy-guardian
- install-script-risk-guardian
- container-base-image-guardian
- generated-code-provenance-guardian
- ci-permission-guardian

## Structured Finding Example

```json
{
  "guardian": "lockfile-consistency",
  "rule": "dependency-manifest-without-lockfile-update",
  "disposition": "warning",
  "summary": "A new npm dependency was added but the lockfile was not updated.",
  "evidence_refs": ["package.json"],
  "recommended_action": "Update the lockfile or explain why this package manager does not require one."
}
```

## Lifecycle Usage

Planning:
- identify dependency or tooling changes

Implementation:
- check new dependency necessity and provenance

Testing:
- verify scanner results and lockfile consistency

Review:
- inspect dependency diff, license, vulnerability, and build script risk

Verification:
- produce evidence for accepted risk or remediation

Supply-chain-analysis:
- perform full posture assessment
