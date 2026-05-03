# Research: Assistant Command Packs

## Decision 1: Ship assistant assets from a top-level `assistant/` directory

- **Decision**: Store the command packs in a repository-managed `assistant/` directory with one installation/readme surface plus per-assistant folders for Claude, Codex, and Copilot.
- **Rationale**: The feature's product is a set of assistant-facing assets rather than a new runtime component. A top-level directory keeps the asset surface explicit, portable, and easy to document without coupling it to internal build or agent-customization files.
- **Alternatives considered**:
  - Reuse `.agents/` for user-facing command packs: rejected because `.agents/` already serves repository-internal skill and workflow instructions rather than end-user command-pack installation assets.
  - Hide the assets under `.github/` or another assistant-specific directory: rejected because the feature must remain portable across multiple assistant environments rather than privileging one platform.

## Decision 2: Author per-assistant files directly, but enforce one shared command contract

- **Decision**: Create assistant-specific files for Claude, Codex, and Copilot directly in the repository while enforcing a shared structure and coverage contract through documentation and tests.
- **Rationale**: Direct files are the smallest viable slice. They preserve the assistant-specific packaging each environment expects while avoiding the extra generator, templates, and stale-output problem a single-source pipeline would add in the first release.
- **Alternatives considered**:
  - Generate all packs from a single source: rejected for the initial slice because it adds a build step and code-generation maintenance before the command set has proven stable.
  - Let each assistant pack evolve independently with no shared structure: rejected because semantic drift would quickly break cross-assistant consistency.

## Decision 3: Reuse the existing CLI backend instead of adding new runtime services or workflow subcommands

- **Decision**: Use the current Boundline CLI commands as the direct execution backend: `doctor` for readiness checks, `run` for bounded execution, and `inspect` for latest-trace or explicit-trace summaries. Model `start`, `plan`, `step`, `status`, and `next` as assistant-native workflow commands that gather context, route to existing CLI commands, and interpret results.
- **Rationale**: The repository already has a usable local execution and inspection surface. Reusing it keeps the feature within the existing CLI-first architecture and avoids expanding the orchestration runtime just to satisfy assistant packaging.
- **Alternatives considered**:
  - Add new CLI subcommands for every assistant command name: rejected because it would expand the runtime surface and require new execution semantics before the assistant packs themselves have shipped.
  - Maintain workflow state only in hidden assistant memory: rejected because the constitution requires explicit, inspectable decision paths rather than opaque conversation state.

## Decision 4: Make workflow continuity explicit through conversation context and trace references

- **Decision**: Define a small explicit context model for assistant commands consisting of workspace reference, goal, latest trace reference, latest terminal outcome, and pending user input.
- **Rationale**: Assistant-native workflows still need state continuity, especially in chat-only environments. Keeping that state explicit in the command contract allows the assistant to continue a workflow without hidden heuristics and without inventing a new persistence service.
- **Alternatives considered**:
  - Introduce a new file-backed workflow session store: rejected for the first slice because the existing trace store plus conversation context already cover the required continuity and inspectability.
  - Require users to restate all inputs on every command: rejected because it breaks the core usability goal of command-pack continuity.

## Decision 5: Validate command-pack completeness and CLI alignment with Rust tests

- **Decision**: Add Rust-based contract, unit, and integration tests that load assistant assets from disk, verify command coverage and required sections, and exercise shell-enabled plus chat-only flows against the current CLI output surfaces.
- **Rationale**: The repository already relies on Cargo-based validation. Rust tests can check asset completeness and backend mapping portably in CI without introducing a separate shell validation toolchain.
- **Alternatives considered**:
  - Shell-only validation scripts: rejected because they add portability work and duplicate the existing test harness.
  - Manual review only: rejected because this feature is highly susceptible to drift between command assets and CLI behavior.

## Decision 6: Use latest-trace inspection as the status and next-step evidence source

