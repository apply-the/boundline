# Getting Started with Boundline

This guide is the practical companion to the README. It assumes you want the
fastest credible path from installation to a bounded Boundline session.

## Quick Path Brutale

### 1. Install Boundline

Use the release-aligned path that matches your machine:

- macOS via Homebrew formula once the release bundle checksums are published:

```bash
brew install https://raw.githubusercontent.com/apply-the/boundline/v0.40.0/distribution/homebrew/Formula/boundline.rb
```

- Windows via winget after the release manifest is published:

```powershell
winget install ApplyThe.Boundline
```

- Source fallback when bundled channels are unavailable:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
```

For updates, stay on the same path:

- Homebrew: `brew upgrade boundline`
- winget: `winget upgrade ApplyThe.Boundline`
- Source fallback: `cargo install --path . --force`

If a bundled channel for the current release is not published yet, use the
source fallback temporarily and treat the bundled path as not ready rather than
guessing.

### 2. Verify The Install

Run install diagnostics before touching workspace state:

```bash
boundline doctor --install
```

Read the output literally:

- `boundline_version` is the running CLI version.
- `supported_canon_version` is the Canon compatibility target for this Boundline release.
- `channel_candidates` names the bundled or fallback paths that make sense on this machine.
- `companion_state` is `ready`, `already_satisfied`, `blocked`, or `repair_needed`.

If the install is not ready, follow the printed action exactly and rerun
`boundline doctor --install`.

### 3. Verify The Workspace

```bash
boundline doctor --workspace <workspace>
```

This checks that the repository exists, is writable, and has the local state
surfaces Boundline needs for traces and any optional execution profile bootstrap.

### 4. Start One Bounded Session

```bash
boundline start --workspace <workspace>
boundline capture --workspace <workspace> --goal "Fix the failing add test"
boundline plan --workspace <workspace>
boundline plan --workspace <workspace> --confirm
boundline run --workspace <workspace>
boundline status --workspace <workspace>
boundline inspect --workspace <workspace>
```

That is the main product path: `start -> capture -> plan -> confirm -> run ->
status -> inspect`.

Read planning output literally before you continue:

- `context_summary` is the bounded context story Boundline thinks it has.
- `context_credibility` tells you whether the current context is credible, stale, or insufficient.
- `context_primary_inputs` names the files or artifacts that actually shaped the plan.
- `context_provenance` explains why those inputs were selected.
- `context_staleness_reason` tells you why Boundline refused to guess.

If `plan` stops because the context is weak, add a brief that names the right
files, narrow the goal, or rerun after failing validation produces a better
evidence anchor.

## Optional Bootstrap

Use `init` only when you want Boundline to scaffold local compatibility/bootstrap
files, assistant setup, or domain-template defaults before the first session:

```bash
boundline init --workspace <workspace>
```

`--template` is optional. The built-in starting templates are `bug-fix`,
`change`, and `delivery`. They seed the generated compatibility execution
profile only; they do not replace the session-native workflow.

If you need domain defaults at bootstrap time, seed them explicitly:

```bash
boundline init \
  --workspace <workspace> \
  --domain systems \
  --domain react \
  --domain-standard "react=follow the shared UI system" \
  --context-binding "react|design-system|mcp:design-system"
```

## Optional Routing Setup

Use `config` when routing ownership, assistant bindings, or effective policy
matter before the run:

```bash
boundline config set --scope global --slot planning --runtime codex --model gpt-5-codex
boundline config set --workspace <workspace> --scope workspace --reviewer safety --runtime copilot --model gpt-5.4
boundline config show --workspace <workspace> --scope effective
```

The effective view is the authoritative read-side surface for slot routing,
assistant bindings, runtime capability policy, and slot effort policy.

## Optional Named Workflow Layer

If the workspace defines `.boundline/workflows.toml`, you can use a named entrypoint
without leaving the same session-owned runtime:

```bash
boundline workflow list --workspace <workspace>
boundline workflow run governed-delivery --workspace <workspace> --goal "Fix the failing add test"
boundline workflow status --workspace <workspace>
boundline workflow resume --workspace <workspace>
boundline workflow inspect --workspace <workspace>
```

The workflow layer is intentionally thin. It reuses the same capture, plan,
run, review, govern, and inspect surfaces instead of creating a second runtime.

## Optional Cluster Entry

When one bounded change spans more than one registered workspace, enter through
the primary workspace and keep that session authoritative:

```bash
boundline cluster init \
  --workspace <primary-workspace> \
  --cluster-id delivery-a \
  --member <primary-workspace> \
  --member <secondary-workspace>

boundline start --cluster <primary-workspace>
boundline capture --cluster <primary-workspace> --goal "Fix the failing add test"
boundline plan --cluster <primary-workspace>
boundline plan --cluster <primary-workspace> --confirm
boundline run --cluster <primary-workspace>
boundline status --cluster <primary-workspace>
```

## When Canon Matters

Canon is not the product entrypoint. Boundline still owns orchestration, session
state, planning, execution, and validation. Canon matters only when you enable
governed routes, governed approvals, or governed artifact capture.

The current Boundline release documents Canon `0.39.0` as the supported CLI target
for the machine-facing `canon governance start|refresh|capabilities --json`
`v1` adapter surface. Install diagnostics keep that boundary explicit after
install or upgrade.

## When To Read More

If you need the deeper model rather than the first-run path, continue with:

- [docs/architecture.md](architecture.md) for the Boundline-versus-Canon boundary, routing model, compatibility path, workflows, clusters, and governance role
- [assistant/README.md](../assistant/README.md) for assistant command packs that follow the same quick-path-first product story

| Command | What it is for |
| --- | --- |
| `boundline init` | Bootstrap optional compatibility `.boundline` workspace files and assistant setup |
| `boundline config show|set|unset` | Inspect or edit routing defaults at global/workspace scope |
| `boundline cluster init|status|inspect` | Register a bounded multi-workspace cluster and inspect member state |
| `boundline doctor` | Verify the installed Boundline plus Canon pairing or validate a workspace before running |
| `boundline start` | Initialize or reset the active workspace session |
| `boundline capture` | Store the delivery goal plus negotiated packet in session state |
| `boundline flow` | Select `bug-fix`, `change`, or `delivery` |
| `boundline plan` | Build one evidence-driven goal-plan proposal from the active session when the negotiated packet is credible and the assembled context pack is bounded enough to support planning |
| `boundline step` | Execute one step of the current task |
| `boundline run` | Execute the current task until completion or operator intervention, or bootstrap the native route directly from `--goal`; add `--compatibility` for manifest-backed execution |
| `boundline status` | Show the current session snapshot, including negotiated follow-up cues and context-pack credibility |
| `boundline next` | Show the CLI-reported next action from the active negotiated boundary and current context state |
| `boundline inspect` | Summarize the latest trace or a specific trace, including negotiated delivery and context-pack cues |
| `boundline workflow list|run|status|resume|inspect` | Discover and reuse the same session-native route through a named workflow entrypoint |

## Choosing the Right Manifest Shape

Boundline keeps declarative manifests as an explicit compatibility surface; `boundline init`
scaffolds that policy when you intentionally want manifest-backed behavior.

- use `attempts` when you want explicit authored change attempts
- use `adaptive` when you want Boundline to choose one bounded workspace slice and
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