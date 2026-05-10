# Gemini CLI Command Notes

This folder documents Boundline usage from Gemini CLI surfaces.

Gemini remains CLI-first in `0.43.0`, but it follows the same primary Boundline
workflow surface used by the other assistants.

compatibility remains an explicit subordinate route.

For now, Gemini support is CLI-only. Use the same Boundline command workflow as
other assistants and treat Gemini as the explicit `gemini-cli` assistant
binding when `effective_routing` resolves to `gemini`:

```bash
cargo run --bin boundline -- workflow list --workspace <workspace>
cargo run --bin boundline -- workflow run <name> --workspace <workspace>
cargo run --bin boundline -- workflow status --workspace <workspace>
cargo run --bin boundline -- workflow resume --workspace <workspace>
cargo run --bin boundline -- workflow inspect --workspace <workspace>
cargo run --bin boundline -- init --workspace <workspace> --canon-mode-selection auto-confirm --assistant gemini --route planning=gemini:<model>
cargo run --bin boundline -- doctor --install
cargo run --bin boundline -- config show --workspace <workspace> --scope workspace
cargo run --bin boundline -- config set-canon --workspace <workspace> --mode-selection auto-confirm
cargo run --bin boundline -- start --workspace <workspace> --json
cargo run --bin boundline -- capture --workspace <workspace> --goal "<goal>" --json
cargo run --bin boundline -- plan --workspace <workspace> --json
cargo run --bin boundline -- plan --workspace <workspace> --confirm --json
cargo run --bin boundline -- run --workspace <workspace> --json
cargo run --bin boundline -- status --workspace <workspace> --json
cargo run --bin boundline -- next --workspace <workspace> --json
cargo run --bin boundline -- inspect --workspace <workspace> --json
```

Canon-default mode shorthand uses `boundline run --mode <mode>`. For example,
`/boundline-run requirements` maps to `boundline run --mode requirements`, and
mode aliases such as `/boundline-requirements` use the same mapping. Preserve
`governance_runtime`, `mode_selection_preference`, `selected_mode`,
`approval_state`, and `next_action` from CLI output.

Gemini command equivalents for the repository-managed chat surface are:
`/boundline-init`, `/boundline-doctor`, `/boundline-config-show`,
`/boundline-config-set-canon`, `/boundline-capture`, `/boundline-run`, and all
mode aliases `/boundline-<mode>`. Use the same CLI mapping as the Claude, Codex,
and Copilot packs.

Use `boundline config show|set|unset` for routing changes rather than manual file
editing. If a workspace declares `assistant_runtimes` and the active
implementation or verification route selects `gemini` without that capability,
native execution now records an explicit delegation packet instead of silently
falling back to a different assistant family. Treat workflow list, run, status, resume, and
inspect as the primary Boundline workflow surface in this release; compatibility
remains an explicit subordinate route when the operator intentionally chooses
it. When native planning or follow-through surfaces `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, or
`context_staleness_reason`, preserve those values exactly instead of
paraphrasing them away. Do the same for `goal_plan_state`,
`goal_plan_revision`, `planning_rationale`, and `verification_strategy`, plus
`delegation_mode`, packet identity, target owner, headline, and evidence
summary when delegated continuity becomes authoritative, plus
selector-driven `latest_selection_headline`,
`latest_selection_reason`, and inspect `selector:` lines when they appear.
When Canon-grounded memory is surfaced through those context fields or through
`governance_next_action`, preserve the governed artifact refs, credibility, and
stale-memory wording exactly: those fields can be the authoritative stop
condition for the next bounded action.
When `latest_checkpoint_id`, `latest_checkpoint_scope`, or
`latest_checkpoint_restore_command` appear, preserve them exactly and treat the
reported restore command as the authoritative rewind path for failed or blocked
mutating work.
