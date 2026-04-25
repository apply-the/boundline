# Contract: Assistant Chat-Only Fallback

## Purpose

Defines how assistant commands continue a Synod workflow when direct shell execution is unavailable.

## Fallback Exchange Shape

1. The assistant asks only for missing required context.
2. The assistant provides one exact copyable command.
3. The user runs that command locally and pastes the resulting output.
4. The assistant summarizes the result and updates the explicit workflow context.
5. The assistant recommends the next command or asks for one remaining missing field.

## Required Inputs by Workflow

| Command | Minimum Context Before Fallback Can Continue |
|---------|----------------------------------------------|
| `synod-start` | `workspace_ref` |
| `synod-plan` | Broad user goal |
| `synod-step` | Current workflow context or pasted inspection output |
| `synod-run` | `workspace_ref`, `goal` |
| `synod-status` | `workspace_ref` or `trace_ref` |
| `synod-next` | `workspace_ref` or `trace_ref`, plus latest known outcome if already available |
| `synod-inspect` | `trace_ref` or `workspace_ref` |

## Output Handling Rules

- The assistant MUST accept pasted output from `synod doctor`, `synod run`, or `synod inspect`.
- The assistant MUST extract the terminal state, actionable problems, recovery events, trace references, and next-step cues when present.
- The assistant MUST ask for more output only when the pasted content is insufficient to determine the command outcome.
- The assistant MUST preserve already confirmed context and MUST NOT ask the user to re-enter it unless the pasted output contradicts it.

## Non-Success Handling

| Backend Outcome | Assistant Obligation |
|-----------------|----------------------|
| Invalid invocation or missing input | Explain the missing field and provide the corrected command |
| Non-success run | Summarize failure or exhaustion, surface recovery signals if present, and recommend `synod-next` or `synod-inspect` |
| Trace read failure | Ask for a corrected trace reference or workspace and provide the replacement inspect command |
| Readiness failure | Summarize the blocking prerequisite and route back to `synod-start` once corrected |

## Behavioral Guarantees

- Copyable commands must remain directly runnable from the repository root.
- Fallback guidance must remain consistent with the direct shell-enabled path.
- The assistant must always make clear whether it is executing a command, waiting for pasted output, or only routing the user to the next command.