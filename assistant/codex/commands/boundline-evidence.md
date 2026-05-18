# Command: /boundline-evidence

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Show which runtime, workspace, and Canon-governed signals currently support the active Boundline answer.
Do not collapse Boundline evidence, Canon evidence, and missing evidence into one undifferentiated summary.

## Required Context
- `workspace_ref` or `trace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns

## Shell-Enabled Path
If `trace_ref` is known, run `cargo run --bin boundline -- inspect --trace <trace> --json`.
Otherwise, if `workspace_ref` is known, run `cargo run --bin boundline -- inspect --workspace <workspace> --json`.
If the assistant is already anchored in the workspace, run `cargo run --bin boundline -- inspect --json` exactly once.

## Chat-Only Path
Ask only for the missing `workspace_ref` or `trace_ref`, then provide one exact copyable command:

`cargo run --bin boundline -- inspect --workspace <workspace> --json`

or

`cargo run --bin boundline -- inspect --trace <trace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Summarize `authored_input_summary`, `authored_input_sources`, `authored_input_deduplicated_sources`, `context_summary`, `context_primary_inputs`, `context_provenance`, `decision_timeline`, `failure_evidence`, `adaptive_evidence`, `governance_timeline`, `governance_approval_provenance`, `review_timeline`, and `next_command`. Preserve Canon-grounded provenance, contract-line, packet, approval, readiness, security, audit, or promotion wording exactly when present. If Canon evidence is missing, stale, incompatible, or blocked, list that as a real evidence gap.

## Next-Step Routing
Prefer the CLI-reported `next_command`; if more state is needed, route to `/boundline-inspect`.
Allowed follow-up commands: `/boundline-why`, `/boundline-risk`, `/boundline-next-best`, `/boundline-inspect`, `/boundline-status`, `/boundline-start`.
