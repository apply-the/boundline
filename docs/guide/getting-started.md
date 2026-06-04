# Getting Started

This guide walks you through the mental model and your first serious workflow with Boundline. 

If you prefer a recipe to just get something running in 5 minutes without any theory, head over to the [Quickstart](./Quickstart).

## What Boundline Does

Boundline transforms non-deterministic AI chat loops into predictable, traceable, and governable software delivery processes.

Instead of keeping track of "what the AI just did" in an ephemeral chat window, Boundline forces an explicit workflow backed by local state. Every objective is tracked as a session, every step is verified against a drafted plan, and every outcome is durably recorded in local traces.

## Before You Start

Boundline assumes you are operating inside a Git repository. It leverages your existing codebase context, so you get the best results when running it inside an initialized project rather than an empty folder.

## Install

First, you need the Boundline CLI. Refer to the [Installation](./Installation) page for instructions on setting up the official packages for Linux, macOS, or Windows.

Once installed, verify it's working:
```bash
boundline doctor --install
```

## Initialize Your First Workspace

Boundline keeps state local to your repository. Before you can start a session, you must initialize the workspace:

```bash
cd your-repository
boundline init --assistant codex
```

This creates the default directories (`docs/project/`, `docs/evidence/`), writes a workspace-level configuration, and scaffolds any requested assistant plugins.

*For a deep dive on how to perfectly tune a repository for Boundline, read [First Workspace](./first-workspace).*

## Understand goal, plan, run, status

Boundline breaks work into explicit, discrete stages:

1. **`goal`**: Records the objective for the current session. What are we trying to achieve?
2. **`plan`**: Drafts bounded work. Boundline analyzes the codebase and outputs a concrete, step-by-step plan.
3. **`run`**: Executes the next approved step in the plan.
4. **`status`**: Inspects the current state of the session and tells you what happens next.

## Your First Workflow

Let's execute a real session end-to-end.

1. **Start the session**:
   ```bash
   boundline goal --goal "Refactor the authentication logic into a new module"
   ```
2. **Generate the plan**:
   ```bash
   boundline plan
   ```
   *Boundline will scan the repository and build an execution plan. If the plan
   lacks credibility, if the Canon backlog packet is closure-limited, or if the
   full backlog packet still lacks execution-handoff evidence, Boundline will
   stop and prompt you with a `phase_request`.*
3. **Execute the work**:
   ```bash
   boundline run
   ```
   *This executes the first step. Run it again for subsequent steps.*
4. **Verify the outcome**:
   ```bash
   boundline status
   boundline inspect
   ```
   *`status` tells you if the goal is complete, while `inspect` provides the trace-backed explanation of everything that occurred.*

## Common Next Steps

Once you are comfortable with the basic lifecycle, explore these advanced topics:

- Learn how to interact with Boundline via chat interfaces using [Assistant Command Packs](../../assistant/README).
- Explore [Common Workflows](./common-workflows) to handle complex tasks, large refactors, or debugging sessions.
- Read about [Core Concepts](./core-concepts) to understand how Boundline integrates with Canon governance rules.

## Troubleshooting

If you encounter issues during installation, initialization, or session execution:
- Run `boundline doctor` to verify system health.
- Run `boundline probe` to check workspace readiness.
- See the [Troubleshooting](../adapters/troubleshooting) guide for recovery paths.
