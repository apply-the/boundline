# Quickstart: Guided CLI UX And Clearer Messaging

## Scenario 1: Guided init explains optional routes

```bash
cargo run --bin boundline -- init
```

Expected outcome:

- The assistant prompt lists supported assistants inline.
- The route prompt says blank input is allowed when defaults exist.
- The route prompt shows supported slots and a valid example.
- The success summary shows `route_setup`, `assistant_setup`, and where to inspect overrides.

## Scenario 2: Explicit route override remains understandable

```bash
cargo run --bin boundline -- init \
  --assistant copilot \
  --route planning=copilot:gpt-4o
```

Expected outcome:

- The summary distinguishes the explicit planning override from the remaining
  seeded default slots inside `route_setup`.
- The output tells the operator where to inspect the effective config later.

## Scenario 3: Malformed route input explains recovery

```bash
cargo run --bin boundline -- init \
  --assistant copilot \
  --route planning-copilot-gpt-4o
```

Expected outcome:

- The command exits non-zero.
- The error names the malformed route.
- The error explains the expected `SLOT=RUNTIME:MODEL` shape and shows a valid
  example.

## Scenario 4: Existing assistant assets preview safely

```bash
cargo run --bin boundline -- init \
  --assistant copilot
```

Expected outcome when assistant assets already exist:

- The command returns preview-only output unless `--force` is provided.
- Preview lines mention refreshing the relevant assistant pack.
- No existing assistant file is silently replaced.

## Scenario 5: Doctor output remains readable in rich and plain modes

```bash
cargo run --bin boundline -- doctor --install
cargo run --bin boundline -- doctor
```

Expected outcome:

- Install and workspace diagnostics group content into `summary`, `checks`, and
  `actions` clearly.
- Redirected or CI output keeps the same meaning without depending on color.