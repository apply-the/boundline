---
description: "Diagnose missing workspace context for assistant follow-through"
---

# Command: /boundline-doctor-context

Shared guidance: `assistant/README.md`

Run `boundline doctor --workspace <workspace>`. Summarize `boundline_config`,
`canon_project_memory`, `expert_pack_inputs`, `provider_readiness`,
`advanced_context_index`, and `session_evidence`, then surface the CLI-reported
fix commands. Keep advisory gaps explicit and do not infer missing context from
chat history.