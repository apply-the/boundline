# M1C command-surface verification

Evidence date: 2026-07-27

Owning tasks: T013 and T014

Starting Boundline commit:
`c6d33a02b7fe15f160bd64ffc79ba6797910b464`

Decision: **GO M1C**

T013 and T014 are complete. The command-surface contract contains sixteen
tests and all sixteen pass. The required advisory-inclusive `cargo deny` gate
passed during the dedicated M1C closure run. The earlier managed-environment
lock failures remain recorded below as historical evidence.

## Repository isolation

The preflight found all three repositories on
`083-governed-1-0-release`, clean, and at their expected commits:

- Boundline: `c6d33a02b7fe15f160bd64ffc79ba6797910b464`
- Canon: `6f883253273218cce38190b547db60c77a8b8869`
- Speckit adapter: `8821fe4eb815a825d05e98e7da6bc374e9e6b810`

Only Boundline was changed.

## Command inventories

The pre-change public root parser registered:

```text
doctor goal flow plan probe step run orchestrate workflow index
checkpoint inspect status next continue session govern assistant init
update config adapter provider cluster models council override help-next
evals trace exec
```

The proposed post-change stable root inventory, in deterministic help order,
is:

```text
init goal plan run status inspect doctor config models provider adapter
index session assistant update
```

Clap's generated root `help` subcommand is disabled. The `--help` flag remains
operational. The contract extracts the rendered `Commands:` section and
compares it exactly, in order, with the fifteen-command inventory above; a
generated `help` row or any other extra row fails the contract.

The hidden preview gateway contains:

```text
preview flow
preview workflow
preview cluster
preview council
preview evals
preview override
preview trace
```

`checkpoint` and `exec` remain callable only through internal Rust APIs and are
absent from the public parser, help, and completion inventory.

The following StableTargetPending commands remain absent:

```text
approve
session abort
session cleanup
recover inspect
recover complete
recover restore
recover abandon
rpc
serve --transport mcp-stdio
```

Removed roots fail before normal parsing with deterministic, side-effect-free
migration guidance:

```text
orchestrate -> boundline run --until
step        -> boundline run --one-step
continue    -> boundline run --resume
next        -> structured next_actions from boundline status
probe       -> boundline doctor or boundline status
help-next   -> boundline doctor or boundline status
govern      -> boundline plan or boundline run
```

The `govern` diagnostic does not advertise the pending `approve` command.

## Test-first record

The first focused execution was intentionally RED:

```text
cargo test --test cli_090 --all-features
```

- Start: 2026-07-27T11:03:53Z
- Exit: 101
- Duration: 0.36 seconds
- Result: ten compile errors because the canonical command-surface module,
  preview gateway, stable run routes, and removal diagnostic were not yet
  implemented.

The pre-change complete nextest selection was 2,688 passed and 2 skipped.
The final complete selections are 2,704 passed and 2 skipped. The increase is
exactly the sixteen M1C contract tests; no prior test identity was removed.

A base-to-worktree audit of every changed Rust file found zero missing
pre-M1C `#[test]` function identities. In particular, the following seven
historical names remain present with assertions updated for the M1C surface:

```text
run_plan_cli_parses_plan_accepted_plan_and_resume_modes
run_plan_dispatch_defaults_workspace_to_current_directory
run_plan_dispatch_reports_requested_variants_and_missing_plan
orchestrate_cli_parses_stream_intent_and_goal
orchestrate_cli_parses_planning_stage_completion_resume
orchestrate_cli_parses_goal_clarification_answer_resume
orchestrate_cli_accepts_legacy_intent_values
```

The subsequent quality review also found that several probe, govern, and
voting tests retained their names but had been reduced to duplicate public
removal assertions. Those tests now exercise the existing internal
`execute_probe` and `execute_govern` capabilities and validate their persisted
status projections. A repository search finds no remaining test use of the
generic removed-command helper outside its definition; public removal remains
covered once in `cli_090`.

## Quality-review corrections

- Preview commands with optional workspace selection now resolve the current
  repository and load its provider environment when `--workspace` is omitted.
  Preview cluster commands default their workspace argument to `.` so the same
  rule applies to all seven gateway commands.
- `run --one-step` and `run --resume` conflict with every custom-run and
  orchestration-only option. `run --until` rejects `--mode` and
  `--compatibility`, which its existing orchestration handler does not consume,
  while retaining the goal, brief, governance, risk, zone, owner, and
  `--no-canon` inputs that handler does consume.
- Incompatible route invocations fail during parsing and leave an isolated
  workspace unchanged.
- VS Code settings migration removes both obsolete standalone workflow regex
  keys and the broader obsolete pre-M1C keys. Duplicate managed regex literals
  were removed from the cleanup catalog.
- Tests consume the exported canonical classification inventories. The one
  independent stable list remains the exact golden help fixture. Because Clap
  derives registrations from Rust enum variants, a binding contract compares
  the derived parser and rendered help with the lower-dependency inventory.

The executable RED observed during this review was:

