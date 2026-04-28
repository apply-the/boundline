# Quickstart: Human-Facing Brief Ingestion

## Prerequisites

- Work from the repository root.
- Start with no active `.synod/session.json` in the target workspace or create a fresh workspace fixture.
- Keep authored inputs inside the workspace boundary.
- Use Markdown files for attached or referenced briefs in the first slice.
- Run commands through `cargo run --bin synod -- ...` when validating locally.

## Scenario 1: Start a bug fix from direct text only

```bash
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture \
  --workspace <workspace> \
  --goal "Fix the flaky login retry test in the auth flow"
cargo run --bin synod -- flow bug-fix --workspace <workspace>
cargo run --bin synod -- plan --workspace <workspace>
cargo run --bin synod -- status --workspace <workspace>
```

Expected outcome:

- Synod captures the direct text as the active authored brief without requiring `.synod/execution.json`.
- `status` shows the bounded goal and recommends the next command.
- No clarification is required when the request is already specific enough to plan.

## Scenario 2: Reuse multiple Markdown briefs and referenced workspace documents

```bash
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture \
  --workspace <workspace> \
  --goal "Implement caching for search results using docs/architecture/search-cache.md and docs/bugs/cache-regression.md" \
  --brief docs/architecture/search-cache.md \
  --brief docs/bugs/cache-regression.md
cargo run --bin synod -- flow change --workspace <workspace>
cargo run --bin synod -- plan --workspace <workspace>
cargo run --bin synod -- inspect --workspace <workspace>
```

Expected outcome:

- Synod resolves the explicit briefs first, then deduplicates any repeated workspace document references mentioned in the goal text.
- `inspect` exposes the accepted source order, any deduplication performed, and the resulting bounded brief summary.
- If one source is missing or conflicts materially with another, Synod stops with one explicit clarification instead of silently dropping it.

## Scenario 3: Start a governed change from human business values

```bash
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture \
  --workspace <workspace> \
  --goal "Prepare the payments retry change for the next release" \
  --brief docs/payments/retry-brief.md \
  --governance canon \
  --risk medium \
  --zone payments \
  --owner platform
cargo run --bin synod -- flow change --workspace <workspace>
cargo run --bin synod -- plan --workspace <workspace>
cargo run --bin synod -- run --workspace <workspace>
cargo run --bin synod -- status --workspace <workspace>
```

Expected outcome:

- Synod records the human-facing governance intent without asking for stage IDs, Canon modes, or manifest fields.
- The existing governance runtime path applies the internal stage-scoped behavior after planning.
- If approval or a missing business field blocks governed execution, `status` reports the blocked or awaiting-approval state together with the next user-facing action.

## Scenario 4: Explicit clarification for an unbounded request

```bash
cargo run --bin synod -- start --workspace <workspace>
cargo run --bin synod -- capture \
  --workspace <workspace> \
  --goal "Improve the platform docs and fix whatever tests are broken"
cargo run --bin synod -- plan --workspace <workspace>
```

Expected outcome:

- Synod refuses to invent a bounded flow or implementation scope from an overly broad brief.
- The session records one targeted clarification that asks only for missing external business context.
- Planning does not continue until the operator provides a narrower request.

## Validation

Run the repository validation commands after implementation:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Expected outcome:

- CLI validation covers direct text, Markdown brief ingestion, workspace boundary checks, and governed human-input scenarios.
- Session, status, inspect, and trace surfaces preserve authored input provenance across the full delivery loop.
- Existing manifest-driven workflows remain available for automation and tests.