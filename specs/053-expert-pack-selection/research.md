# Research: Expert Pack Selection

## Current Implementation Surfaces

- `src/domain/domain_templates.rs` already provides built-in domain families, human-facing summaries, and bounded domain detection from repository cues.
- `src/domain/configuration.rs` already resolves effective domain-template and reviewer-role configuration across workspace, cluster, and global scopes.
- `src/orchestrator/goal_planner.rs` already turns bounded targets into domain-context outcomes and `ContextPack` inputs before planning continues.
- `src/domain/goal_plan.rs` plus session and trace projection surfaces already persist and render operator-visible planning context.
- Current planning already accepts broader Canon signals, but this slice narrows
	the supported Canon contribution to governed expertise inputs discovered from
	compatible publication and lineage surfaces.

## Boundaries Confirmed During Planning

- Expert-pack selection should stay local-first and must remain useful without Canon.
- Existing reviewer roles are the correct runtime-role recommendation surface for this slice; the feature should not invent a second routing system.
- Existing effective config precedence is the correct owner for enablement, suppression, and override behavior.
- External expert-pack installation is deferred; the first slice should model only built-in packs with stable identifiers.

## Implementation Direction

- Add stable built-in expert-pack definitions close to current domain-family and routing context so selection rules remain local and inspectable.
- Compute expert-pack selection alongside existing domain-context resolution in goal planning rather than in a second runtime layer.
- Persist selected packs, rejected candidates, and supporting signals in the goal plan so `status`, `next`, and `inspect` can project the same result.
- Treat Canon expertise inputs as optional supporting signals that can confirm a
	compatible candidate or be ignored with an explicit reason, but never choose
	runtime roles directly or suppress a locally credible candidate.

## Provider-Doc Audit

- Reviewed current OpenAI, Anthropic, and Google model documentation on
	2026-05-14 against `assistant/catalog/model-catalog.toml`.
- No catalog changes are required for this slice.
- The bundled catalog already carries the current coding-relevant OpenAI IDs
	(`gpt-5.5`, `gpt-5.4`, `gpt-5.4-mini`, `gpt-5.4-nano`), Google Gemini IDs
	(`gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite`,
	`gemini-3.1-pro-preview`, `gemini-3.1-flash-lite`), and the Boundline
	adapter IDs for the currently documented Anthropic families (`opus-4.7`,
	`sonnet-4.6`, `haiku-4.5`).
- Newly documented audio, media, and research-specific provider models do not
	change the expert-pack-selection contract for this bounded slice.

## Likely Touchpoints

- `src/domain/domain_templates.rs`
- `src/domain/configuration.rs`
- `src/domain/goal_plan.rs`
- `src/orchestrator/goal_planner.rs`
- `src/cli/session.rs`
- `src/cli/output.rs`
