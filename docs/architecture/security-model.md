# Security Model

The Boundline Security Model is built around a single, uncompromising principle: **The AI is a guest in your repository, not the owner.**

To prevent runaway costs, destructive code mutations, and invisible context drift, Boundline enforces rigid boundaries at the runtime level.

## Explicit Operator Control

Boundline inherently rejects the "black box" agent loop. Execution does not happen asynchronously in the cloud; it happens locally, synchronously, and visibly.

- **No Opaque Loops**: The runtime will pause execution and block the session if it encounters a risk threshold it cannot resolve.
- **Read-Only by Default**: Discovery and planning phases (`boundline plan`) are mathematically guaranteed to be side-effect free on the repository source files.
- **Operator Overrides**: The human operator always retains the ultimate capability to override a council, reject a plan, or manually edit the `.boundline/session.json` state.

## Bounded Execution & Guardians

When `boundline run` executes a plan, it does so under the supervision of **Guardians**.

Guardians are fast, local validation checks that evaluate the outcome of an action before the runtime commits the state. Examples include:
- **Scope Verification**: Did the agent edit a file outside of its assigned context boundary?
- **Syntax Validation**: Does the modified code pass standard compiler checks (`cargo check`, `npm run build`)?
- **Destructive Limits**: Did the agent attempt to delete more lines of code than the `workflows.toml` risk limit allows?

If a Guardian trips, the execution is halted, the state is rolled back, and the session is marked as `blocked`.

## Adaptive Governance Calibration

Governance controls are not binary. Boundline supports graduated control levels
defined in `.boundline/calibration-policy.toml`:

| Level | Behavior |
|---|---|
| **Advisory** | Findings are logged but do not block execution. |
| **Catch** | Findings block execution by default; operator may override. |
| **Rule** | Findings block execution; override requires explicit justification. |
| **Hook** | Findings unconditionally block execution; no override permitted. |

The `boundline override` command lets the operator bypass catch and rule
findings with a trace-visible record. Each override is persisted in the session
trace and audit log. The calibration policy can evolve over time as trust in
the runtime grows — levels can be raised or lowered per finding category.

## Safe Local Execution

The entire architecture is designed to protect your local environment:

- **Checkpoints**: Boundline leverages `.boundline/checkpoints/` to snapshot the state of modified files before applying an AI-generated patch.
- **Isolated Credentials**: Boundline uses `boundline models auth` to manage provider credentials locally. Secrets are never embedded in the traces, context payloads, or session files.
- **Governed Knowledge**: Boundline relies on external boundaries (like Canon) for domain knowledge, ensuring the execution engine remains purely focused on safe delivery mechanics rather than hallucinating business logic.