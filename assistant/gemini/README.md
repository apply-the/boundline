# Gemini CLI Command Notes

This folder documents Synod usage from Gemini CLI surfaces.

Gemini remains CLI-first in `0.33.0`, but it follows the same primary Synod
workflow surface used by the other assistants.

compatibility remains an explicit subordinate route.

For now, Gemini support is CLI-only. Use the same Synod command workflow as
other assistants and treat Gemini as the explicit `gemini-cli` assistant
binding when `effective_routing` resolves to `gemini`:

```bash
cargo run --bin synod -- workflow list --workspace <workspace>
cargo run --bin synod -- workflow run <name> --workspace <workspace>
cargo run --bin synod -- workflow status --workspace <workspace>
cargo run --bin synod -- workflow resume --workspace <workspace>
cargo run --bin synod -- workflow inspect --workspace <workspace>
cargo run --bin synod -- init --workspace <workspace> --template bug-fix
cargo run --bin synod -- doctor --workspace <workspace>
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"
cargo run --bin synod -- plan --workspace <workspace>
cargo run --bin synod -- run --workspace <workspace>
```

Use `synod config show|set|unset` for routing changes rather than manual file
editing. If a workspace declares `assistant_runtimes` and the active
implementation or verification route selects `gemini` without that capability,
native execution now fails explicitly instead of silently falling back to a
different assistant family. Treat workflow list, run, status, resume, and
inspect as the primary Synod workflow surface in this release; compatibility
remains an explicit subordinate route when the operator intentionally chooses
it. When native planning or follow-through surfaces `context_summary`,
`context_credibility`, `context_primary_inputs`, `context_provenance`, or
`context_staleness_reason`, preserve those values exactly instead of
paraphrasing them away.
