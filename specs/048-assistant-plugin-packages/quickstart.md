# Quickstart: Assistant Plugin Packages

## 1. Verify the version bump

```bash
rg -n '0\.49\.0' Cargo.toml CHANGELOG.md ROADMAP.md distribution assistant
```

Expected:

- Workspace version is `0.49.0`.
- Distribution metadata, plugin metadata, catalog metadata, changelog, and roadmap mention the new feature version where they carry release-specific values.

## 2. Inspect package folders

```bash
find .claude-plugin .codex-plugin .cursor-plugin .copilot-prompts assistant/commands assistant/prompts assistant/assets -maxdepth 2 -type f | sort
```

Expected:

- Claude Code, Codex, Cursor, and Copilot prompt-pack package files exist.
- Shared metadata, command definitions, starter prompts, Copilot prompt pack, icon, and logo exist.

## 3. Run package validation

```bash
bash scripts/validate-assistant-plugins.sh
```

Expected:

- The command runs the focused Rust test target.
- JSON, required fields, paths, command coverage, version alignment, and prohibited wording checks pass.

## 4. Check chat-to-runtime mapping

```bash
rg -n '/boundline:(start|capture|plan|run|status|inspect|recover|govern)|session.json|next_command' .claude-plugin .codex-plugin .cursor-plugin .copilot-prompts assistant docs/guides/assistant-plugin-packages.md README.md
```

Expected:

- Required commands are discoverable.
- `.boundline/session.json` is described as authoritative.
- Non-success states and CLI-reported next actions are represented in package guidance.

## 5. Final verification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

Expected:

- Formatting, clippy, and tests pass.
- Touched Rust files created or modified by this slice have at least 95% line coverage.
