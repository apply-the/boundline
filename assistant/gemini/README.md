# Gemini CLI Command Notes

This folder documents Synod usage from Gemini CLI surfaces.

For now, Gemini support is CLI-only. Use the same Synod command workflow as
other assistants and treat Gemini as the explicit `gemini-cli` assistant
binding when `effective_routing` resolves to `gemini`:

```bash
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
different assistant family.
