---
description: "Discover named Boundline workflows"
---

# Command: /boundline-workflow-list

Shared guidance: `assistant/README.md`

## Intent
Discover the named workflow entrypoints available in a workspace and keep the
operator on the primary Boundline product surface.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- workflow list --workspace <workspace>` exactly once.

## Chat-Only Path
If shell execution is unavailable, ask only for the missing workspace and then provide this exact copyable command:

`cargo run --bin boundline -- workflow list --workspace <workspace>`

Wait for the user to paste the output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize `workflow registry status`, `workflow_count`, each surfaced `workflow`, `summary`, `recommended_when`, `phases`, `invoke_with`, and `explanation`. Keep the result on the Boundline-owned workflow surface and route to `/boundline-workflow-run` once the user chooses a workflow.

## Next-Step Routing
If the user chooses a workflow name, route to `/boundline-workflow-run`. Otherwise preserve the discovered `invoke_with` guidance instead of inventing provider-specific commands.
Allowed follow-up commands: `/boundline-workflow-run`, `/boundline-status`, `/boundline-goal`.