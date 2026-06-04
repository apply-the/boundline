# Data Model: Backlog Contract

## Entity: Backlog Quality Assessment

Represents the effective backlog-readiness decision for one active planning
session.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `state` | `ready`, `clarification_required`, or `blocked` | Yes | `ready` only when the packet is credible enough for execution admission; `clarification_required` only for recoverable omissions in an otherwise credible full packet; `blocked` for unsafe or closure-limited packets. |
| `findings` | Ordered list of stable labels | No | Labels remain concise and machine-readable. Ordering determines the one current question, if any. |
| `task_count` | Positive integer | No | Counts the validated delivery slices or downstream-ready backlog entries that Boundline can admit for later work. |
| `mvp_scope` | Short string | No | Names the first independently deliverable slice exposed by the packet. |
| `unmapped_items` | Ordered list of labels or slice identifiers | No | Lists backlog elements that could not be traced to available goal, plan, or acceptance evidence. |

## Entity: Backlog Packet Evidence Window

Defines the Canon `0.67.0` evidence that Boundline may inspect without changing
the producer contract.

### Full Packet

Expected Canon backlog artifacts:

- `backlog-overview.md`
- `epic-tree.md`
- `capability-to-epic-map.md`
- `dependency-map.md`
- `delivery-slices.md`
- `sequencing-plan.md`
- `acceptance-anchors.md`
- `planning-risks.md`
- optional `execution-handoff.md`

### Risk-Only Packet

Closure-limited Canon backlog output:

- `backlog-overview.md`
- `planning-risks.md`

**Validation Rule**: A risk-only packet is never execution-ready for Boundline
and maps to `blocked`.

## Entity: Backlog Slice Evidence

Represents the smallest backlog item Boundline may treat as execution-ready
source material in this slice.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `slice_id` | Stable identifier | Yes for ready packets | Must be unique within the packet and referenceable from sequencing or handoff content. |
| `scope_label` | Short title or slice name | Yes | Must remain above fine-grained implementation-task detail. |
| `implementation_refs` | One or more file paths or artifact refs | Yes for admitted slice | Must point to explicit downstream implementation surfaces rather than vague intent only. |
| `verification_anchors` | One or more independent proof markers | Yes for admitted slice | Must show how the slice can be validated independently at downstream execution time. |
| `handoff_ref` | `execution-handoff.md` reference | Yes for admitted slice | Marks the packet as downstream-ready instead of closure-limited. |

## Entity: Backlog Quality Finding

Represents one missing, invalid, or weak backlog signal.

| Label family | Initial-slice behavior | Recovery |
|---|---|---|
| `backlog_packet_pending` | Clarification-required until a reusable full packet exists | Wait for Canon backlog completion or rerun. |
| `backlog_packet_not_reusable` | Blocked | Resolve Canon closure or producer failure first. |
| `missing_backlog_document` | Clarification-required or blocked depending on whether the packet is merely incomplete or closure-limited | Restore the missing artifact or rerun Canon. |
| `missing_stable_slice_ids` | Blocked | Boundline cannot safely link readiness without stable slice identity. |
| `missing_execution_handoff` | Clarification-required for an otherwise credible full packet | Surface or regenerate the governed handoff artifact. |
| `missing_implementation_refs` | Clarification-required or blocked depending on whether the admitted slice remains identifiable | Surface explicit downstream file paths or artifact refs. |
| `missing_independent_verification_anchors` | Clarification-required | Supply or regenerate the downstream-ready proof anchors. |
| `unmapped_item:*` | Non-blocking visibility unless it prevents the admitted slice from being credible | Preserve the unmapped label instead of inventing traceability. |

## Entity: Backlog Clarification Request

Reuses the existing structured `phase_request` handoff already used by other
planning gates.

| Field | Purpose |
|---|---|
| `request_id` | Stable resume identity for the current backlog question. |
| `kind` | Clarification classification understood by assistant hosts. |
| `reason` | Concise explanation of why backlog quality cannot advance. |
| `question` | Exactly one operator question for the highest-priority recoverable finding. |
| `expected_answer` | Existing answer contract used by host surfaces. |
| `resume_command` | Raw CLI continuation that resumes the same session. |
| `assistant_resume_command` | Host-safe assistant continuation when available. |
