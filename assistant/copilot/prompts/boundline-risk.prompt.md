---
description: "Surface the main delivery risks from authoritative Boundline state"
---

# Command: /boundline-risk

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Surface the most relevant delivery risks from authoritative Boundline runtime state and any available Canon-governed signals.
Keep confidence bounded and make missing evidence or governance gaps explicit.

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
Summarize `execution_condition`, `failure_evidence`, `adaptive_evidence`, `review_timeline`, `terminal_status`, `terminal_reason`, `governance_next_action`, `follow_through_guidance`, and `next_command`. Preserve `context_summary`, `context_credibility`, `context_staleness_reason`, and any Canon-governed packet, approval, readiness, security, audit, or promotion wording exactly. When evidence is partial, stale, or missing, state the risk confidence as bounded rather than certain.

## Next-Step Routing
Prefer the CLI-reported `next_command`; when the safest next step is more inspection, route to `/boundline-inspect`.
Allowed follow-up commands: `/boundline-why`, `/boundline-evidence`, `/boundline-next-best`, `/boundline-inspect`, `/boundline-status`, `/boundline-start`.
