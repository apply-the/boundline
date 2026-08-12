# T101 Boundline US1 Coverage Closure Evidence

Accepted on 12 August 2026 for branch `083-governed-1-0-release`.

## Scope and immutable boundaries

- Starting HEAD: `310c4821f2b513d42fa7b78fb5753eada7f2008f`.
- Accepted US1 range: `a86adb1f89aa35c2df6963ba39748e9d1fe65f0a..310c4821f2b513d42fa7b78fb5753eada7f2008f`.
- T101 test commit: `6f1b7e2455b6d753302bd9a8a9e033780544fa8c`.
- T021-T037 remain complete. T038-T048 remain unstarted.
- No production behavior, protocol, package version, threshold, coverage
  exclusion, waiver, adapter source, tag, or published artifact changed.
- T088 remains incomplete because it owns the future release-wide file gate;
  T101 closes only the accepted US1 deficit.

The historical accepted US1 measurement of 108,084/116,783 (92.551142%) in
`us1-t021-t037.md` is preserved. A fresh run at the T101 starting HEAD produced
105,984/112,568 (94.151091%) using the current official aggregation script.
The fresh measurement is the remediation baseline; it does not rewrite the
historical observation.

Forbidden throughout T101: coverage exclusions, threshold changes, ignore
attributes, denominator-only dead-code removal, assertion-free line touching,
snapshot churn, new waivers, and T038+ behavior.

## Exact US1 Rust-file inventory

Git-derived production inventory and fresh pre-T101 coverage:

| Owning task | Production file | Pre-T101 coverage |
|---|---|---:|
| T037 | `crates/boundline-cli/src/lib.rs` | N/A: no executable lines |
| T037 | `crates/boundline-cli/src/projection/mod.rs` | N/A: no executable lines |
| T037 | `crates/boundline-cli/src/projection/session.rs` | 90/97 (92.78%) |
| T030 | `crates/boundline-core/src/execution/capability.rs` | 126/130 (96.92%) |
| T028 | `crates/boundline-core/src/execution/lease.rs` | 111/117 (94.87%) |
| T028-T032 | `crates/boundline-core/src/execution/mod.rs` | N/A: no executable lines |
| T032 | `crates/boundline-core/src/execution/worktree.rs` | 149/158 (94.30%) |
| T022 | `crates/boundline-core/src/identity/mod.rs` | N/A: no executable lines |
| T022 | `crates/boundline-core/src/identity/repository.rs` | 139/149 (93.29%) |
| T022-T036 | `crates/boundline-core/src/lib.rs` | N/A: no executable lines |
| T036 | `crates/boundline-core/src/publication/mod.rs` | N/A: no executable lines |
| T036 | `crates/boundline-core/src/publication/transaction.rs` | 241/271 (88.93%) |
| T034 | `crates/boundline-core/src/transaction/challenge.rs` | 112/113 (99.12%) |
| T026 | `crates/boundline-core/src/transaction/evidence.rs` | 58/59 (98.31%) |
| T024 | `crates/boundline-core/src/transaction/fingerprint.rs` | 98/105 (93.33%) |
| T037 | `crates/boundline-core/src/transaction/governed_session.rs` | 76/80 (95.00%) |
| T024-T037 | `crates/boundline-core/src/transaction/mod.rs` | N/A: no executable lines |
| T037 | `src/cli.rs` | 5,193/5,434 (95.56%) |
| T037 | `src/cli/command_surface.rs` | 29/42 (69.05%) |

The remaining Rust files in the range are test and integration fixture files,
not production files subject to the per-file T088 rule.

## Behavioral remediation

The test-only commit adds assertions for real accepted US1 behavior:

- repository identity rejects a non-Git directory;
- fingerprints expose a versioned, content-sensitive digest;
- execution leases reject revision changes while held and reject path-escaping
  session identifiers;
- managed worktrees reject nested state roots and preserve an existing checkout
  on duplicate session creation;
- governed projections preserve independent capability and completed-publication
  authority denials;
- command-surface constructors preserve the stable/removed diagnostic boundary;
- publication rejects invalid refs, mismatched bases, concurrent publishers,
  non-Git repositories, symlink/path escapes, tampered durable manifests, and
  Git inspection failures;
- publication preserves concurrent untracked/index state and fails final tree
  or cleanliness verification after compare-and-swap;
- successful publication projects the exact authoritative tree digest.

No genuine production defect was found. Two focused tests initially used
invalid fixtures: an untracked path where the deletion helper requires an
indexed path, and a non-`diff` Git command where `git_quiet` interprets exit 1
as a real diff. Both fixtures were corrected before acceptance; production
source remained unchanged apart from `#[cfg(test)]` modules.

## Final coverage

Final command, run after the complete regression suite and from a fully cleaned
Cargo target:

