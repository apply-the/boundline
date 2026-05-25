# Antigravity Command Pack

This folder contains the repo-local Boundline command pack for Antigravity chat
surfaces.

Support mode: `repo-local-full`.

Antigravity now scaffolds a repo-local package surface through
`boundline init --assistant antigravity`, while the CLI remains authoritative
for status, inspect, explain-plan, and all delight follow-through lines.

Global bootstrap remains a separate manual-fallback path under
`assistant/global/antigravity/`. Compatibility remains an explicit subordinate route.

Use the same primary Boundline workflow surface as the other packaged
assistants:

```bash
cargo run --bin boundline -- doctor --install
cargo run --bin boundline -- workflow list --workspace <workspace>
cargo run --bin boundline -- workflow run <name> --workspace <workspace>
cargo run --bin boundline -- workflow status --workspace <workspace>
cargo run --bin boundline -- workflow resume --workspace <workspace>
cargo run --bin boundline -- workflow inspect --workspace <workspace>
cargo run --bin boundline -- orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream
cargo run --bin boundline -- plan --workspace <workspace> --json
cargo run --bin boundline -- run --workspace <workspace> --json
cargo run --bin boundline -- status --workspace <workspace> --json
cargo run --bin boundline -- next --workspace <workspace> --json
cargo run --bin boundline -- inspect --workspace <workspace> --json
```

Use `boundline config show|set|unset` for routing changes rather than manual
file editing. Treat workflow list, run, status, resume, and inspect as the
primary Boundline workflow surface in this release; compatibility remains an
explicit subordinate route when the operator intentionally chooses it.

When native planning or follow-through surfaces `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, or
`context_staleness_reason`, preserve those values exactly instead of
paraphrasing them away. Do the same for `goal_plan_state`,
`goal_plan_revision`, `planning_rationale`, and `verification_strategy`, plus
`delegation_mode`, packet identity, target owner, headline, and evidence
summary when delegated continuity becomes authoritative.

When `latest_checkpoint_id`, `latest_checkpoint_scope`, or
`latest_checkpoint_restore_command` appear, preserve them exactly and treat the
reported restore command as the authoritative rewind path for failed or blocked
mutating work.