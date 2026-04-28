# Getting Started with Synod

This guide is the practical version of the README: what Synod does, how to
install it, and how to use it in a workspace.

## What Synod Is

Synod is a local CLI for bounded software-delivery work.

You use Synod to:

- read an execution manifest from `<workspace>/.synod/execution.json`
- capture human-authored goals and Markdown briefs without authoring a task JSON request
- apply only the changes declared in that manifest
- run the workspace validation command after each attempt
- keep session state in `<workspace>/.synod/session.json`
- keep traces in `<workspace>/.synod/traces/`

Today, human-friendly input stops at goal and brief capture. Synod still does
not provide a `synod init` workflow or interactive setup that writes the
workspace execution manifest for you.

If review, adaptive execution, or governance are configured, Synod projects that
state through the same CLI instead of introducing a separate runtime surface.

The shipped CLI binary is `synod`.

If you enable Synod governance through Canon, the current Synod adapter is
validated against Canon `0.20.0`.

## Install Synod

Synod currently targets Rust `1.95.0`.

Run from source:

```bash
git clone https://github.com/apply-the/synod.git
cd synod
cargo run --bin synod -- --help
```

Or install the binary locally:

```bash
cargo install --path .
synod --help
```

## First Run in a Workspace

### 1. Prepare the Workspace Manifest

Create a workspace with a `.synod/execution.json` file.

This is still a manual step today. The roadmap now tracks a future `synod init`
flow, but the current CLI expects the execution manifest or the legacy fixture
to exist before `doctor`, `plan`, or `run` can proceed.

Minimal example:

```json
{
  "name": "red-to-green-execution",
  "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
  "validation_command": {
    "program": "cargo",
    "args": ["test", "--quiet"]
  },
  "attempts": [
    {
      "attempt_id": "fix-add",
      "summary": "Replace subtraction with addition",
      "failure_mode": "replan",
      "changes": [
        {
          "path": "src/lib.rs",
          "find": "left - right",
          "replace": "left + right"
        }
      ]
    }
  ]
}
```

Synod prefers this manifest and still supports the legacy
`<workspace>/.synod/fixture.json` format for older workspaces.

### 2. Check the Workspace

Before starting a session, validate the target workspace:

```bash
synod doctor --workspace <workspace>
```

### 3. Start the Session and Capture the Goal

Use the session workflow when you want the full operator loop:

```bash
synod start --workspace <workspace>
synod capture --workspace <workspace> --goal "Fix the failing add test"
synod flow bug-fix --workspace <workspace>
```

Since `0.10.0`, `synod capture` (and `synod run`) also accept one or more
`--brief <path>.md` arguments alongside or instead of `--goal`. Each brief
must be a Markdown file (`.md` or `.markdown`) inside the workspace; their
contents are concatenated under stable provenance headers and projected
through the existing capture pipeline:

```bash
synod capture --workspace <workspace> \
  --goal "Fix the failing add test" \
  --brief docs/context.md
```

`synod flow` is optional. Use it when you want to pin the run to one of the
built-in flows: `bug-fix`, `change`, or `delivery`.

### 4. Plan and Run

```bash
synod plan --workspace <workspace>
synod run --workspace <workspace>
```

### 5. Inspect the Result

After the run, use the read-side commands to understand what happened:

```bash
synod status --workspace <workspace>
synod next --workspace <workspace>
synod inspect --workspace <workspace>
```

These commands tell you:

- what Synod is currently doing in the workspace
- what trace was produced
- whether the run succeeded, failed, blocked, or needs follow-up
- what the next CLI action should be

## Direct Run Without the Full Session Flow

Use the direct workflow when you only want to launch one bounded run:

```bash
synod run --workspace <workspace> --goal "Fix the failing add test"
```

Direct run still uses the workspace execution manifest as the bounded execution
contract; it only skips the explicit session setup.

## The Core Commands

| Command | What it is for |
| --- | --- |
| `synod doctor` | Validate the workspace and manifest before running |
| `synod start` | Initialize or reset the active workspace session |
| `synod capture` | Store the delivery goal in session state |
| `synod flow` | Select `bug-fix`, `change`, or `delivery` |
| `synod plan` | Build the next bounded task from the active session |
| `synod step` | Execute one step of the current task |
| `synod run` | Execute the current task until completion or operator intervention |
| `synod status` | Show the current session snapshot |
| `synod next` | Show the CLI-reported next action |
| `synod inspect` | Summarize the latest trace or a specific trace |

## Choosing the Right Manifest Shape

Synod is intentionally bounded by the workspace manifest until a future init
flow can scaffold that policy for you.

- use `attempts` when you want explicit authored change attempts
- use `adaptive` when you want Synod to choose one bounded workspace slice and
  generate deterministic repair candidates
- add `review` when the run must pass through reviewer findings and vote
  resolution
- add governance configuration when specific stages must route through Canon

## Next Reading

- [README.md](../README.md) for the short product overview
- [docs/adaptive-execution.md](adaptive-execution.md) for adaptive slicing and replanning
- [docs/review-voting.md](review-voting.md) for multi-reviewer councils
- [assistant/README.md](../assistant/README.md) for assistant command packs