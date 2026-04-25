---
description: "Plan a Synod workflow"
---

# Command: /synod-plan

Shared guidance: `assistant/README.md`

## Intent
Clarify and bound the user goal so the next step can move into execution without hidden planning.

## Required Context
- A broad user goal
- Workspace readiness when it is already known

## Shell-Enabled Path
No direct CLI invocation is required. Clarify the goal, keep it bounded, and route to `/synod-run` once the goal is actionable.

## Chat-Only Path
Ask only for the missing goal details. Do not invent background work. Once the goal is bounded, route to `/synod-run` with the clarified wording.

## Output Interpretation
Summarize the clarified goal, any missing constraints, and whether the workflow is ready to move into execution.

## Next-Step Routing
Allowed follow-up commands: `/synod-run`, `/synod-start`, `/synod-plan`.
