# Data Model: Advanced Context Intelligence

## 1. RetrievalIndexManifest

**Purpose**: Represents the workspace-local retrieval index that augments the
existing runtime-intelligence substrate.

**Key Fields**:

- `index_id`: stable identifier for the current workspace index
- `workspace_root`: canonical workspace path the index belongs to
- `schema_line`: compatibility line for the retrieval schema
- `mode_capabilities`: supported retrieval modes for this workspace
- `build_state`: `ready`, `stale`, `building`, or `insufficient`
- `authority_sources`: structured inputs the index is allowed to consume
- `last_refresh_trace_id`: latest trace that materially refreshed the index
- `last_refresh_reason`: why the most recent refresh was performed

**Relationships**:

- owns many `RetrievedEvidenceCandidate` records
- is referenced by many `RetrievalQuery` records
- may reference Canon artifacts only through compatible producer metadata

**Validation Rules**:

- `workspace_root` must match the active workspace using the index
- `build_state = ready` requires a successful refresh and compatible schema
- Canon-backed entries are invalid if required producer metadata is missing or
  incompatible

## 2. RetrievalQuery

**Purpose**: Represents one bounded request for additional context at a single
decision point.

**Key Fields**:

- `query_id`: stable query identifier
- `origin_command`: `plan`, `run`, `status`, `next`, or `inspect`
- `goal_ref`: active session or task reference that motivated the query
- `retrieval_mode`: `disabled`, `local`, or `remote`
- `risk_posture`: bounded risk classification used to size evidence depth
- `requested_scope`: sources eligible for retrieval in this query
- `refinement_budget`: maximum allowed refinement passes
- `refresh_budget`: maximum allowed stale-refresh retries
- `terminal_state`: `selected`, `degraded`, `insufficient`, or `exhausted`
- `terminal_reason`: surfaced reason for the final outcome

**Relationships**:

- reads from one `RetrievalIndexManifest`
- selects or rejects many `RetrievedEvidenceCandidate` records
- may create many `RelationshipProjection` and `ImpactAnalysisFinding` records

**Validation Rules**:

- only one `RetrievalQuery` may be active per decision point
- `retrieval_mode = remote` is valid only when explicit workspace policy allows it
- `terminal_state = selected` requires at least one explainable selected
  evidence candidate

**State Transitions**:

- `pending -> running -> selected`
- `pending -> running -> degraded`
- `pending -> running -> insufficient`
- `pending -> running -> exhausted`

## 3. RetrievedEvidenceCandidate

**Purpose**: Represents one repository or Canon-backed artifact considered for
context expansion.

**Key Fields**:

- `candidate_id`: stable identifier for the evidence candidate
- `source_kind`: `workspace_file`, `project_memory`, `trace`,
  `review_finding`, `verification_evidence`, or `canon_artifact`
- `source_ref`: path or canonical reference to the source artifact
- `authority_rank`: `structured`, `canon`, `workspace_override`, or `semantic`
- `selection_state`: `discovered`, `selected`, `downgraded`, `rejected`, or `expired`
- `selection_reason`: why the candidate was kept, downgraded, or rejected
- `provenance_summary`: concise explanation of where the candidate came from
- `compatibility_state`: whether the source contract and metadata were usable
- `staleness_state`: whether the source changed after retrieval

**Relationships**:

- belongs to one `RetrievalQuery`
- may participate in many `RelationshipProjection` records
- may support many `ImpactAnalysisFinding` records

**Validation Rules**:

- a candidate cannot be `selected` without `selection_reason`
- `authority_rank = semantic` cannot override a conflicting `structured` or
  compatible Canon authority input
- `canon_artifact` candidates require a compatible contract line and producer
  attribution before selection

**State Transitions**:

- `discovered -> selected`
- `discovered -> downgraded`
- `discovered -> rejected`
- `selected -> expired`

## 4. RelationshipProjection

**Purpose**: Represents an explainable relationship inferred or confirmed from
retrieved evidence.

**Key Fields**:

- `relationship_id`: stable identifier
- `from_candidate_id`: evidence record that supports the relation
- `subject_ref`: system, domain, invariant, test, contract, reviewer, or risk
  target
- `relationship_kind`: typed relation such as `affects_domain`,
  `exercises_test`, `exposes_contract`, `suggests_reviewer`, or `supports_risk`
- `credibility_state`: `credible`, `tentative`, or `insufficient`
- `explanation`: human-readable reason the relation exists

**Relationships**:

- belongs to one `RetrievalQuery`
- references one or more `RetrievedEvidenceCandidate` records
- may feed one or more `ImpactAnalysisFinding` records

**Validation Rules**:

- a relationship cannot be projected without at least one supporting evidence candidate
- `credibility_state = credible` requires explicit explanation and provenance

## 5. ImpactAnalysisFinding

**Purpose**: Represents a delivery-relevant consequence surfaced from retrieved
evidence and projected relationships.

**Key Fields**:

- `finding_id`: stable identifier
- `finding_kind`: `affected_system`, `affected_domain`, `missing_test`,
  `contract_exposure`, `reviewer_gap`, or `evidence_gap`
- `subject_ref`: primary system, test, contract, or reviewer target
- `status`: `open`, `acknowledged`, `resolved`, or `invalidated`
- `severity_state`: bounded delivery severity used for prioritization
- `recommended_follow_up`: explicit next action surfaced to the operator
- `supporting_relationship_ids`: relationships backing the finding

**Relationships**:

- belongs to one `RetrievalQuery`
- is backed by one or more `RelationshipProjection` records
- may be persisted into session or trace state for later follow-through

**Validation Rules**:

- a finding cannot be `open` without a `recommended_follow_up`
- a finding cannot be projected when all supporting relationships are
  `insufficient`

**State Transitions**:

- `open -> acknowledged`
- `open -> resolved`
- `open -> invalidated`
- `acknowledged -> resolved`

## 6. RemoteTransmissionPolicy

**Purpose**: Represents the policy boundary for remote semantic expansion.

**Key Fields**:

- `policy_state`: `blocked`, `local_only`, or `remote_allowed`
- `policy_source`: workspace config, cluster config, or operator override
- `applies_to`: source classes that may or may not leave the local machine
- `blocked_reason`: why remote transmission is not permitted

**Validation Rules**:

- `policy_state = remote_allowed` requires explicit opt-in
- Canon-backed or local code artifacts must remain local when the policy is not
  `remote_allowed`

## Cross-Entity Invariants

- Structured runtime context remains authoritative over all `semantic`
  candidates and relationship projections.
- Incompatible Canon artifact metadata must be surfaced as unavailable or
  rejected, never partially merged into the authoritative context.
- Every selected candidate, relationship, and impact finding must be explainable
  from persisted provenance and selection rationale.
- Retrieval state must be reusable by later session-native commands and
  invalidatable when source files or artifacts change materially.