# Quickstart

Want to see it working in 5 minutes? This is the fastest route to a bounded session, without any theory. 

If you prefer a structured learning path, read the [Getting Started](./getting-started) guide instead.

## 1. Install Boundline

```bash
# macOS
brew tap apply-the/boundline
brew install boundline
```
*(For Linux, Windows, or source installs, see [Installation](./installation).)*

## 2. Initialize a Workspace

Navigate to an existing Git repository and bootstrap Boundline:

```bash
cd my-project
boundline init --assistant codex
```

## 3. Start a Goal

Define the objective for this session:

```bash
boundline goal --goal "Fix the failing add test"
```

## 4. Draft the Plan

Ask Boundline to analyze the repository and draft a step-by-step plan:

```bash
boundline plan
```

## 5. Execute the Next Step

Execute the first approved action from the plan:

```bash
boundline run
```

## 6. Inspect Status

Verify the current state of the session and see what happens next:

```bash
boundline status
boundline inspect
```

**That's it!** You can continue running `boundline run` until the goal is completed, or jump into [Getting Started](./getting-started) to understand what actually just happened under the hood.
