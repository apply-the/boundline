# Quickstart: S7.1 Assistant Delight Follow-Through

## Goal

Validate that the follow-through extends the shipped delight layer without
changing the primary session-native workflow or introducing a new Canon
dependency.

## Prerequisites

- A checkout on branch `063-assistant-delight-followthrough`
- A workspace that can produce an active Boundline session and latest trace
- At least one representative scenario with review activity and, ideally, an
  active reasoning profile
- The shared scenario notes in
  `tests/fixtures/063-assistant-delight-followthrough/scenarios.json`

## 1. Produce authoritative session and trace state

Use the existing session-native flow to create or refresh the active session and
latest trace for a representative bounded task:

```bash
cargo run --bin boundline -- run --workspace <workspace> --goal "Explain or repair a bounded change"
cargo run --bin boundline -- inspect --workspace <workspace> --json
```

Expected outcome:

- the workspace has an active `.boundline/session.json`
- `inspect` can load the latest authoritative trace
- the returned trace summary includes enough context to evaluate delight output
- the scenario matches one of the shared fixture notes that will be encoded in
  `tests/fixtures/063-assistant-delight-followthrough/`

## 2. Verify profile-aware explanation disclosure

Use a scenario that activates reasoning-profile support when available, then
exercise the explanation surfaces backed by inspect or status.

Expected outcome:

- `challenge`, `hidden-impact`, or `explain-plan` disclose whether a reasoning
  profile is active
- active profile output names the profile, why it was selected, and what it
  changed
- degraded scenarios explicitly disclose fallback behavior rather than implying
  full profile-aware support

## 3. Verify inspect closure views

Exercise the implemented inspect projections for `context`, `council`, and
`timeline` on the same session.

Expected outcome:

- context explains evidence assembly, provenance, credibility, and weak-context
  signals in operator language
- council explains whether review or council activity happened, was skipped, or
  was unavailable
- timeline preserves decision, review, governance, step, and recovery order
  together with the authoritative terminal status and reason
- none of the views require reading raw trace payloads first

## 4. Verify host parity and fallback behavior

Review the assistant asset surfaces for the hosts touched by this slice:

- Claude, Codex, and Copilot should retain their existing delight coverage
- Cursor must remain an explicit copy-ready asset surface or move to a richer
  parity state intentionally
- Gemini must remain an explicit CLI-first fallback unless richer parity is
  intentionally implemented

Expected outcome:

- every host declares one clear support mode
- `.boundline/session.json` and CLI output remain the authority for all hosts
- unsupported host behavior points to an explicit fallback CLI path

## 5. Verify delight usefulness signals

After at least one delight answer and one operator decision, inspect the
implemented signal projection.

Expected outcome:

- time to first useful answer is visible once a delight answer produces a
  bounded next action that is later accepted without override in the same
  session; the stored signal identifies the command that produced it
- explanation attribution completeness or rate is inspectable from session or
  trace authority
- next-action acceptance or override behavior is visible without mining
  unrelated logs

## 6. Run focused validation

Run the narrowest checks that cover the touched surfaces:

```bash
cargo test --test contract assistant_command_pack_contract
cargo test --no-run --all-targets
```

Also run the new focused unit or integration tests added for:

- inspect or status projections
- reasoning-profile disclosure output
- session or trace usefulness signal capture
- Cursor or Gemini host parity or fallback behavior