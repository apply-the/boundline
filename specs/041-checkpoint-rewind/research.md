# Research: Checkpoint Rewind

## Decision 1: Refound the repository into three workspace members

- **Decision**: Convert the repository root into a Rust workspace with
  `crates/boundline-core`, `crates/boundline-adapters`, and
  `crates/boundline-cli`.
- **Rationale**: The roadmap explicitly ties checkpoint safety to the deferred
  Rust workspace migration. Checkpoint domain state, filesystem-backed stores,
  and command dispatch are now distinct enough that keeping them in one crate
  would continue to blur responsibilities.
- **Alternatives considered**:
  - Keep the single-crate layout and add checkpoint modules in place. Rejected
    because it would postpone the roadmap-mandated refoundation and keep I/O,
    orchestration, and CLI wiring coupled.
  - Split into more than three crates immediately. Rejected because the safety
    slice does not need finer-grained architecture to deliver user value.

## Decision 2: Keep checkpoint storage workspace-local under `.boundline/checkpoints/`

- **Decision**: Persist checkpoint manifests and captured file payloads under
  `.boundline/checkpoints/` in the owning workspace.
- **Rationale**: The feature must work in dirty and non-VCS repositories, so it
  cannot depend on Git internals or remote storage. Workspace-local persistence
  matches the existing session and trace ownership model.
- **Alternatives considered**:
  - Git-native rollback. Rejected because Git is optional and the roadmap marks
    Git-backed rollback out of scope for this slice.
  - Global checkpoint storage. Rejected because it would blur workspace
    authority and make cluster restore semantics harder to inspect.

## Decision 3: Define checkpoint scope from bounded runtime evidence

- **Decision**: Build each pre-mutation checkpoint from the bounded file set the
  runtime already knows about, primarily execution-profile `attempt.changes`
  paths, persisted `latest_changed_files`, and cluster member authority.
- **Rationale**: The current runtime already records changed-file evidence from
  bounded fixture attempts. Reusing that evidence avoids inventing a new broad
  repository snapshot heuristic and keeps checkpoint creation causal and
  inspectable.
- **Alternatives considered**:
  - Snapshot the entire workspace every time. Rejected because it is too broad,
    slower, and misaligned with Boundline's bounded-delivery model.
  - Infer changed files from post-mutation diffs only. Rejected because the
    checkpoint must exist before the mutation starts.

## Decision 4: Make restore safety-first with explicit override

- **Decision**: `checkpoint restore <id>` refuses by default when unrelated
  newer edits would be overwritten, and only proceeds when the operator passes
  an explicit `--force` override.
- **Rationale**: Checkpoints are a safety feature. Silent overwrite would
  undermine the entire slice. Explicit refusal plus override keeps the operator
  in control.
- **Alternatives considered**:
  - Always restore regardless of newer edits. Rejected because it risks silent
    data loss.
  - Require interactive confirmation. Rejected because the CLI already models
    explicit flags for bounded operator intent and tests need deterministic
    behavior.

## Decision 5: Keep trace history append-only and record restore as an event

- **Decision**: Never delete or rewrite existing trace files during restore;
  instead record restore attempts and outcomes as new inspectable state.
- **Rationale**: Boundline trace history is the execution audit surface.
  Rewriting it during recovery would make failure analysis harder and violate
  the append-only inspection story.
- **Alternatives considered**:
  - Delete post-checkpoint traces on successful restore. Rejected because it
    hides evidence about what actually happened.
  - Keep restore state only in checkpoint manifests. Rejected because operators
    also need restore visibility from trace-driven surfaces like `inspect`.

## Decision 6: Add checkpoint commands as a top-level CLI group

- **Decision**: Add `boundline checkpoint list` and
  `boundline checkpoint restore <id>` as first-class commands under the main CLI
  command surface.
- **Rationale**: The roadmap explicitly names these commands, and they should be
  discoverable without overloading `status` or `inspect` into mutation tools.
- **Alternatives considered**:
  - Hide restore behind `inspect` or `status`. Rejected because operators need a
    direct recovery command, not only a read-side surface.

## Decision 7: Keep docs split into quick path and advanced architecture

- **Decision**: Preserve the lightweight README quick path and move routing,
  cluster, delegation, workspace-architecture, and Canon details into the
  advanced architecture layer.
- **Rationale**: The user's feedback is that Boundline now has the right product
  boundary but risks feeling intimidating. Checkpoint safety should strengthen
  the quick path rather than repacking advanced concepts back into it.
- **Alternatives considered**:
  - Document everything in the README. Rejected because that is exactly the
    density problem the feedback identifies.

## Decision 8: Treat the release closeout as part of the feature

- **Decision**: Keep the feature incomplete until version bump, docs, roadmap,
  changelog, workspace validation, clippy, formatting, and touched-file
  coverage are all complete.
- **Rationale**: The request is explicitly for feature-complete delivery rather
  than a code-only checkpoint slice.
- **Alternatives considered**:
  - Defer release-surface work until after the runtime lands. Rejected because
    the workspace refoundation changes the repo and command structure that the
    docs and release metadata must describe.