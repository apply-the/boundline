# Getting Started with Synod

This guide is the practical version of the README: what Synod does, how to
install it, and how to use it in a workspace.

## What Synod Is

Synod is a local CLI for bounded software-delivery work.

You use Synod to:

- run the primary session-native path: `start -> capture -> plan -> run -> status -> next -> inspect`
- use `synod init` only when you want scaffolded compatibility defaults or assistant setup
- capture human-authored goals and Markdown briefs without authoring a task JSON request
- inspect or tune runtime/model routing with `synod config`
- execute bounded actions from live state and recorded evidence
- use declarative execution profiles only when you intentionally want compatibility behavior
- keep session state in `<workspace>/.synod/session.json`
- keep traces in `<workspace>/.synod/traces/`
- rely on `run`, `status`, `next`, and `inspect` to tell the same routing and `execution_condition` story

For most users the path is simple: run `synod doctor`, `synod start`, `synod capture`,
`synod plan`, and `synod run`. `synod init` is optional bootstrap for generated
compatibility profiles and assistant setup.

If review, adaptive execution, or governance are configured, Synod projects that
state through the same CLI instead of introducing a separate runtime surface.

The shipped CLI binary is `synod`.

If you enable Synod governance through Canon, the current Synod adapter is
validated against Canon `0.25.0`.

In `0.26.0`, `capture` persists one negotiated delivery packet before
planning, `plan` blocks on non-credible negotiation states, and `run`,
`status`, `next`, and `inspect` surface the same acceptance-boundary story.
Governed `bug-fix:investigate` and later verify-stage Canon
`security-assessment` can remain on the same native session route, and the
optional `synod workflow` surface projects the same approval, blocked, and
packet-lineage story without introducing a second operator workflow. The same
session-native commands can also run against `--cluster <primary-workspace>` so
one authoritative session can plan and deliver a bounded change across
registered member repositories while `run`, `status`, `next`, and `inspect`
surface `route_owner`, `route_config_projection`, `cluster_route_owner`,
`cluster_authoritative_workspace`, `cluster_execution_condition`,
participating workspaces, and any blocking workspace explicitly. On the
explicit compatibility path, adaptive execution can still choose broader
bounded repair families, surface explicit `candidate_family` plus credibility
and rejection guidance, and stop cleanly when validation evidence is too weak
to justify another materially different bounded attempt. `status` plus `next`
also remain usable by naming the latest compatibility trace as the
authoritative inspect-only follow-up when no active session exists.

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

### 1. Optional Bootstrap

Run init when you want scaffolded compatibility defaults:

```bash
synod init --workspace <workspace>
```

`--template` is optional. If you omit it, Synod uses `bug-fix`.
Available starting templates are:

- `bug-fix`: start from a small targeted repair
- `change`: start from a bounded implementation change
- `delivery`: start from a broader delivery update

Templates only seed the generated compatibility execution profile. They do not lock the
workspace, and they do not replace `synod flow`.

If you want a different starting point later, regenerate it explicitly:

```bash
synod init --workspace <workspace> --force --template change
```

If you simply need another task of the same kind, do not rerun init. Start a
new session and run the workflow again.

If you need finer control than the generated starting point for the explicit
compatibility path, edit `<workspace>/.synod/execution.json` directly. The file shape is:

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

### 2. Check the Workspace

Before starting a session, validate the target workspace:

```bash
synod doctor --workspace <workspace>
```

Optional routing setup:

```bash
synod config set --scope global --slot planning --runtime codex --model gpt-5-codex
synod config set --workspace <workspace> --scope workspace --reviewer safety --runtime copilot --model gpt-5.4
synod config show --workspace <workspace> --scope effective
```

Optional clustered setup:

```bash
synod cluster init \
  --workspace <primary-workspace> \
  --cluster-id delivery-a \
  --member <primary-workspace> \
  --member <secondary-workspace>

synod cluster status --workspace <primary-workspace>
synod cluster inspect --workspace <primary-workspace>
synod config set --cluster <primary-workspace> --scope cluster --slot planning --runtime codex --model gpt-5-codex
synod config show --workspace <secondary-workspace> --cluster <primary-workspace> --scope effective
```

The primary workspace keeps the authoritative session at
`<primary-workspace>/.synod/session.json`. Member workspaces can still persist
their own terminal traces under `<member-workspace>/.synod/traces/` during a
clustered handoff.

### 3. Start the Session and Capture the Goal

Use the session workflow when you want the full operator loop:

```bash
synod start --workspace <workspace>
synod capture --workspace <workspace> --goal "Fix the failing add test"
synod flow bug-fix --workspace <workspace>
```

When the same bounded change spans a registered cluster, use the primary
workspace as the session-native entrypoint instead:

