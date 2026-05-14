# Quickstart: Delivery Control Consumer

## Goal

Validate the smallest useful consumer slice: Boundline can read Canon-owned
repo-visible knowledge, classify the result, and surface explicit continue,
warning, or hard-stop outcomes.

## Scenario 1: Stable Canon knowledge

1. Provide a workspace with compatible Canon-managed `docs/project/` and
   `docs/evidence/` surfaces.
2. Run the session-native planning path.
3. Confirm Boundline can treat the stable inputs as credible context and expose
   the relevant Canon refs in inspection output.

## Scenario 2: V1 hard stop

1. Provide a workspace where Canon governance is blocked or a required source
   artifact is missing.
2. Run the same planning path.
3. Confirm Boundline stops explicitly instead of fabricating missing producer
   facts.

## Scenario 3: V1 warning and replan

1. Provide a workspace with stale project memory or a missing evidence source,
   while other credible context still exists.
2. Run planning and inspection.
3. Confirm Boundline warns or replans instead of escalating the condition to a
   hard stop.

## Scenario 4: Contract incompatibility

1. Provide Canon repo-visible output that advertises an unknown major contract
   line.
2. Run planning or inspection.
3. Confirm Boundline rejects the input explicitly and surfaces repair guidance.

## Scenario 5: Mixed evidence authorship

1. Provide a `docs/evidence/` file with Canon and Boundline managed blocks.
2. Inspect the file and the session-native output.
3. Confirm each block remains attributable to its producer and source ref.

## Fixture Bundle Requirements

Each deterministic fixture bundle used by this slice should contain:

1. a `project.boundline.toml` example;
2. a `.boundline/workflows.toml` example with a `delivery_paths` section;
3. stable or stale `docs/project/` fixtures with compatible sidecars;
4. mixed-producer `docs/evidence/` fixtures using the shared managed-block
   marker; and
5. enough metadata to prove continue, warning, replan, or hard-stop outcomes.

## Validation Commands After Implementation

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --no-run --all-targets`
- `cargo nextest run`