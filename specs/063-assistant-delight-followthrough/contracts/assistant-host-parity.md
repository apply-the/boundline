# Contract: Assistant Host Parity

## Scope

This contract defines how assistant hosts expose the S7.1 delight follow-through
surface without hiding support gaps.

## Shared Invariants

- `.boundline/session.json` and CLI-reported state remain authoritative for all
  hosts.
- The default palette remains compact and aligned to the repository-managed
  session workflow contract.
- `explain-plan` and `doctor-context` remain contextual commands unless a host
  explicitly justifies broader visibility.
- Host packaging MUST not invent session state from chat history.
- Every host MUST declare one support mode and one explicit fallback path when
  full parity is not available.

## Support Modes

| Support Mode | Meaning | Required Operator Disclosure |
| --- | --- | --- |
| `repo-local-full` | Repository-managed assistant assets expose the delight surface directly. | Host docs and assets describe the command surface and preserve session or trace authority. |
| `copy-ready-assets` | Boundline ships copy-ready global or bootstrap assets, but host installation details remain environment-specific. | Host docs explain what to copy and what CLI path remains authoritative. |
| `manual-fallback` | Boundline does not claim native packaged parity for the host. | Host docs point directly to the CLI-first fallback path and state the boundary explicitly. |

## Host Matrix

| Host | Current Baseline | S7.1 Contract |
| --- | --- | --- |
| Claude | Repository-managed command assets | Retain delight command coverage and any new S7.1 follow-through disclosure using the same state authority. |
| Codex | Repository-managed command assets | Retain delight command coverage and any new S7.1 follow-through disclosure using the same state authority. |
| Copilot | Repository-managed prompt assets | Retain delight prompt coverage and any new S7.1 follow-through disclosure using the same state authority. |
| Cursor | Copy-ready bootstrap assets | Either remain an explicit copy-ready asset surface with documented fallback or intentionally graduate to richer parity. No undocumented middle state is allowed. |
| Gemini | Explicit CLI-first fallback | Either remain an explicit manual fallback with clear CLI guidance or intentionally graduate to richer parity. No implicit native support claims are allowed. |

## Command Coverage Rules

- Hosts with repository-managed delight assets MUST keep the existing delight
  command set aligned with the session workflow contract.
- Cursor and Gemini decisions MUST be visible in manifests or docs before the
  implementation is considered done.
- If a host cannot expose a delight surface directly, the fallback path MUST
  point to the Boundline CLI command that preserves the same authority model.

## Validation Rules

- Contract tests MUST continue to validate required Claude, Codex, and Copilot
  assets.
- Cursor and Gemini docs or manifests MUST explicitly encode their support mode.
- Support mode changes MUST update both the relevant host docs and the shared
  manifest surface.