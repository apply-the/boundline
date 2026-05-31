# Contract: Known Profile And Versioning

## Purpose

Define the cross-repo contract for the first shipped known profile (`speckit`),
including which repository owns which responsibility and how compatibility is
expressed without lockstep releases.

## Known Profile Definition: `speckit`

- `adapter_id`: `speckit`
- `display_name`: `Speckit`
- `registration_command`: `boundline adapter add speckit`
- `default_binary`: `boundline-adapter-speckit`
- `compatibility_line`: `framework-adapter-v1`
- `adapter_repo_ref`: `../boundline-adapter-speckit/`
- `template_repo_ref`: `../boundline-framework-template/`

## Repository Responsibilities

### Boundline repo (`./`)

Owns:

- the adapter-management CLI surface
- the persisted config model in `.boundline/config.toml`
- the stdio protocol contract and typed serde models
- the stage and hook catalog used for validation
- lifecycle routing, fallback semantics, audit records, and operator-facing
  output
- the known-profile registry entry for `speckit`
- documentation and release notes describing the host-side feature

Must not own:

- the reusable template implementation itself
- Speckit-specific business logic or profile-specific execution code

### Template repo (`../boundline-framework-template/`)

Owns:

- the reusable Rust scaffold for external adapters
- example implementations of `describe`, `preflight`, `execute-stage`, and
  `emit-hook`
- template tests and golden fixture usage against the host contract
- template README guidance for creating a new custom adapter repo

Must not own:

- Boundline host runtime logic
- Speckit-specific defaults or release branding

### Speckit adapter repo (`../boundline-adapter-speckit/`)

Owns:

- the concrete `boundline-adapter-speckit` binary
- Speckit-specific capability declarations and required config schema
- Speckit-specific stage execution and hook behavior
- repository-local tests and install or usage docs for the Speckit profile

Must not own:

- the host CLI contract for registration or lifecycle routing
- the generic template scaffold used by other adapters

## Compatibility Rules

- Boundline publishes the protocol line through the host-known profile and the
  `boundline-adapters` crate.
- The template repo pins a released `boundline-adapters` reference and declares
  the same compatibility line in its README and package metadata.
- The Speckit adapter repo pins a released `boundline-adapters` reference and
  reports the same compatibility line from `describe`.
- The adapter also reports a supported Boundline version range. If the running
  Boundline version falls outside that range, activation is blocked before stage
  routing begins.

## Release Coordination Rules

- Boundline may release host changes without lockstep template or Speckit
  releases as long as the compatibility line and version range stay valid.
- Any breaking protocol change requires a new compatibility line and explicit
  docs updates in all three repos.
- Non-breaking additions may keep the same line, but they must remain backward
  compatible for adapters already advertising that line.
- The template and Speckit repos must never commit a local path dependency back
  to the Boundline workspace; they depend on a released git tag instead.

## Required Docs And Testing Touchpoints

- Boundline docs: configuration, getting-started, architecture, release notes,
  and changelog entries for host behavior
- Template docs: README and bootstrap instructions for creating a new adapter
- Speckit docs: README, installation notes, profile-specific config guidance,
  and compatibility notes
- Cross-repo validation: CI or release checks should compile the template repo,
  compile the Speckit repo, and run at least one Boundline temp-workspace smoke
  flow using the same compatibility line

## Blocking States

Activation must be blocked when any of the following are true:

- the known profile is requested but the resolved binary is missing
- the adapter reports a different `adapter_id` than the selected known profile
- the protocol line is unsupported by the running Boundline version
- the adapter's supported Boundline version range excludes the current host
- the template or Speckit repo attempts to rely on a non-release local path
  dependency in committed files