```bash
synod start --cluster <primary-workspace>
synod capture --cluster <primary-workspace> --goal "Fix the failing add test"
synod flow bug-fix --cluster <primary-workspace>
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

The current release persists `negotiation_goal_summary`,
`negotiation_resolution`, and `negotiation_acceptance_boundary` during
capture. If negotiation is still `pending_clarification`, `conflicting`, or
otherwise blocked, `synod plan` stops early and the CLI keeps the follow-up
story explicit instead of silently inventing a plan.

`synod flow` is optional. Use it when you want to pin the run to one of the
built-in flows: `bug-fix`, `change`, or `delivery`. This is separate from the
init template: `init` bootstraps an optional compatibility profile, while `flow`
selects the shape of the current session-native run.

### 4. Plan and Run

```bash
synod plan --workspace <workspace>
synod run --workspace <workspace>
```

Or continue the same bounded cluster-owned session:

```bash
synod plan --cluster <primary-workspace>
synod run --cluster <primary-workspace>
```

`synod plan` now succeeds only when the negotiated delivery packet is
credible. The resulting `GoalPlan`, terminal `run` output, and later
`status`, `next`, and `inspect` views keep the same negotiation summary and
acceptance-boundary wording on both native and explicit compatibility routes.

### 4a. Optional Named Workflow Layer

If the workspace defines `.synod/workflows.toml`, you can discover the
available named workflows and enter the same session-owned route through one of
them:

```bash
synod workflow list --workspace <workspace>
synod workflow run governed-delivery --workspace <workspace> --goal "Fix the failing add test"
synod workflow status --workspace <workspace>
synod workflow resume --workspace <workspace>
synod workflow inspect --workspace <workspace>
```

This layer is bounded by the same Synod phases and persists progress in the
active session. It does not replace `start`, `capture`, `plan`, `run`,
`status`, or `inspect`; it composes over them. `workflow list` provides the
named workflow summary, phase chain, and invocation guidance, while
`workflow run|resume` can now stop at actionable `review` or `govern` follow-through
states instead of treating those phases as static blockers.

### 5. Inspect the Result

After the run, use the read-side commands to understand what happened:

```bash
synod status --workspace <workspace>
synod next --workspace <workspace>
synod inspect --workspace <workspace>
```

For clustered work, read the same surfaces through the primary workspace:

```bash
synod status --cluster <primary-workspace>
synod next --cluster <primary-workspace>
synod inspect --cluster <primary-workspace>
```

These commands tell you:

- which route is active and why
- which `negotiation_goal_summary`, `negotiation_resolution`, and `negotiation_acceptance_boundary` currently bound the work
- which `route_owner` currently controls the follow-up story
- which clustered workspace remains authoritative when the run spans a registered cluster
- whether the authoritative follow-up state comes from the active session or the latest compatibility trace
- which material route or config inputs affected the current interpretation
- what `execution_condition` currently applies
- which `cluster_execution_condition` currently applies, which workspaces participated, and whether a member is blocking continuation
- what trace was produced
- whether the run succeeded, failed, blocked, or needs follow-up
- what the next CLI action should be

## Direct Run Without the Full Session Flow

Use the direct workflow only when you intentionally want the explicit
compatibility path instead of the session-native operator loop:

```bash
synod run --workspace <workspace> --goal "Fix the failing add test"
```

Direct run still uses the workspace execution manifest as the bounded execution
contract; it does not replace the normal session-native path.

If that manifest defines `adaptive`, failed validation can reprioritize the next
bounded adaptive attempt from the latest validation record while keeping the
route explicit as compatibility execution. In `0.26.0`, the same path can also
choose bounded ordering-boundary, result-status, and numeric-literal repairs,
and it reports explicit exhaustion instead of continuing blindly when the
validation evidence is absent or insufficient.

After a direct compatibility run, `synod status --workspace <workspace>` and
`synod next --workspace <workspace>` can now point you back to
`synod inspect --workspace <workspace>` even when there is no resumable active
session. Look for `continuity_authority: compatibility_trace` and the CLI-
reported inspect command instead of assuming you must restart from `synod
start`. In `0.26.0`, the same outputs also surface `route_owner: compatibility`
and any relevant `route_config_projection` so the inspect-only route stays
explicit even when summary wording otherwise matches the native path. They also
preserve the negotiated delivery summary so compatibility follow-up does not
lose the active acceptance boundary.

## The Core Commands

| Command | What it is for |
| --- | --- |
| `synod init` | Bootstrap optional compatibility `.synod` workspace files and assistant setup |
| `synod config show|set|unset` | Inspect or edit routing defaults at global/workspace scope |
| `synod cluster init|status|inspect` | Register a bounded multi-workspace cluster and inspect member state |
| `synod doctor` | Validate the workspace and any configured compatibility manifest before running |
| `synod start` | Initialize or reset the active workspace session |
| `synod capture` | Store the delivery goal plus negotiated packet in session state |
| `synod flow` | Select `bug-fix`, `change`, or `delivery` |
| `synod plan` | Build the next bounded task from the active session when the negotiated packet is credible |
| `synod step` | Execute one step of the current task |
| `synod run` | Execute the current task until completion or operator intervention, preferring session-native routing when a `GoalPlan` exists |
| `synod status` | Show the current session snapshot, including negotiated follow-up cues |
| `synod next` | Show the CLI-reported next action from the active negotiated boundary |
| `synod inspect` | Summarize the latest trace or a specific trace, including negotiated delivery cues |
| `synod workflow list|run|status|resume|inspect` | Discover and reuse the same session-native route through a named workflow entrypoint |

## Choosing the Right Manifest Shape

Synod keeps declarative manifests as an explicit compatibility surface; `synod init`
scaffolds that policy when you intentionally want manifest-backed behavior.

- use `attempts` when you want explicit authored change attempts
- use `adaptive` when you want Synod to choose one bounded workspace slice and
  generate deterministic repair candidates
- add `review` when the run must pass through reviewer findings and vote
  resolution
- add governance configuration when specific stages must route through Canon

## Next Reading

- [README.md](../README.md) for the short product overview
- [docs/configuration.md](configuration.md) for init/config precedence and routing details
- [docs/adaptive-execution.md](adaptive-execution.md) for adaptive slicing and replanning
- [docs/review-voting.md](review-voting.md) for multi-reviewer councils
- [assistant/README.md](../assistant/README.md) for assistant command packs