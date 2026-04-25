# Data Model: Assistant Command Packs

## AssistantCommandPack

Represents the full command-pack surface shipped for one assistant environment.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `assistant_id` | Enum | Yes | `claude`, `codex`, or `copilot` |
| `root_ref` | Repository-relative path | Yes | Root folder containing the assistant's command-pack files |
| `installation_surface` | Enum | Yes | `commands` for Claude/Codex, `prompts` for Copilot |
| `command_definitions` | Ordered list of `AssistantCommandDefinition` | Yes | Must include the complete initial command set |
| `readme_ref` | Repository-relative path | Yes | Points to the shared `assistant/README.md` file |
| `supported_execution_modes` | Set | Yes | Must include `shell-enabled` and `chat-only` |

### Validation Rules

- `assistant_id` must be unique per pack.
- `command_definitions` must include exactly one definition for each required command name.
- `installation_surface` must match the folder and file naming rules defined in the command-pack contract.
- `readme_ref` must resolve to the shared `assistant/README.md` file.

### Relationships

- One `AssistantCommandPack` contains many `AssistantCommandDefinition` records.
- One `AssistantCommandPack` references one shared documentation surface.

## AssistantCommandDefinition

Represents one assistant-facing workflow command.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `command_name` | Enum | Yes | `synod-start`, `synod-plan`, `synod-step`, `synod-run`, `synod-status`, `synod-next`, or `synod-inspect` |
| `assistant_id` | Enum | Yes | Owning assistant pack |
| `user_intent` | Non-empty string | Yes | Plain-language outcome the user expects |
| `required_inputs` | Ordered list of input descriptors | Yes | Includes required and optional fields such as workspace, goal, or trace reference |
| `direct_backend` | Enum or null | No | One of `doctor`, `run`, `inspect-workspace`, `inspect-trace`, or `none` for routing-only commands |
| `chat_fallback_examples` | Ordered list of copyable commands | Yes | Exact commands the user can run manually when shell execution is unavailable |
| `summary_focus` | Ordered list | Yes | Defines what the assistant must extract from direct output or pasted output |
| `next_routes` | Ordered list of command names | Yes | Allowed follow-up commands after success, failure, or missing context |

### Validation Rules

- Every definition must describe both shell-enabled and chat-only behavior.
- `direct_backend = none` is allowed only when the definition explicitly collects context or routes to another command.
- `chat_fallback_examples` must be executable against the current CLI surface.
- `next_routes` must contain only command names from the initial assistant command set.

### Relationships

- Belongs to one `AssistantCommandPack`.
- Consumes and updates one `ConversationWorkflowContext` per invocation.
- Produces zero or one `CommandOutcomeSummary`.

### State Transitions

`invoked` -> `collecting_context` -> `ready`

`ready` -> `executing_direct_backend` -> `summarized`

`ready` -> `waiting_for_pasted_output` -> `summarized`

`collecting_context` -> `routed`

## ConversationWorkflowContext

Represents the explicit conversational state carried across assistant commands.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `workspace_ref` | Path-like string | No | Required before any direct CLI command can run |
| `goal` | Non-empty string | No | Required before `/synod-run`; may be clarified by `/synod-plan` |
| `latest_trace_ref` | Path-like string | No | Preferred for explicit inspection when available |
| `latest_terminal_status` | Enum | No | Latest known terminal outcome from run or inspect results |
| `last_command_name` | Enum | No | Most recent assistant command executed |
| `pending_question` | String | No | Missing input the assistant still needs from the user |
| `recommended_next_command` | Enum | No | Next command suggested after summarization |

### Validation Rules

- `workspace_ref` or `latest_trace_ref` must be present before `status`, `next`, or `inspect` can continue.
- `goal` must be present before `run` can continue.
- `pending_question` must be empty once a command reaches `ready`, `summarized`, or `routed`.

### Relationships

- Read and written by every `AssistantCommandDefinition`.
- Supplies input for `CommandOutcomeSummary` routing decisions.

### State Transitions

`empty` -> `scoped`

`scoped` -> `ready_to_run`

`ready_to_run` -> `active_or_inspectable`

`active_or_inspectable` -> `summarized`

`summarized` -> `rerouted`

## CommandOutcomeSummary

Represents the user-readable result returned by an assistant command after direct execution or pasted-output interpretation.

### Fields

| Field | Shape | Required | Notes |
|-------|-------|----------|-------|
| `source_command` | Enum | Yes | The assistant command that produced the summary |
| `backend_path` | Enum | Yes | `direct-cli`, `chat-fallback`, or `routing-only` |
| `terminal_status` | Enum | No | Present when the command reached an execution or inspection outcome |
| `terminal_reason` | String | No | Readable reason when the underlying CLI surfaced one |
| `key_results` | Ordered list of strings | Yes | Most important step, status, or context findings |
| `recovery_signals` | Ordered list of strings | No | Retry or replanning cues surfaced from the trace summary |
| `trace_ref` | Path-like string | No | Attached when available from run or inspect output |
| `next_command` | Enum | No | Suggested follow-up command |

### Validation Rules

- `key_results` must never be empty.
- If `terminal_status` is non-success, `terminal_reason` or at least one `recovery_signal` must be present.
- `next_command` must be one of the allowed routes declared by the source command definition.

### Relationships

- Produced by one `AssistantCommandDefinition`.
- Updates one `ConversationWorkflowContext`.