- **Decision**: Have assistant `status`, `next`, and fallback inspection paths rely on `boundline inspect --workspace <workspace>` when no explicit trace path is provided, and `boundline inspect --trace <trace>` when a concrete trace reference is available.
- **Rationale**: The existing inspect command already exposes the most useful evidence for follow-up routing: terminal status, recovery events, and readable step summaries. Reusing it avoids a second status backend.
- **Alternatives considered**:
  - Create a separate status data format: rejected because it duplicates the readable trace-summary surface that already exists.
  - Infer next steps without inspection evidence: rejected because it would reintroduce hidden assistant logic and weaken trace-backed guidance.

*** Add File: /Users/rt/workspace/boundline/specs/003-assistant-command-packs/data-model.md
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
| `command_name` | Enum | Yes | `boundline-start`, `boundline-plan`, `boundline-step`, `boundline-run`, `boundline-status`, `boundline-next`, or `boundline-inspect` |
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
| `goal` | Non-empty string | No | Required before `/boundline-run`; may be clarified by `/boundline-plan` |
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

*** Add File: /Users/rt/workspace/boundline/specs/003-assistant-command-packs/quickstart.md
# Quickstart: Assistant Command Packs

## Prerequisites

1. Work from the repository root on branch `003-assistant-command-packs`.
2. Have Rust 1.95.0 with `cargo` available so the local Boundline CLI can run.
3. Use a writable workspace so Boundline can persist traces under `.boundline/traces/`.
4. Choose one supported assistant environment: Claude, Codex, or Copilot.

## Asset Layout

- Shared installation and usage guidance lives in `assistant/README.md`.
- Claude command files live in `assistant/claude/commands/`.
- Codex command files live in `assistant/codex/commands/`.
- Copilot prompt files live in `assistant/copilot/prompts/`.

## Shell-Enabled Walkthrough

### 1. Start from chat

Invoke `/boundline-start` in your assistant.

Expected outcome:

- The assistant asks for the workspace only if it is missing.
- The assistant runs or recommends:

```bash
cargo run --bin boundline -- doctor --workspace "$PWD"
```

- The assistant summarizes whether the workspace is ready and what prerequisite, if any, must be fixed.

### 2. Bound the goal

Invoke `/boundline-plan`.

Expected outcome:

- The assistant asks only for the missing goal details.
- The assistant turns the goal into a bounded `boundline run` objective.
- The assistant routes directly to `/boundline-run`.

### 3. Execute the workflow

Invoke `/boundline-run`.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- run --workspace "$PWD" --goal "Summarize the current bounded developer flow"
```

- The assistant summarizes the terminal status, recovery signals, and trace location.

### 4. Check latest status or next step

Invoke `/boundline-status` or `/boundline-next`.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- inspect --workspace "$PWD"
```

- `/boundline-status` summarizes the latest trace.
- `/boundline-next` uses that same evidence to recommend the most relevant follow-up command.

### 5. Inspect a specific trace

Invoke `/boundline-inspect` with a trace path when you need a specific run rather than the latest one.

Expected outcome:

- The assistant runs or recommends:

```bash
cargo run --bin boundline -- inspect --trace "$PWD/.boundline/traces/<task-id>.json"
```

- The assistant summarizes final status, recovery events, and next action guidance.

## Chat-Only Walkthrough

1. Invoke the same assistant command.
2. Let the assistant ask only for missing inputs.
3. Copy the provided `cargo run --bin boundline -- ...` command into your terminal.
4. Paste the command output back into the chat.
5. Follow the assistant's summary and next-step recommendation.

Minimum fallback checkpoints:

- `/boundline-start` must recover from a not-ready workspace.
- `/boundline-run` must surface a trace location even for non-success outcomes.
- `/boundline-status`, `/boundline-next`, and `/boundline-inspect` must continue from either a workspace or an explicit trace path.

## Validation Commands

Run these commands from the repository root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Minimum Validation Scenarios

1. Each supported assistant exposes the full seven-command pack.
2. `/boundline-start` and `/boundline-run` work in both shell-enabled and chat-only modes.
3. `/boundline-status` and `/boundline-next` can summarize the latest trace without requiring raw log inspection.
4. `/boundline-inspect` can explain a specific run using only a trace path or workspace reference.