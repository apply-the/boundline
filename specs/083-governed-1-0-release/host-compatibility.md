# Host Compatibility Matrix

Stable 1.0 compatibility covers the generated schema, calls to Boundline
services, and semantic equivalence of normalized authoritative projections.
It does not promise compatibility with untested future host releases.

## M0 host inventory

Version probes were performed on 2026-07-25 on the recorded macOS arm64
baseline. A successful version probe is an inventory observation, not semantic
projection qualification.

| Host | Locally installed or tested version | M0 execution evidence | Schema / projection state | M0 disposition |
|---|---|---|---|---|
| Claude | Claude Code `2.0.65` | `claude --version` succeeded | Frozen pack and projection fixtures do not exist yet | Unqualified; Release Owner; T076/M5; no stable compatibility claim until qualification |
| Codex | `codex-cli 0.146.0-alpha.3` | `codex --version` succeeded; the launcher also reported a non-fatal sandbox PATH warning | Frozen pack and projection fixtures do not exist yet | Unqualified; Release Owner; T076/M5; no stable compatibility claim until qualification |
| GitHub Copilot | No `gh` or `copilot` executable located | Not executable on the M0 host | Frozen pack and projection fixtures do not exist yet | Unqualified; Release Owner; T076/M5; fail closed rather than infer support from model availability |
| Cursor | No CLI executable or application bundle located | Not executable on the M0 host | Frozen pack and projection fixtures do not exist yet | Unqualified; Release Owner; T076/M5; fail closed |
| Antigravity | Antigravity `2.3.1`; Antigravity IDE `2.1.1` | Bundle versions read locally; no executable host invocation was available | Frozen pack and projection fixtures do not exist yet | Unqualified; Release Owner; T076/M5; fail closed |

The M0 acceptance condition is an exact inventory plus an owned disposition
for every unavailable host. It deliberately does not claim semantic
equivalence before M5 creates the frozen machine interfaces and generated host
packs. T076 owns final host execution and normalized projection qualification;
T073 supplies the cross-repository projection comparator.

For every qualified row, release evidence must record:

- exact host version and operating system;
- generated pack digest;
- supported invocation surface;
- normalized capability, status, inspect, trace, and completion projections;
- declared volatile fields removed before semantic comparison;
- unsupported or preview capabilities;
- fixture commit and execution result.

No row becomes stable from schema generation alone. A tested host invocation
and semantically equivalent projection result are both required.
