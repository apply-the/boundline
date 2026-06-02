# Boundline

> [!TIP]
> This wiki is aligned with **Boundline 0.66.0**. For older versions, refer to the repository tags.

![Boundline - Bounded Delivery Runtime](https://github.com/apply-the/boundline/blob/main/tech-docs/images/boundline-banner.jpg)

**The local delivery orchestrator for bounded engineering work.** Turn goals into executed plans safely, without losing control to an opaque AI loop.

## <i class="fa-solid fa-rocket"></i> Why Boundline?

- <i class="fa-solid fa-bullseye"></i> **Goal-Driven Execution:** Translates high-level objectives into concrete, step-by-step technical plans.
- <i class="fa-solid fa-floppy-disk"></i> **Session-Based State:** Maintains explicit, resumable session state locally on disk. You are never hostage to ephemeral chat memory.
- <i class="fa-solid fa-hand"></i> **Safe Delivery:** Executes steps safely using your repository's existing constraints and Canon governance rules.
- <i class="fa-solid fa-file-signature"></i> **Explicit Traces:** Never lose context. Every execution step is recorded in local, auditable traces.
- <i class="fa-solid fa-plug"></i> **Agnostic Architecture:** Seamlessly plugs into external frameworks and capability providers.

## <i class="fa-solid fa-brain"></i> How it Works

Boundline forces an explicit, inspectable workflow:
1. `goal` → Record the objective for the active session.
2. `plan` → Draft the bounded work from the repository evidence.
3. `run` → Execute the next approved step.
4. `inspect` → Report the authoritative runtime state.

## <i class="fa-solid fa-bolt"></i> Quick Start

```bash
boundline doctor --install
cd my-project
boundline init --assistant codex --route planning=copilot:gpt-4o
boundline config set-semantic-acceleration --scope workspace --policy local
boundline index status --workspace .
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
```

If the workspace needs one explicit framework adapter, register it after init:

```bash
boundline adapter add speckit --workspace .
boundline adapter show --workspace . --json
```

The adapter JSON report surfaces the compatibility line, declared supported
transports, stage overrides, hook subscriptions, and config-completeness state
before `plan` or `run` tries to hand off a stage.

Current public repositories for this adapter line:

- [boundline-framework-template](https://github.com/apply-the/boundline-framework-template): starter scaffold for a compatible framework adapter that speaks the host-owned V1 subprocess contract.
- [boundline-adapter-speckit](https://github.com/apply-the/boundline-adapter-speckit): concrete Speckit-backed adapter implementation that can claim `plan` and `run` when preflight succeeds.

## <i class="fa-solid fa-hammer"></i> Key Commands

| Command | What it does |
|---|---|
| `boundline goal` | Set the objective for the current session. |
| `boundline plan` | Generate a technical plan to achieve the goal. |
| `boundline run` | Execute the next pending step in the plan. |
| `boundline status` | Check the current session status and next actions. |
| `boundline inspect` | View detailed execution traces and evidence. |
| `boundline adapter show --json` | Inspect adapter compatibility, transports, and config readiness. |
| `boundline index status` | Report derived-index lifecycle state for local semantic retrieval. |
| `boundline index doctor` | Diagnose tracked, stale, corrupt, or degraded derived-index state. |

## <i class="fa-solid fa-book"></i> Deep Dive Documentation

Explore the wiki sidebar for details on architecture, configuration, and scaling:
- [Getting Started](Getting-Started)
- [Configuration and Precedence](Configuration-Reference)
- [Architecture and Decisions](Architecture-And-Decisions)
- [Project Scale Delivery Model](Project-Scale-Delivery)
- [Assistant Integrations](Assistant-Integrations)