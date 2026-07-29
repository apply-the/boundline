# M2b-B Canon stable CLI and one-shot RPC evidence

## Decision

GO for completion of T055 and T056 only. T057-T059 remain unstarted.

Starting commits were Boundline
`bc37ce4bb9aca6e8634301a64f9fdfdf4efe5468`, Canon
`d19a47f49d5114af2a88a0e9adbad93938ee453b`, and the Speckit adapter
`ce7cb2e57c7ad8bd38064c684a4736029397756c`.

Canon closure commits are `2c67c84` (RED boundary), `959ed51`
(standalone CLI), `7d833d4` (one-shot RPC), and `0cded7e`
(equivalence and fault tests). The final Canon closure commit is
`0cded7e`.

## Frozen inventories

The pre-change visible Canon roots also exposed `verify`, `skills`,
`governance`, `pr-review`, `list`, `policy-shaping`,
`observability-design`, and `help-next`. Those remain hidden compatibility
paths, not stable aliases. The final visible ordered roots are:

```text
init run resume status approve inspect publish assistant rpc
```

The exact stable profiles are `discovery`, `requirements`, `architecture`,
`backlog`, `change`, `refactor`, `verification`, `pr-review`, and `incident`.
There is no stable `implementation` profile.

The exact published one-shot operations are:

```text
capabilities start refresh approve inspect publish
```

## Real handler mapping

| Surface | Handler and effect |
| --- | --- |
| `init` | Existing `EngineService` initialization and optional assistant installation |
| `run --profile` | Typed draft admission through the M2b-A builder, validator, graph, and atomic store |
| `resume` | Existing persisted-run continuation handler |
| `status` | Existing read-only run status projection |
| `approve` | Existing named-authority handler |
| `inspect decision-memory` | Read-only public M2b-A graph projection |
| `publish` | Existing Canon packet/projection publication; never authoritative workspace mutation |
| `assistant install` | Existing embedded host-pack installer |
| `rpc --stdio` | Bounded one-request dispatcher over the deterministic governance service |
| RPC `capabilities` | Exact operation/profile registry projection |
| RPC `start`, `approve` | Typed bundle admission or exact durable replay |
| RPC `refresh`, `inspect`, `publish` | Read-only projection of the durable snapshot |

No operation executes a model, provider, network request, semantic reviewer,
credential read, or background process. Recorded execution-audit counters
remain zero for every category.

## Framing and failure contract

RPC reads one complete JSON object, bounded to 1,048,576 bytes, emits one JSON
response, flushes stdout, and terminates. Empty input, whitespace, malformed
JSON, invalid UTF-8, arrays, unknown fields, unknown operations, trailing
bytes, concatenated values, oversized input, and semantic-execution requests
fail closed. Successful machine output contains no diagnostic, terminal
escape, secret, debug, or local absolute-path data.

MCP is absent and runtime-disabled. T056 freezes JSON stdin/stdout only; no
normative MCP frame exists in this milestone.

| Condition | Exit |
| --- | ---: |
| accepted / capabilities | 0 |
| invalid invocation (Clap) | 2 |
| invalid input / framing / internal invariant fallback | 1 |
| stale state | 2 |
| authority denied | 3 |
| required evidence missing / deterministic rejection | 5 |
| persistence failure | 6 |
| identity or digest conflict | 7 |
| unsupported operation or contract state | 8 |

Mutation identity is `request_id == bundle_id`. Exact retries return the
recorded graph digest without rewriting the snapshot. Different canonical
content under the same identity returns `identity_digest_conflict` and
preserves the original bytes. A forced persistence obstruction returns
`persistence_failure` and creates no terminal success.

## RED and equivalence evidence

The first focused run was intentionally RED: six tests ran, two passed and
four failed because the root inventory drifted, `--profile` was absent, RPC
was unregistered, and framing had no typed response. Commit `2c67c84`
preserves that boundary.

The final contract group has fourteen passing cases. It proves CLI mutation
followed by RPC inspection and RPC mutation followed by CLI inspection have
equal terminal status, graph digest, and decision-memory projection. Read-only
operations preserve exact snapshot bytes. Authority denial, insufficient
evidence, identity conflict, persistence failure, strict decoding, and
transport rejection all return non-success without synthetic completion.

## Verification

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --test stable_surface --all-features` | PASS, 14 |
| `cargo test --test cli_contract --all-features` | PASS, 10 |
| `cargo test --test profile_registry --all-features` | PASS, 6 |
| `cargo test --workspace --all-features` | PASS after correcting the historical help assertion |
| `cargo nextest run --workspace --all-features` | PASS, 1,755 passed, 0 skipped in 379.834s |
| `cargo deny check licenses advisories bans sources` | PASS |
| dedicated-target `cargo llvm-cov --workspace --all-features --no-clean --summary-only` | PASS, 95.342318% line coverage (45,914/48,157), 95.34% reported |
| LCOV intersection with the M2b-B production Rust diff | PASS, 94.543147% patch coverage (745/788) |

The first complete suite run exposed one historical assertion that prohibited
the substring `review`, contradicting the frozen `pr-review` profile. The test
now rejects only actual nonstable profiles and the full gate passes.

One intentionally concurrent Nextest attempt collided with the local GPG
keybox while `cargo test` was creating signed fixture commits. The isolated
rerun above exercised all 1,755 tests and passed; no product test was waived
or reclassified.

## Contract and dependency disposition

`canon-contracts 0.90.0` has no diff from source commit
`bd361d7e2ad112e5e0d599267e8024e748293605`. Signed tag `0.90.0` still targets
that commit and verifies with EDDSA fingerprint
`16A8F86BAA21130A2657B41B25C2C7D21E91FD50`. The published artifact checksum
remains `f6874bfbeca46f10307a9345c7aba84ba9d67fbb0b7a83b1eb6e1d3e17d44680`.

Canon CLI now directly consumes the already published workspace contract
crate, producing only the expected lockfile dependency edge. No public DTO,
package version, tag, registry artifact, Boundline runtime, or adapter file
changed.

## Review

Specification review found and resolved the ambiguous historical help test.
Safety review found and resolved persistence errors initially classified as
generic input and terminal non-accepted states initially returning process
success. The final transport has no request loop, background task, semantic
execution, inferred authority, generic `ok`, hidden fallback, or partial
persistence represented as success.
