# Security Policy

## Supported Versions

Boundline is still pre-`1.0.0`. Security fixes are provided for the latest
release line only.

| Release line | Support status |
| --- | --- |
| Latest tagged release | Supported |
| `main` | Best effort, unstable |
| Older releases | Not supported |

If you are running an older release, upgrade to the latest published version
before reporting a bug unless the upgrade itself is blocked by the issue.

## Reporting a Vulnerability

Please do not open a public GitHub issue with working exploit details.

Preferred path:

1. Use GitHub private vulnerability reporting for this repository through the
   Security tab when it is available.
2. Include the affected Boundline version or commit, platform, install method,
   impact, reproduction steps, and any proof-of-concept needed to validate the
   issue.
3. Redact secrets, tokens, private repository content, customer data, and any
   unrelated `.boundline/` or `.canon/` workspace artifacts.

If private vulnerability reporting is not available in the repository UI,
open a minimal public issue requesting a private contact path and do not attach
exploit details, secrets, or sensitive logs to that issue.

## What To Include

Useful reports usually include:

- the Boundline version from `boundline --version`, or the exact commit SHA
- the installation path used: Homebrew, winget, `cargo install --path .`, or a
  local source build
- the operating system and architecture
- the commands, workflow surface, assistant host, or Canon integration path involved
- expected behavior, actual behavior, and the security impact
- a minimal reproduction, patch sketch, or mitigation if you already have one

## Scope

This policy covers security issues in artifacts maintained in this repository,
including:

- the `boundline` CLI and workspace crates
- packaged release artifacts and install metadata
- assistant command packs and shared metadata under `assistant/`
- shipped repository assets, prompts, and workflow configuration examples
- CI or release automation in `.github/workflows/`, `distribution/`, and `scripts/release/`

Issues that only exist in third-party services or host applications outside
this repository should also be reported upstream. Dependency advisories are
still useful to report here when they are exploitable through Boundline's
shipped or documented configurations.

## Response and Disclosure

This project does not currently publish a formal security response SLA.
Maintainers will triage reports as capacity allows, may ask for more detail or
a minimal reproduction, and will try to coordinate a fix before public
disclosure.

When a fix ships, the user-visible change should appear in release notes or the
changelog for the affected release.

## Security Hygiene

- Prefer official release surfaces documented in the README.
- Stay on the latest release line.
- Use least-privilege credentials when running Boundline in sensitive repositories.
- Review generated workflow assets, assistant command pack content, and local
  automation before using them in higher-risk environments.
- Dependency advisories are checked in CI with `cargo deny check advisories`.