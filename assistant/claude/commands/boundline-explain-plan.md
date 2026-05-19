# Command: /boundline-explain-plan

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

Host support stays explicit in this release: Claude uses `repo-local-full`, Cursor remains `copy-ready-assets`, and Gemini remains `manual-fallback`. Do not imply richer parity for Cursor or Gemini than the declared support mode.

## Intent
Explain the current bounded plan in human terms using the active session-native state.
Keep validation, governance, and recovery posture explicit instead of implied.

## Required Context
- `workspace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns when status references it

## Shell-Enabled Path
If `workspace_ref` is known, run `cargo run --bin boundline -- status --workspace <workspace> --json`.
If the assistant is already anchored in the workspace, run `cargo run --bin boundline -- status --json` exactly once.
If a trace needs deeper follow-up after status, use `cargo run --bin boundline -- inspect --trace <trace> --json`.

## Chat-Only Path
Ask only for the missing `workspace_ref`, then provide one exact copyable command:

`cargo run --bin boundline -- status --workspace <workspace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Summarize `explain_plan_summary`, `explain_plan_validation`, `explain_plan_governance`, `explain_plan_recovery`, `why_summary`, `risk_summary`, `reasoning_profile_id`, `reasoning_selection_reason`, `reasoning_contribution`, `reasoning_fallback_disclosure`, `time_to_first_useful_answer_ms`, `time_to_first_useful_answer_command`, `explanation_attribution_rate`, `next_action_acceptance_rate`, `latest_next_action_outcome`, and `next_command`. Preserve any Canon or blocked-state wording exactly and keep the recovery posture visible instead of turning it into generic advice.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If the plan explanation points to a trace-only follow-up, route to `/boundline-inspect`.
Allowed follow-up commands: `/boundline-next-best`, `/boundline-risk`, `/boundline-assumptions`, `/boundline-inspect`, `/boundline-status`, `/boundline-start`.
