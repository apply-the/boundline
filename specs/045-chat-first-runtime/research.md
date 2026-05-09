# Research: Chat-First Host-Integrated Runtime

## Decision 1: Reuse existing session and trace projections as the host contract

- Decision: Use the existing `SessionStatusView` and `TraceSummaryView` as the
  canonical structured payloads for host-facing command output.
- Rationale: These views already encode the bounded delivery state, continuity
  authority, follow-up guidance, and trace evidence that assistant hosts need.
  Reusing them avoids inventing a second runtime model that could drift from the
  CLI's actual control flow.
- Alternatives considered:
  - Create a new host-specific state model: rejected because it would duplicate
    existing observability surfaces and risk divergence.
  - Continue relying on line-oriented text parsing only: rejected because it is
    brittle for host-chat integrations and hides the real contract inside prose.

## Decision 2: Add structured output to existing commands instead of building a new chat runtime

- Decision: Add an opt-in structured output mode to the existing lifecycle and
  inspection commands rather than introducing a new standalone chat command or
  TUI surface.
- Rationale: The assistant command packs already map host intents onto the
  session-native CLI commands. Adding a structured contract there delivers host
  value immediately while keeping the delivery engine sequential and inspectable.
- Alternatives considered:
  - Build a new `chat` or `host` runtime surface first: rejected because it
    would front-load UI and routing complexity before proving the host contract.
  - Make structured output the default: rejected because existing operators and
    scripts already rely on the current human-readable output.

## Decision 3: Keep human-readable output as the default and treat structured output as additive

- Decision: Preserve the current text output contract for direct human use and
  make the structured host contract explicit and opt-in.
- Rationale: Boundline already has script-friendly and human-readable CLI
  surfaces. The new host contract should strengthen host integrations without
  breaking direct terminal use or forcing all operators into a machine-oriented
  format.
- Alternatives considered:
  - Replace the text contract entirely: rejected because it would be a larger
    behavioral break than the first slice needs.
  - Support only machine-readable output in assistant packs: rejected because
    chat-only fallback paths still need the plain-text surface when shell access
    is unavailable.

## Decision 4: Update assistant command packs to prefer the structured shell-enabled path

- Decision: Align the assistant command packs so shell-enabled paths prefer the
  structured output mode while chat-only fallback continues to use copyable
  plain-text commands and pasted output.
- Rationale: The command packs are the host-facing entrypoint already managed by
  this repository. Updating them keeps the runtime contract visible and reduces
  per-host prompt drift.
- Alternatives considered:
  - Leave command packs unchanged and rely on external host instructions:
    rejected because the repository owns the assistant integration contract.
  - Update only one host surface: rejected because the current assistant packs
    for Claude, Codex, and Copilot already share the same lifecycle mapping.