# S9 - Large Codebase Context Substrate

## Owner

Boundline

## Status

Required before stronger autonomy

## Speckit Seed Notes

- Seed role: repository-scale safety substrate for context selection and edits.
- First slice: refuse unsafe huge full-file reads, add paged reads with stable
  digests, and show omitted context plus skip reasons in `inspect`.
- Depends on: current local context-intelligence baseline and trace/status
  projection surfaces.
- De-duplication: sqlite-vec activation belongs to seed 01; provider-supplied
  context belongs to seed 07; this seed owns local large-repository behavior.

## Strategic Role

This feature makes Boundline credible on real repositories.

A governed delivery runtime that cannot handle huge repositories and huge files safely becomes a toy governance wrapper.

## Problem

Large codebases break naive AI workflows because they create:

- context-window overflow
- irrelevant token flooding
- full-file overreads
- hidden impact surfaces
- unsafe full-file rewrites
- missed callers, tests, schemas, and contracts
- hallucinated understanding from partial context
- expensive and slow planning

## Core Scope

### Must Cover

- Search-before-read policy
- Hard read limits
- Paged file reads
- Stable file digests
- Snippet references
- Symbol-aware indexing
- Repository map
- Context Pack budgeting
- Lazy hash references for huge logs, diffs, CI output, generated files
- Hybrid retrieval ranking
- Test/evidence-guided retrieval
- Patch-safe editing
- Trace-visible context selection
- Large-file risk findings

## Algorithms And Techniques

### Search-Before-Read

Before reading a large file, Boundline should require one or more of:

- ripgrep query
- file path search
- symbol search
- import/export search
- test relation lookup
- previous trace relation lookup

### Symbol-Aware Indexing

Prefer semantic units over arbitrary token chunks:

- functions
- methods
- classes
- structs
- traits/interfaces
- modules
- routes
- schemas
- tests
- migrations
- config blocks

### Repository Map

Store a compact navigation model:

```text
file -> symbols
symbol -> location
file -> imports/exports
symbol -> callers/callees where available
symbol -> tests where available
file -> Canon refs where available
file -> owner hints where available
```

### Context Pack Budgeting

Every included context item must carry:

- reason
- source
- authority
- token or byte estimate
- lifecycle relevance
- risk relevance
- whether full text, summary, signature, or snippet was included

### Hybrid Ranking

Rank context candidates with multiple signals:

- lexical score
- symbol match
- graph distance
- dependency relation
- test relation
- recent trace relation
- changed-file proximity
- Canon authority
- guidance/guardian relevance
- optional vector similarity

### Lazy Hash References

For huge logs and diffs:

```text
store full artifact by digest
include digest + summary + relevant excerpt
resolve full content only on demand
```

### Patch-Safe Editing

For huge files:

- never rewrite full file by default
- use anchored hunks
- verify anchors before apply
- verify file digest expectations
- re-run formatter or parser
- emit trace for applied hunks
- fall back to manual review if anchors drift

## Suggested Technology

Start local and embedded:

- SQLite
- SQLite FTS5
- stable JSON sidecar index files
- ripgrep integration
- tree-sitter where language support is practical
- LSP integration later where available

Then add optional:

- sqlite-vec for local vector retrieval
- optional vector provider interface
- optional graph export
- Kùzu later for local graph queries if SQLite relationship tables become insufficient

Avoid as mandatory V1 dependencies:

- Neo4j
- Qdrant
- remote vector databases
- mandatory cloud indexing

## Data Model Sketch

```text
files(id, path, digest, size, language, modified_at)
symbols(id, file_id, kind, name, start_line, end_line, signature, digest)
relations(source_id, target_id, kind, confidence)
snippets(id, file_id, start_line, end_line, digest, summary)
context_pack_items(id, session_id, source_ref, inclusion_reason, budget_cost)
```

## Acceptance Criteria

- Boundline refuses huge full-file reads unless explicitly allowed.
- Boundline can build a repo map for common language stacks.
- Context Packs show why items were selected.
- Large logs and diffs are compacted with hash references.
- Patch application uses anchors and post-apply verification.
- Inspect can show omitted context and why it was omitted.
- Tests cover huge file, huge diff, and missing index cases.

## Risks

- Overbuilding the index before proving retrieval quality.
- Treating vector search as source of truth.
- Hiding important code behind summaries.
- Slowing down every session.

## Hard Rule

Summaries are for navigation. Source code is required for edits.