```text
bash scripts/coverage.sh -- --test-threads=1
```

- Exit: 0; duration: 529.90 s.
- Aggregate: 106,153/112,700 = 94.190772% (gate: 92.77%).
- LCOV: `lcov.info`.
- LCOV SHA-256: `d9c373b1aba6f3f833d93310dcd31f6847cde859f3a66a67d1c23208efade80d`.
- Log SHA-256: `0a83b325fbcb503cfa62ecfe2296d14aa148bae24c226acda7f6055e8bb49000`.

Two preceding post-regression attempts failed before tests at the compiler/linker
boundary with `ENOSPC`. The first cleanup removed 3.6 GiB from the isolated
LLVM target but did not provide enough linker headroom. A scoped `cargo clean`
then removed 47.2 GiB of regenerable Cargo artifacts. The final run started
with 34 GiB free and completed from scratch. These failures changed no tracked
state and are retained as environmental evidence.

Final per-file results from the same LCOV report:

| Production file | Final coverage |
|---|---:|
| `crates/boundline-cli/src/lib.rs` | N/A: no executable lines |
| `crates/boundline-cli/src/projection/mod.rs` | N/A: no executable lines |
| `crates/boundline-cli/src/projection/session.rs` | 108/111 (97.30%) |
| `crates/boundline-core/src/execution/capability.rs` | 126/130 (96.92%) |
| `crates/boundline-core/src/execution/lease.rs` | 113/117 (96.58%) |
| `crates/boundline-core/src/execution/mod.rs` | N/A: no executable lines |
| `crates/boundline-core/src/execution/worktree.rs` | 151/158 (95.57%) |
| `crates/boundline-core/src/identity/mod.rs` | N/A: no executable lines |
| `crates/boundline-core/src/identity/repository.rs` | 142/149 (95.30%) |
| `crates/boundline-core/src/lib.rs` | N/A: no executable lines |
| `crates/boundline-core/src/publication/mod.rs` | N/A: no executable lines |
| `crates/boundline-core/src/publication/transaction.rs` | 350/368 (95.11%) |
| `crates/boundline-core/src/transaction/challenge.rs` | 112/113 (99.12%) |
| `crates/boundline-core/src/transaction/evidence.rs` | 58/59 (98.31%) |
| `crates/boundline-core/src/transaction/fingerprint.rs` | 101/105 (96.19%) |
| `crates/boundline-core/src/transaction/governed_session.rs` | 76/80 (95.00%) |
| `crates/boundline-core/src/transaction/mod.rs` | N/A: no executable lines |
| `src/cli.rs` | 5,193/5,434 (95.56%) |
| `src/cli/command_surface.rs` | 61/63 (96.83%) |

The official diff/LCOV intersection for
`310c4821f2b513d42fa7b78fb5753eada7f2008f..6f1b7e2455b6d753302bd9a8a9e033780544fa8c`
measured 122/132 executable changed lines = 92.424242% (gate: 90%).

## Regression gates

| Command | Result | Duration |
|---|---|---:|
| `cargo fmt --all -- --check` | PASS | 1.47 s |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS | 0.33 s |
| `cargo test --workspace --all-features` | PASS | 232.16 s |
| `cargo nextest run --workspace --all-features` | PASS: 2,788 passed, 2 policy skips | 165.81 s |
| `cargo test --workspace --all-features catalog` | PASS: 38 selected tests | 3.14 s |
| `cargo deny check licenses advisories bans sources` | PASS; five declared duplicate-version warnings | 1.38 s |
| `bash scripts/check-no-local-paths.sh` | PASS | 0.24 s |
| `bash scripts/check-rust-no-panic.sh` | PASS | 18.44 s |
| `git diff --check` | PASS | 0.02 s |
| `npm run docs:build` | PASS; known non-fatal Vite chunk-size advisory | 6.94 s |

Nextest increased from 2,769 to 2,788 executed tests, exactly 19 additional
test targets, while retaining the two policy skips. No test target disappeared.

## Independent integrity, scope, and quality review

- Coverage thresholds remain 92.77% aggregate, 95% for every executable US1
  production file, and 90% patch coverage.
- Git/LCOV inventory matches the accepted US1 range; module-only files are
  recorded as N/A rather than assigned synthetic percentages.
- No exclusion, ignored line, waiver, threshold edit, denominator-only source
  deletion, or assertion-free synthetic test was introduced.
- Every new test names an observable authority, durability, Git/filesystem, or
  fail-closed regression that would make it fail.
- T021-T037 semantics are preserved. No recovery journal, reconciliation,
  adapter, protocol, version, publication, tag, or T038+ work was introduced.
- No Critical or Important review finding remains.

Decision: **GO T101**. T038-T048 are cleanly unblocked but remain unstarted.
