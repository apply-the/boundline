# Antigravity Command Pack

This folder contains the repo-local Boundline command pack for Antigravity chat
surfaces.

Support mode: `repo-local-full`.

Antigravity bootstraps its repo-local package surface through
`boundline init --assistant antigravity` and refreshes that surface through
`boundline update --workspace <workspace> --target assistant --apply`, while
the CLI remains authoritative
for status, inspect, explain-plan, and all delight follow-through lines.

Global bootstrap remains a separate manual-fallback path under
`assistant/global/antigravity/`. Compatibility remains an explicit subordinate route.

Named workflows remain available through the workflow CLI when a workspace
authors `.boundline/workflows.toml`, but this packaged assistant surface does
not ship dedicated `/boundline-workflow-*` commands:

```bash
boundline doctor --install
boundline workflow list --workspace <workspace>
boundline workflow run <name> --workspace <workspace>
boundline workflow status --workspace <workspace>
boundline workflow resume --workspace <workspace>
boundline workflow inspect --workspace <workspace>
boundline orchestrate --workspace <workspace> --goal "<goal>" --until phase-request --json-stream
boundline plan --workspace <workspace> --json
boundline run --workspace <workspace> --json
boundline status --workspace <workspace> --json
boundline next --workspace <workspace> --json
boundline inspect --workspace <workspace> --json
```

Use `boundline config show|set|unset` for routing changes rather than manual
file editing. Treat workflow list, run, status, resume, and inspect as
CLI-only named-workflow helpers; the packaged assistant surface remains the
core session-native commands above. Compatibility remains an explicit
subordinate route when the operator intentionally chooses it.

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