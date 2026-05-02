# Quickstart: Context Assembly Foundation

Exercise the complete `0.33.0` story: explicit context assembly before plan confirmation, surfaced provenance on the primary path, explicit stop behavior for non-credible context, and release validation.

## Preconditions

- Work from the repository root.
- Use a workspace with:
  - authored input or direct goal text
  - at least one relevant source file
  - optional `.canon/` artifacts to verify governed evidence reuse
- Ensure no stale `SPECIFY_FEATURE` environment variable points at an older slice when using Speckit scripts.

## Scenario 1: Context pack is created during planning

1. Start a fresh session:
   `cargo run --bin synod -- start`
2. Capture a bounded goal with authored context:
   `cargo run --bin synod -- capture --goal "fix the failing context summary output"`
3. Plan the task:
   `cargo run --bin synod -- plan`
4. Verify the output and persisted session expose:
   - a context summary
   - a credible context state
   - explicit primary context inputs
   - provenance or narrowing lines explaining why those inputs were selected

## Scenario 2: Context projection stays visible during follow-through

1. Continue with:
   `cargo run --bin synod -- status`
2. Ask for the bounded next action:
   `cargo run --bin synod -- next`
3. Inspect the current authoritative trace or session:
   `cargo run --bin synod -- inspect`
4. Verify the same context-pack vocabulary remains visible on these surfaces.

## Scenario 3: Non-credible context stops planning explicitly

1. Capture a goal in a workspace with no credible relevant code or artifact inputs.
2. Run:
   `cargo run --bin synod -- plan`
3. Verify planning stops explicitly and the output explains that the context pack is insufficient or stale rather than silently proceeding.
4. Verify `status` or `next` keeps the same blocked context summary visible so the recovery action is inspectable.

## Scenario 4: Release closeout

1. Run formatting:
   `cargo fmt --all`
2. Run linting:
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. Run compile-oriented validation when needed:
   `cargo test --no-run --all-targets`
4. Run focused tests and the broader suite as needed.
5. Refresh coverage and confirm touched Rust files remain above 95%.
