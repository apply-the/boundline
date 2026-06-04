# Session Memory And Repository Knowledge Distillation

## Integration Update

This roadmap item must stay separate from:

- Persistent Context Snapshot Cache from `06-large-codebase-context-substrate.md`
- Trace Compaction Policy from `08-evals-and-runtime-observability.md`

Cache is not memory.

Trace compaction is not memory.

Memory is reviewed, accepted, trace-linked knowledge that may influence future work.

## Relationship To Other Roadmap Files

| Related file | Relationship |
|---|---|
| `06-large-codebase-context-substrate.md` | May provide local cache and repo map, but those are not memory |
| `08-evals-and-runtime-observability.md` | Provides trace refs and compaction, but compaction summaries are not memory |
| `18-completion-verification-runtime.md` | May produce evidence refs that support future memory proposals |
| `19-plan-execution-orchestration.md` | May generate repeated operational patterns worth proposing as memory |

## Explicit Non-Memory

### Persistent Context Cache

Examples:

- repo map
- active spec snapshot
- adapter capability cache
- retrieval index state
- last planning context

Reason: derived and rebuildable.

### Trace Compaction Summaries

Examples:

- summarized assistant transcript
- compacted command output
- archived trace fragments

Reason: trace hygiene, not reusable knowledge.

### Raw Logs

Examples:

- test logs
- build logs
- debug dumps

Reason: evidence or diagnostics, not memory.

## Memory Types

### Operational Memory

Examples:

- build command
- test command
- formatting command
- local setup caveat

### Repository Convention

Examples:

- architecture pattern
- folder ownership
- test fixture style
- API error convention

### Known Pitfall

Examples:

- flaky test condition
- migration hazard
- generated file warning

### Candidate Canon Knowledge

Examples:

- domain term
- invariant
- architecture decision
- project standard

## Memory Protocol Reminder

Before saving memory:

1. summarize proposed memory
2. cite trace or evidence source
3. ask for confirmation unless policy allows auto-capture
4. classify memory type
5. set authority and expiry/review status
6. redact secrets
7. store with stable ID

## Acceptance Criteria Additions

- Cache entries are not promoted to memory without source evidence and review.
- Trace summaries are not treated as memory.
- Raw logs are not saved as memory.
- Memory entries cite trace references.
- Inspect shows memory provenance and authority.
- Stale memory can be marked deprecated.

## Risks

- Memory becomes stale.
- Incorrect heuristics become truth.
- Users skip review.
- Memory overlaps poorly with Canon project memory.
- Cache or trace summaries are mistaken for governed knowledge.

## Hard Rules

- Memory is not governance. Canon governs knowledge.
- Cache is not memory.
- Trace compaction is not memory.
