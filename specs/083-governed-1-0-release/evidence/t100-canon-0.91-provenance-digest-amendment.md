# T100 Canon 0.91 Provenance Digest Amendment

**Evidence date**: 2026-07-30

**Decision**: GO T100

**Task state**: T100 complete; T099 incomplete and `READY_TO_PUBLISH`;
T057-T059 not started

## Historical T099 block

The first T099 execution stopped at Gate 4 because the frozen
`canon-contracts 0.91.0` provenance-manifest digest could not be reproduced.
The incorrect frozen value was:

```text
f2cb4ba8607f7463d780fb6d6e66a1a309d9a540b465749720cee912261a8aca
```

The stop occurred before upload. No package was published, no `0.91.0` tag
was created locally or remotely, no crates.io login was performed during
T100, no Boundline dependency was added, and no T057-T059 implementation
started. Public registry metadata still returned HTTP 404 for
`canon-contracts 0.91.0` at the T100 clean-state gate.

All three repositories began T100 clean on branch
`083-governed-1-0-release` at:

```text
Boundline:       a3e1c91b9ae3e31fde197d7a8c7ba0dfdd54a4b0
Canon:           cd9209a861acf8d2960b185c476fedfdac9e41d2
Speckit adapter: ce7cb2e57c7ad8bd38064c684a4736029397756c
```

Canon had no local or remote `0.91.0` tag. Canon and the adapter remained
clean and unchanged throughout the amendment.

## Frozen inputs and normative framing

The package inputs remain:

```text
canon-contracts
0.91.0
cd9209a861acf8d2960b185c476fedfdac9e41d2
eb5f20db71872d0d73fa065a47abc17fe2598cfc5ab931056ac040ab41bb98f5
10cb130804af53215e5b889d44211bcc12cea2c56a24d7b965829a8da990af86
00e88e6bf7a7cbc4d104d8a4dd742d1c4a33b6538ee0a27377a40d7c6517ead7
ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed
f45c4689f6a255e53723c3b2eeb72958e93b512dbf943e125e1b16f69eb746e9
```

The provenance preimage encodes those exact eight values as UTF-8 in that
order. One byte `0x0a` follows every field, including the eighth and final
field. The visual representation is:

```text
canon-contracts\n
0.91.0\n
cd9209a861acf8d2960b185c476fedfdac9e41d2\n
eb5f20db71872d0d73fa065a47abc17fe2598cfc5ab931056ac040ab41bb98f5\n
10cb130804af53215e5b889d44211bcc12cea2c56a24d7b965829a8da990af86\n
00e88e6bf7a7cbc4d104d8a4dd742d1c4a33b6538ee0a27377a40d7c6517ead7\n
ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed\n
f45c4689f6a255e53723c3b2eeb72958e93b512dbf943e125e1b16f69eb746e9\n
```

Here `\n` documents actual newline bytes; the two characters backslash and
`n` are not hashed. The corrected SHA-256 is:

```text
78e2d6f8b6f580bcd123c8e7bdc7ac80bef00059dd7058c8e0acd15253dcc414
```

## Independent reproduction

A temporary Python verifier accepted exactly eight values, encoded them as
UTF-8, appended a newline after each value, and emitted only the digest in
machine mode. Its focused tests covered UTF-8 and final-newline framing,
machine-only output, and rejection of seven or nine values:

```text
3 tests passed
Python result:
78e2d6f8b6f580bcd123c8e7bdc7ac80bef00059dd7058c8e0acd15253dcc414
```

An independent shell implementation used `printf '%s\n'` for the same eight
values and `shasum -a 256`:

```text
78e2d6f8b6f580bcd123c8e7bdc7ac80bef00059dd7058c8e0acd15253dcc414
```

Both implementations therefore hashed the same byte preimage.

## Published 0.90 control

The unchanged algorithm was also applied to these accepted 0.90 fields:

```text
canon-contracts
0.90.0
bd361d7e2ad112e5e0d599267e8024e748293605
72750de23605333965185af08e6521bcebff04256e5185efa331841b3e187444
6fa9df47b600a9fea5cc60e0ba6abac1686e6cb6d82163b78cd8ae429796988a
8c9e4ecb3c2f5370efd7457143a3d99caf78d04174d9648edca0455913e6baf7
f6874bfbeca46f10307a9345c7aba84ba9d67fbb0b7a83b1eb6e1d3e17d44680
681e6248d98959046a2adc40deee78cff1b8665968c85028fd382132d4b366e7
```

