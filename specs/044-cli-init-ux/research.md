# Research: Guided CLI UX And Clearer Messaging

## Decision 1: Make the CLI the source of truth for route discoverability

- Decision: Put supported assistant names, supported route slots, at least one
  valid `SLOT=RUNTIME:MODEL` example, and blank-input/default behavior directly
  in guided prompts, clap help, and post-init summaries.
- Rationale: The current failure mode is not missing capability; it is missing
  discoverability. Operators should not need to reverse-engineer route syntax
  from docs or source comments.
- Alternatives considered:
  - Leave discoverability in docs only: rejected because the user reaches the
    confusing prompt before reading docs.
  - Require explicit routes always: rejected because assistant defaults already
    exist and should remain the low-friction first-run path.

## Decision 2: Prefer assisted recovery over terse parser errors

- Decision: Translate malformed route, unknown slot, unknown assistant, missing
  capability, and overwrite conflicts into human-readable corrective messages
  that name the bad input and show the next valid action.
- Rationale: First-run CLI ergonomics depend on fast recovery, especially when
  the operator mistypes one value in a guided flow or one flag in a copied
  command.
- Alternatives considered:
  - Preserve the current validation strings verbatim: rejected because they are
    technically correct but do not answer the operator's actual question.
  - Add fuzzy recovery everywhere: rejected for now; bounded closest-match or
    example guidance is enough for the initial slice.

## Decision 3: Rich output must be semantic, capability-aware, and optional

- Decision: Introduce structured, semantically grouped output for init and
  doctor that can use lightweight color or headings only when stdout is a TTY,
  while keeping the exact same meaning in plain text for CI and redirection.
- Rationale: The Lucas F. Costa CLI UX guidance is useful here: better layout
  should reduce time-to-value, but Boundline still needs grep-friendly text and
  predictable automation behavior.
- Alternatives considered:
  - Full-screen TUI or ASCII-art-heavy output: rejected because it adds a new
    surface, increases maintenance cost, and risks breaking scriptability.
  - No richer formatting at all: rejected because the first-run output is dense
    and currently too easy to skim incorrectly.

## Decision 4: Keep assistant command-pack scaffolding tied to init summaries

- Decision: Since repository-local assistant assets are part of the init
  contract, treat assistant setup as a first-class section of the init preview
  and success output, including create/update/unchanged status.
- Rationale: The recent regression showed that assistant setup silently falling
  out of init breaks the chat-first product story. Surfacing it explicitly makes
  the contract inspectable.
- Alternatives considered:
  - Hide assistant scaffolding behind docs only: rejected because users expect
    the repo to become usable from chat immediately after init.
  - Always overwrite assistant assets: rejected because safe reruns are already
    part of the init contract.

## Decision 5: Ship the UX slice as a versioned release update

- Decision: Bump the product version from `0.43.0` to `0.44.0` and align docs,
  distribution metadata, and assistant guidance to the new init/doctor behavior.
- Rationale: This is user-visible behavior on the primary operator entrypoint
  and includes new output contracts, so the release metadata should move with
  the feature.
- Alternatives considered:
  - Leave the version unchanged: rejected because the repository already tracks
    slices by release-aligned feature increments.