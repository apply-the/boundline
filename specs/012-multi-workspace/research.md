# Research: Multi-Workspace Orchestration

## Decision 1: Store cluster metadata in the primary workspace under `.synod/cluster.toml`

- Decision: Persist cluster identity, primary workspace, member list, and cluster-scoped defaults in a new `.synod/cluster.toml` file located in one designated primary workspace.
- Rationale: The feature needs one inspectable source of truth for cluster membership and defaults, but it must remain independent from Canon and avoid introducing a new repository-level control plane. Reusing `.synod/` keeps the operator mental model aligned with existing Synod workspace state.
- Alternatives considered:
  - Store cluster metadata in every member workspace: rejected because it creates drift and reconciliation problems immediately.
  - Store cluster metadata under `.canon/`: rejected because the constitution forbids making core Synod behavior depend on Canon surfaces.
  - Store cluster metadata only in memory during one command: rejected because later status, inspect, and config flows need persistent cluster context.

## Decision 2: Model clustered execution as a projection over existing per-workspace session and trace state

- Decision: Keep per-workspace session and trace files authoritative for their local repository, and add a cluster projection that reads member state, records the active cluster identity in the primary session, and aggregates the latest member status and trace references.
- Rationale: This yields immediate delivery value without inventing a distributed session engine. It also preserves the existing single-workspace execution model and minimizes risk to proven orchestration logic.
- Alternatives considered:
  - Replace per-workspace sessions with one shared distributed session file: rejected because it broadens the feature into distributed control flow and weakens the local workspace contract.
  - Ignore existing member state and show only cluster metadata: rejected because the feature would not actually improve operator insight.
  - Mirror the full session file into every member workspace: rejected because it adds synchronization complexity without first-slice value.

## Decision 3: Introduce a dedicated `synod cluster` command surface and only extend existing commands where cluster context is required

- Decision: Add `synod cluster init|status|inspect` as the explicit operator entry point for cluster behavior, and extend existing session/config flows only where a cluster-aware projection or precedence resolution is required.
- Rationale: Cluster behavior is a new operator concern and should not be hidden inside unrelated commands. A dedicated surface also keeps single-workspace commands understandable and bounded.
- Alternatives considered:
  - Overload `synod start`, `synod status`, and `synod inspect` with cluster behavior only: rejected because the user would have no clear bootstrap path for cluster membership.
  - Create a separate binary for clustered orchestration: rejected because the capability belongs inside Synod rather than behind a second tool.
  - Require manual file editing for cluster setup: rejected because it recreates the usability failure that `synod init` already fixed for single workspaces.

## Decision 4: Insert cluster scope between workspace-local and user-global config precedence

- Decision: Extend effective config resolution to `CLI > workspace > cluster > global > built-in`, and expose the resolved source for every effective value when cluster-aware config is inspected.
- Rationale: Operators need shared defaults across related repositories, but local repository constraints still need to win. Explicit source attribution prevents cluster config from becoming hidden intelligence.
- Alternatives considered:
  - Make cluster config override local workspace config: rejected because it removes the operator’s most specific control surface.
  - Merge cluster config silently into workspace files: rejected because it hides the new source layer and makes diagnosis harder.
  - Skip cluster-scoped config in the first slice: rejected because the clustered feature would have little operational value without inherited defaults.

## Decision 5: Aggregate cluster status and inspection explicitly, including missing or mismatched member state

- Decision: Cluster status and cluster inspection should enumerate every member and classify it explicitly as healthy, missing session, missing trace, blocked, or mismatched with the active cluster identity.
- Rationale: The constitution requires inspectability and explicit failure paths. Multi-workspace behavior becomes untrustworthy if gaps are collapsed into generic success.
- Alternatives considered:
  - Show only healthy members and omit missing ones: rejected because it hides the most important operator action items.
  - Fail the whole command on the first missing member: rejected because the operator still needs the rest of the cluster picture.
  - Return raw file paths without classification: rejected because it pushes diagnosis back onto the operator.

## Decision 6: Keep automatic cross-repository plan fan-out and distributed execution out of scope for the first slice

- Decision: The first slice stops at cluster bootstrap, shared context projection, aggregated inspection, and inherited defaults. Automatic task distribution, multi-repository plan generation, and remote or parallel execution remain future work.
- Rationale: This preserves a minimal independently valuable capability that can be delivered within the existing sequential execution model.
- Alternatives considered:
  - Include cross-repository plan generation now: rejected because it would require a much larger orchestration change than this slice can validate credibly.
  - Include parallel fan-out across member workspaces: rejected because the constitution still requires sequential-first design for initial slices.
  - Defer clustered work entirely until distributed execution is ready: rejected because operators still gain immediate value from explicit cluster bootstrap and inspection.