# Contract: Assistant Command Definition

## Purpose

Defines the required structure and backend mapping for every assistant command file.

## Required Sections

Every command file MUST include these sections, using assistant-appropriate wording while preserving the same meaning:

| Section | Purpose |
|---------|---------|
| Intent | States the user goal the command serves |
| Required Context | Lists the minimum inputs needed, such as workspace, goal, or trace reference |
| Shell-Enabled Path | States what the assistant runs directly when shell execution is available |
| Chat-Only Path | Gives exact copyable commands and explains what output to paste back |
| Output Interpretation | Defines what the assistant must summarize rather than dumping raw logs |
| Next-Step Routing | States the allowed follow-up commands for success, failure, or missing context |

## Backend Mapping Rules

| Assistant Command | Direct Backend | Notes |
|-------------------|----------------|-------|
| `synod-start` | `synod doctor --workspace <workspace>` when workspace is known | Establishes readiness and missing context; may remain routing-only until workspace is provided |
| `synod-plan` | None required | Clarifies or bounds the goal, then routes to `synod-run` |
| `synod-step` | None required | Advances the workflow by selecting one explicit next action from current context or latest inspection evidence |
| `synod-run` | `synod run --workspace <workspace> --goal <goal>` | Primary execution path |
| `synod-status` | `synod inspect --workspace <workspace>` | Uses latest trace in the workspace to summarize current status |
| `synod-next` | `synod inspect --workspace <workspace>` when evidence is needed | Uses latest trace plus current context to recommend the next command |
| `synod-inspect` | `synod inspect --trace <trace>` or `synod inspect --workspace <workspace>` | Explicit inspection path |

## Output Interpretation Guarantees

- `synod-run`, `synod-status`, `synod-next`, and `synod-inspect` must summarize terminal status, key step results, recovery signals when present, and the most useful next command.
- `synod-start` must summarize readiness and missing prerequisites.
- `synod-plan` and `synod-step` must summarize the clarified goal or next action and explain why that route is being recommended.
- No command may dump raw CLI logs without summarization.

## Routing Guarantees

- Commands with no direct backend MUST end in one of two explicit outcomes: `ready-to-run` or `needs-more-context`.
- A command may recommend only commands from the required command set.
- A command may not imply background execution or hidden follow-up actions.