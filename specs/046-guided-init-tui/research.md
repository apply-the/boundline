# Research: Guided Init TUI and Runtime Catalog

## Decision 1: Use a terminal interaction layer instead of manual `read_line` prompts

- Decision: Replace the current guided `init` prompt collection with a terminal interaction layer that supports select, multi-select, confirmation, defaults, cancellation, and editable text input. The initial implementation will use `dialoguer` in the CLI crate.
- Rationale: The current `stdin().read_line()` approach cannot handle navigation keys, multi-step correction, or bounded validation without leaking raw escape sequences. `dialoguer` provides the missing interaction model while keeping the command in a standard terminal flow rather than introducing a full-screen TUI.
- Alternatives considered:
  - Keep `read_line()` and improve parsing: rejected because it does not solve cursor navigation, selection UX, or stable validation loops.
  - Use a full-screen TUI stack: rejected because the slice only needs bounded bootstrap guidance, not a new runtime surface.
  - Use `inquire` instead of `dialoguer`: acceptable in principle, but `dialoguer` is the lighter initial fit for select, multi-select, confirm, and input prompts.

## Decision 2: Separate route review from route editing and keep editing slot-by-slot

- Decision: After assistant selection, show a route review table with proposed defaults and provide actions to accept defaults, edit one slot, or clear routes. Slot editing remains explicit and one slot at a time.
- Rationale: Asking for all routes in one line is the core UX failure. A review-first table keeps defaults visible and lets operators adjust only the slots they care about without retyping the whole routing surface.
- Alternatives considered:
  - Prompt for every slot unconditionally in a fixed sequence: rejected because it adds friction even when defaults are already correct.
  - Keep one-line `SLOT=RUNTIME:MODEL` entry in guided mode: rejected because it is brittle, hard to validate, and hard to recover from.

## Decision 3: Ship a bundled model catalog with explicit metadata and custom fallback

- Decision: Introduce a repository-managed bundled catalog asset at `assistant/catalog/model-catalog.toml` that is compiled into the CLI, exposes source metadata in guided init, and can propose default runtime/model routes for supported assistant surfaces. Custom model identifiers remain allowed but are labeled unverified.
- Rationale: There is no portable, trustworthy remote discovery surface across Copilot, Codex, Claude, and Gemini. A bundled catalog gives operators explicit known choices now without pretending to perform live discovery.
- Alternatives considered:
  - Remote provider discovery at init time: rejected because the providers do not expose a uniform portable surface and because network discovery would make bootstrap less reliable.
  - Hard-code all catalog data inline in Rust only: rejected because a repository-managed asset is easier to review and update while still being bundled into releases.
  - Store the catalog in workspace config only: rejected because it would make the first bootstrap circular and would drift across workspaces.

## Decision 4: Use a spinner for interactive long-running steps and stable text for non-interactive output

- Decision: Add `indicatif` spinner-based progress feedback for interactive init steps that exceed a bounded delay threshold and degrade to stable line-oriented progress messages when stdout is not a terminal.
- Rationale: Operators need to know when init is actively working rather than frozen, but automation logs must stay stable and copyable. A spinner is appropriate only when the command owns the current terminal line.
- Alternatives considered:
  - No progress feedback: rejected because it makes long-running init work appear hung.
  - Always emit spinner frames: rejected because redirected output and CI logs would capture unreadable control artifacts.
  - Add background progress workers: rejected because the slice must remain sequential and inspectable.

## Decision 5: Keep non-interactive init as a first-class mode rather than a side effect of missing TTY

- Decision: Add an explicit `--non-interactive` path for `boundline init` while retaining repeated `--assistant` and `--route` flags as the automation surface.
- Rationale: Automation must be intentional and testable. An explicit flag makes failure modes clearer when required values are missing and prevents silent prompt attempts in scripts.
- Alternatives considered:
  - Infer non-interactive mode only from TTY detection: rejected because scripts attached to pseudo-terminals or redirected subprocesses still need deterministic behavior.
  - Drop automation support in favor of guided mode only: rejected because CI and scripted repo bootstrap remain valid operator workflows.

## Decision 6: Keep assistant-pack scaffolding explicit and reuse the existing repository assets

- Decision: Continue scaffolding assistant packs from the existing repository-managed `assistant/` assets into the target workspace using the same relative paths, and report created, updated, or unchanged file counts grouped by assistant surface.
- Rationale: Boundline already ships curated assistant prompts and commands. The guided init redesign should improve how operators choose those surfaces, not invent a second assistant-pack format.
- Alternatives considered:
  - Generate assistant assets dynamically from templates during init: rejected because the repository already owns the curated content and generation would introduce unnecessary drift.
  - Hide assistant-pack file actions behind generic success text only: rejected because operators need explicit visibility into which surfaces were scaffolded or refreshed.
