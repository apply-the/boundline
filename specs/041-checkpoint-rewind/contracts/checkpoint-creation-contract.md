# Contract: Checkpoint Creation

## Purpose

Define the observable behavior for implicit checkpoint capture before a bounded
mutating `run` or `step` action.

## Preconditions

- A workspace or cluster session exists.
- The active bounded execution path is about to mutate one or more workspace
  files.
- The checkpoint command surface is available in the current Boundline build.

## Required behavior

- Before the first bounded mutation lands, Boundline creates one checkpoint in
  the owning workspace scope.
- The checkpoint records the triggering command, the owning session or cluster
  authority, and the bounded set of captured files.
- Clustered execution records explicit per-member checkpoint ownership instead
  of flattening member state into one anonymous snapshot.

## Projection rules

- When the resulting run or step fails or blocks, the normal operator output
  must include the latest checkpoint identity and a restore hint.
- Checkpoint creation must not imply that Canon or compatibility routing owns
  the checkpoint lifecycle.