# Gemini CLI Command Notes

This folder documents Synod usage from Gemini CLI surfaces.

For now, Gemini support is CLI-only. Use the same Synod command workflow as
other assistants:

```bash
cargo run --bin synod -- init --workspace <workspace> --template bug-fix
cargo run --bin synod -- doctor --workspace <workspace>
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture --workspace <workspace> --goal "<goal>"
cargo run --bin synod -- plan --workspace <workspace>
cargo run --bin synod -- run --workspace <workspace>
```

Use `synod config show|set|unset` for routing changes rather than manual file
editing.
