# AI Gateway And Inference Economics

## Owner

Boundline

## Status

B-level, after evals and provider protocol

## Speckit Seed Notes

- Seed role: route economics and health policy after provider and eval
  primitives exist.
- First slice: record route latency/cost telemetry and enforce one session cost
  budget without changing model-selection defaults.
- Depends on: provider capability metadata from seed 07 and event schema from
  seed 08.
- De-duplication: provider metadata belongs to the provider protocol, trace
  schema belongs to evals/observability, and this seed owns budget and route
  decision policy only.

## Strategic Role

This feature keeps governed delivery affordable, reliable, and provider-agnostic.

Councils, guardians, large-context reasoning, and provider calls can become expensive unless routing is proportional to risk.

## Problem

Without inference economics:

- simple checks may use expensive models
- councils may overrun cost budgets
- prompt/context caching is missed
- provider outages break runs
- local/private models are hard to use
- token costs are invisible
- model upgrades are uncontrolled

## Core Scope

- AI gateway compatibility
- provider route health
- model readiness
- cost budgets per run/session
- prompt/context caching hooks
- tiered model routing
- fallback policy
- local provider support
- model capability metadata
- LLM call telemetry
- eval-gated route changes

## Routing Tiers

Example:

### Tier 0

Deterministic tool, no LLM.

### Tier 1

Small/cheap model for summarization, extraction, simple classification.

### Tier 2

Balanced model for planning, review, guardian reasoning.

### Tier 3

High-capability model for architecture, red-zone review, complex synthesis.

## Routing Inputs

- lifecycle phase
- authority zone
- risk
- context size
- required reasoning depth
- privacy requirement
- latency budget
- cost budget
- provider health
- eval performance

## Suggested Technology

Support compatibility with:

- LiteLLM-style gateway
- OpenAI-compatible APIs
- Anthropic-compatible APIs
- Gemini-compatible APIs
- Ollama or local OpenAI-compatible endpoints

Do not require a gateway in V1.

## Cost And Telemetry

Record:

- provider
- model
- route
- token input/output if available
- estimated cost
- latency
- cache hit/miss
- failure reason
- fallback route

## Acceptance Criteria

- Boundline can enforce a session cost budget.
- Boundline can choose route by task class and risk.
- Boundline can degrade clearly when preferred route unavailable.
- Local model route can be configured.
- Route choices appear in trace and S8.
- Route changes can be eval-gated.

## Risks

- Routing logic becomes too clever.
- Cheap models produce weak governance.
- Cost controls hide quality problems.
- Provider abstraction leaks.

## Hard Rule

Cost optimization must never silently weaken red-zone governance.
