# Contract: Bundled Catalog Refresh

## Scope

The repository-managed bundled catalog used by `init`, config defaults, and assistant-facing route guidance.

## Reviewed Public Sources

- OpenAI models documentation
- Anthropic Claude models overview
- Google Gemini API models documentation

## Required Behavior

- The bundled catalog must contain the currently documented mainstream route-capable models for the supported runtimes when those models are credible for planning, implementation, verification, review, or adjudication.
- The bundled catalog must exclude non-route model families from the standard picker.
- Catalog refresh work must record either an applied delta or an explicit no-change result in the feature artifacts.
- Built-in default routes must stay aligned with the refreshed catalog entries.

## Evidence

- Updated `assistant/catalog/model-catalog.toml`
- Matching built-in route defaults in `src/domain/configuration.rs`
- Release notes or feature research documenting the applied delta or no-change outcome