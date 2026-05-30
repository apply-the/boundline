# Boundline

![Boundline banner](docs/images/boundline-banner.jpg)
[![Version](https://img.shields.io/github/v/release/apply-the/boundline?color=blue&label=version)](https://github.com/apply-the/boundline/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/apply-the/boundline/actions/workflows/ci.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/ci.yml)
[![Lint](https://github.com/apply-the/boundline/actions/workflows/lint.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/lint.yml)
[![Vulnerabilities](https://github.com/apply-the/boundline/actions/workflows/vulnerabilities.yml/badge.svg)](https://github.com/apply-the/boundline/actions/workflows/vulnerabilities.yml)
[![Coverage](https://codecov.io/gh/apply-the/boundline/branch/main/graph/badge.svg)](https://codecov.io/gh/apply-the/boundline)
[![Quality Gate](https://sonarcloud.io/api/project_badges/measure?project=apply-the_boundline&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=apply-the_boundline)

**The local delivery orchestrator for bounded engineering work.** Turn goals into executed plans safely, without losing control to an opaque AI loop.

## 🚀 Why Boundline?

- 🎯 **Goal-Driven Execution:** Translates high-level objectives into concrete, step-by-step technical plans.
- 💾 **Session-Based State:** Maintains explicit, resumable session state locally on disk. You are never hostage to ephemeral chat memory.
- 🛑 **Safe Delivery:** Executes steps safely using your repository's existing constraints and Canon governance rules.
- 📝 **Explicit Traces:** Never lose context. Every execution step is recorded in local, auditable traces.
- 🔌 **Agnostic Architecture:** Seamlessly plugs into external frameworks and capability providers.

## 🧠 How it Works

Boundline forces an explicit, inspectable workflow:
1. `goal` -> Record the objective for the active session.
2. `plan` -> Draft the bounded work from the repository evidence.
3. `run` -> Execute the next approved step.
4. `inspect` -> Report the authoritative runtime state.

## ⚡ Quick Start

```bash
boundline doctor --install
cd my-project
boundline init --assistant codex --ide vscode 
boundline goal --goal "Fix the failing add test"
boundline plan
boundline run
```

## Use Boundline from chat

Install the assistant pack for your host with `boundline init --assistant <host>` or
`boundline assistant install --host <host> --scope user`, then drive the same
session-native lifecycle from chat. The assistant surface should keep
`.boundline/session.json` authoritative, surface the runtime `next_command`, and
stop cleanly on blocked, clarification-required, failed, exhausted, and terminal
states instead of inventing parallel workflow state.

## Use Boundline from CLI

The CLI remains the source of truth for repo state and delivery progress. Use
`boundline doctor --install` to verify the local runtime, `boundline init` to
bootstrap a workspace, then run `boundline goal`, `boundline plan`,
`boundline run`, `boundline status`, `boundline next`, and
`boundline inspect` as the bounded session advances.

## How chat commands map to CLI/runtime state

Chat command packs are thin wrappers over the Rust runtime. `/boundline:goal`,
`/boundline:plan`, `/boundline:run`, `/boundline:status`, `/boundline:next`, and
`/boundline:inspect` should map directly to the corresponding CLI commands and the
same persisted session and trace state under `.boundline/session.json` and
`.boundline/traces/`. Chat history is advisory only; the CLI runtime and its
persisted outputs remain authoritative.

## 🛠️ Key Commands

| Command | What it does |
|---|---|
| `boundline goal` | Set the objective for the current session. |
| `boundline plan` | Generate a technical plan to achieve the goal. |
| `boundline run` | Execute the next pending step in the plan. |
| `boundline status` | Check the current session status and next actions. |
| `boundline inspect` | View detailed execution traces and evidence. |

## 📚 Deep Dive Documentation

- [Getting Started](docs/getting-started.md)
- [Configuration and Precedence](docs/configuration.md)
- [Architecture and Canon Boundaries](docs/architecture.md)
- [Project Scale Delivery Model](docs/delivery-model.md)
- [Assistant Command Packs](assistant/README.md)

## 🤝 Community And Support
- Bug reports & feature requests: `.github/ISSUE_TEMPLATE/`
- Vulnerability reporting: [SECURITY.md](SECURITY.md)
- Participation expectations: `.github/CODE_OF_CONDUCT.md`
- Contributor workflow: [CONTRIBUTING.md](CONTRIBUTING.md)