```text
cargo test --test integration \
  init_vscode_read_only_auto_approve_merges_existing_settings --all-features
```

- Exit: 101
- Failure: obsolete
  `^boundline workflow (list|status|inspect)` remained in managed settings.

After restoring direct internal probe coverage, the first six-test probe run
also exposed process-environment leakage between the credentialed and
credential-free cases (5 passed, 1 failed). A scoped provider-environment guard
made those cases deterministic. The preview and route-conflict tests were
added after the first narrow implementation edit, so no separate executable
RED was captured for those two findings; this sequencing deviation is
recorded rather than reconstructed.

## Verification results

```text
cargo fmt --all -- --check
```

- Exit: 0
- Final parent verification duration: 1.46 seconds

```text
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

- Exit: 0
- Duration: 9.64 seconds
- Review rerun: exit 0 in 5.29 seconds
- Final parent verification: exit 0 in 0.37 seconds

```text
cargo test --test cli_090 --all-features
```

- Exit: 0
- Result: 16 passed
- Review rerun duration: 5.14 seconds including compilation; tests completed in
  1.02 seconds
- Final parent verification: 16 passed in 1.40 seconds; tests completed in
  1.15 seconds

```text
cargo test -p boundline-cli --lib --all-features
```

- Exit: 0
- Result: 419 passed
- Quality-review rerun duration: 16.59 seconds

```text
cargo test --test contract --all-features
```

- Exit: 0
- Result: 306 passed
- Duration: 35.91 seconds

```text
cargo test --test integration --all-features
```

- Exit: 0
- Result: 298 passed, 2 skipped
- Duration: 59.21 seconds

```text
cargo test --workspace --all-features --quiet
```

- Exit: 0
- Result: 2,704 passed, 2 skipped
- Final parent verification duration: 136.61 seconds

```text
cargo nextest run --workspace --all-features
```

- Exit: 0
- Result: 2,704 passed, 2 skipped across 13 binaries
- Final parent verification duration: 101.30 seconds; nextest execution
  completed in 100.479 seconds

```text
cargo nextest list --workspace --all-features --message-format oneline
```

- Exit: 0
- Result: 2,704 selected test identities

```text
cargo deny check licenses advisories bans sources
```

- Exit: 1 before repository analysis
- Result: the managed sandbox makes
  `<cargo-home>/advisory-dbs/db.lock` read-only, so `cargo deny` could not
  acquire its advisory database lock.
- Escalated execution was rejected by the platform usage limit before process
  start. This is an environmental non-execution, not a successful gate.

```text
cargo deny check licenses bans sources
```

- Exit: 0 in the parent verification environment
- Duration: 0.591770083 seconds
- Result: `bans ok`, `licenses ok`, and `sources ok`.
- Existing allowed duplicate warnings were emitted for `getrandom`,
  `hashbrown`, `thiserror`, `thiserror-impl`, and `windows-sys`.
- Disposition: partial evidence only. It does not satisfy or replace the
  required combined command because the advisory check did not run.

### Advisory-inclusive closure gate

```text
cargo deny check licenses advisories bans sources
```

- Tool: `cargo-deny 0.19.0`
- Start: `2026-07-27T14:11:53Z`
- End: `2026-07-27T14:12:22Z`
- Command duration: 9.96 seconds
- Exit: 0
- Result: `advisories ok`, `bans ok`, `licenses ok`, and `sources ok`
- Advisory source: RustSec `https://github.com/RustSec/advisory-db`
- Advisory revision: `29638ff054fdbb83d2844240f7ef7e576cb52629`
- Fetch timestamp: `2026-07-27T16:12:15+0200`
- Advisory revision timestamp: `2026-07-25T17:33:50+02:00`
- Advisory revision subject: `add advisory for nostr`
- Existing allowed duplicate warnings remained unchanged for `getrandom`,
  `hashbrown`, `thiserror`, `thiserror-impl`, and `windows-sys`.
- Disposition: the repository's existing duplicate policy accepts these
  transitive version lines; no dependency or policy file changed during M1C
  closure.

## Coverage

Disk was checked before coverage. A dedicated generated target at
`<temp>/boundline-m1c-coverage-target` was used.

Focused coverage commands passed:

```text
cargo llvm-cov --workspace --all-features --test cli_090 --lcov
cargo llvm-cov -p boundline-cli --all-features --lib --lcov
```

Measured line coverage after merging those focused reports:

```text
src/cli.rs:                 90.05% (4833/5367)
src/cli/command_surface.rs: 69.05% (29/42)
```

The focused executable patch-line measurement is 93.80 percent (348/371) for
`src/cli.rs`. Treating every executable line in the new canonical inventory as
an added patch line produces a combined result of 91.28 percent (377/413).
This satisfies the frozen 90 percent patch gate without exclusions or ignored
source paths. The focused unit coverage executes every preview dispatch arm
and all three stable `run` routes against isolated workspaces.

## Closure

T013 and T014 are accepted. The required advisory-inclusive `cargo deny`
command executed successfully with current RustSec data.

No T017 or later work was started.
