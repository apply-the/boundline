# Provider Overview

Boundline `0.72.0` adds a native external capability-provider contract.

Providers are bounded capability sources. They are not authorities over
session state, routing policy, or delivery completion. Boundline remains the
runtime that admits, executes, validates, traces, accepts, or rejects provider
results.

## What A Provider Can Do

A provider may expose one or more capabilities through the V1 protocol:

- `capabilities`
- `health`
- `prepare`
- `execute`
- `collect_evidence`

Those operations let Boundline discover capability metadata, confirm runtime
readiness, check required context, execute one bounded request, and normalize
returned evidence.

## What A Provider Cannot Do

Providers do not bypass Boundline runtime policy.

- Discovery does not activate trust.
- Setup incompleteness blocks activation.
- Health failure blocks execution before `execute`.
- Permission conflicts fail closed before execution.
- Patch proposals remain non-authoritative until Boundline validates them.
- Missing or weak evidence can downgrade or reject a provider-backed result.

## Related Pages

- [Provider Protocol](protocol.md)
- [Provider Registration](registration.md)
- [Provider Troubleshooting](troubleshooting.md)