Python and shell independently reproduced the published control:

```text
0ffffc5274ef048086a384d515d573ebbe7cf4dc6174737f0d62a22ba7f68306
```

## Incorrect-value investigation

The incorrect value was tested against:

- omission of the final newline;
- every field-order permutation;
- omission of each field;
- duplication of each field at every position;
- abbreviated source commits;
- a formatted Markdown code block;
- literal backslash-`n` separators;
- the root manifest digest instead of the package manifest;
- the 0.90 source-tree, manifest, and lockfile inputs;
- the known intermediate 14-file artifact from source commit
  `53bebae5cae2e157d5242cdf1071cbab9f60a9d0`.

The intermediate artifact has SHA-256 `c71aa8e9c047773dd6fe6fa61dea1af6377a7920cf286a6b234affc391a54730`
and normalized payload
`5edb14463bf33e601c82ffedb68034690b10f7ebf76dbd30401aea82cbc5a22b`.
It produces provenance digest
`2a882bdaaba5768d8c0dc9e3138dd73510d62e59e3396372a32215237693b39c`
with its own source inputs, and
`034cfcc2010933163b413f2a17a441875d59e71e158783ee3027472e0151c805`
when mixed with the final source inputs.

None of the checked preimages produces the incorrect frozen value.
The original error is therefore classified as:

```text
origin not reproducible
```

## Package-input immutability

A detached worktree at
`cd9209a861acf8d2960b185c476fedfdac9e41d2` reproduced:

```text
source tree:       eb5f20db71872d0d73fa065a47abc17fe2598cfc5ab931056ac040ab41bb98f5
package manifest:  10cb130804af53215e5b889d44211bcc12cea2c56a24d7b965829a8da990af86
workspace lock:    00e88e6bf7a7cbc4d104d8a4dd742d1c4a33b6538ee0a27377a40d7c6517ead7
artifact:          ec789921edf1ac39028cfb68107d5e09d425d4efab95b13292102703aca155ed
normalized payload: f45c4689f6a255e53723c3b2eeb72958e93b512dbf943e125e1b16f69eb746e9
```

No source, manifest, lockfile, package, DTO, operation, or runtime file
changed. Canon and the Speckit adapter require no T100 commit.

## Reference and task amendment

The incorrect digest occurred only in the M2b-C0-A package-readiness
evidence. Its active readiness field now uses the corrected value while its
T100 addendum preserves the historical value and Gate 4 block.

Files changed:

```text
specs/083-governed-1-0-release/tasks.md
specs/083-governed-1-0-release/evidence/m2b-c0-outcome-contract-amendment.md
specs/083-governed-1-0-release/evidence/t100-canon-0.91-provenance-digest-amendment.md
```

The dependency order is now:

```text
T097 -> T098 -> T100 -> T099 -> T057/T058/T059
```

T100 is complete. T099 is incomplete and restored to
`READY_TO_PUBLISH`. T057-T059 remain incomplete.

## Verification and reviews

The focused temporary verifier reported three passing tests. `git diff
--check` exited successfully. The final `npm run docs:build` completed
successfully; VitePress retained its non-fatal large-chunk advisory. No Canon
script, test, source, or documentation file changed, so the conditional Canon
Rust checks were not applicable.

The provenance review found no Critical or Important issue: the field count
and order, UTF-8 encoding, actual newline bytes, final newline, SHA-256, two
independent 0.91 reproductions, 0.90 control, and immutable candidate inputs
all match the amended contract.

The release-safety review found no Critical or Important issue: T100 performed
no publish, tag, crates.io login, package-source change, Boundline dependency
change, or T057-T059 work. The old digest remains only in explicit historical
evidence; every active readiness reference uses the corrected digest.

## T099 resume input

The next separately authorized T099 run must use:

```text
provenance manifest SHA-256:
78e2d6f8b6f580bcd123c8e7bdc7ac80bef00059dd7058c8e0acd15253dcc414
```

Every other frozen source and artifact value remains unchanged. Before that
run, the operator must confirm server-side revocation of the previous
crates.io token, create a fresh short-lived `publish-update` token restricted
to `canon-contracts`, and run `cargo login` only inside the later authorized
T099 execution.
