# M1A Nextest Reconciliation

**Acceptance date**: 2026-07-26

**Owning task**: T011 pre-implementation gate

**Disposition**: Accepted as M1A closure evidence

This record supplements, and does not replace or revise, the historical M0
measurements in `baseline.md`.

The accepted M1A closure identities were Boundline
`8e070682fa45a7c3895e5abe239d635ee0a02f53` and Canon
`6f883253273218cce38190b547db60c77a8b8869`.

## Command discrepancy

M0 used the complete workspace and feature selection:

```text
Boundline:
cargo nextest run --workspace --all-features

Canon:
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.gpgsign \
GIT_CONFIG_VALUE_0=false \
cargo nextest run --workspace --all-features
```

M1A accidentally omitted `--workspace --all-features`:

```text
Boundline:
cargo nextest run

Canon:
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.gpgsign \
GIT_CONFIG_VALUE_0=false \
cargo nextest run
```

Because each repository root is itself a package, the M1A commands selected
only that root package. Boundline omitted `boundline-core`,
`boundline-adapters`, `boundline-cli`, and the newly added
`boundline-protocol` package. Canon omitted `canon-engine`, `canon-cli`,
`canon-adapters`, and the newly added `canon-contracts` package.

Both corrected commands used nextest's default profile with the repositories'
unchanged nextest configuration. `--all-features` restored the complete
feature selection. Canon retained the M0 Git configuration override shown
above; no additional test-selection environment variable was present.

## Counts

| Repository | M0 complete | M1A reported | Corrected M1A selection |
|---|---:|---:|---:|
| Boundline | 2,662 passed, 2 skipped | 1,274 passed, 2 skipped | 2,669 passed, 2 skipped |
| Canon | 1,691 passed | 421 passed | 1,699 passed |

The corrected increases over M0 are exactly the seven
`boundline-protocol` tests and eight `canon-contracts` tests introduced by
M1A.

## Conclusion

No test target, workspace member, feature, or nextest configuration
disappeared between M0 and M1A. The discrepancy was command-selection error,
not corpus loss. The corrected workspace/all-feature commands passed at the
M1A closure commits, and their current pre-M1B selections remain 2,669
Boundline tests and 1,699 Canon tests.
