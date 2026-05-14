# Canon Project Memory Consumer Contract

- **Consumer**: Boundline
- **Canonical Source Of Truth**: Canon stable owner-side contract at
  `docs/integration/project-memory-promotion-contract.md`
- **Canonical Source Identifier**: `canon:docs/integration/project-memory-promotion-contract.md`
- **Supported Contract Line**: V1
- **Supported Compatibility Window**: additive `v1.x` owner-side revisions only

## Boundline Consumes

- producer-neutral managed block markers
- required V1 lineage fields
- promotion state as a Canon-owned fact
- governed stage refs, promotion events, and evidence refs
- stable `docs/project/` and `docs/evidence/` target categories

## Boundline Does Not Own

- Canon promotion policy
- Canon lineage generation
- Canon publish-profile definitions
- Canon write rules for Canon-produced content

## Consumer Rules

- Shared managed blocks use the Canon-owned marker shape:

  ```md
  <!-- project-memory:managed:start producer="canon|boundline" source_ref="..." contract_version="v1" -->
  ...
  <!-- project-memory:managed:end -->
  ```

- Boundline-owned blocks may set `producer`, `source_ref`, `status`, `summary`,
  and other consumer-local evidence text for `producer="boundline"` sections.
- Boundline MUST NOT set or override Canon-owned semantics such as
  `promotion_state`, `promotion_profile`, `approval_state`,
  `packet_readiness`, `update_strategy`, or Canon stable-target routing.
- Reject unknown major contract lines.
- Accept additive V1-compatible fields without redefining Canon semantics.
- Surface Canon producer facts in planning and inspection, then apply Boundline
  consumer policy for continue, warning, or hard-stop outcomes.