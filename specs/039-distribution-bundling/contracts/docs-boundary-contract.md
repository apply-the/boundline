# Contract: Documentation Boundary

**Feature**: 039-distribution-bundling  
**Date**: 2026-05-03

## Purpose

Define the product-documentation split between fast onboarding and advanced
architecture while keeping the Boundline versus Canon boundary explicit.

## Required Surface

- README must lead with a quick path that covers install, verification,
  initialization, and the first bounded run.
- `docs/getting-started.md` or its linked advanced companion must explain the
  session-native model, the source-install fallback, and when Canon matters.
- Advanced docs must state clearly that Boundline owns orchestration and delivery
  state while Canon remains a bounded governance companion.
- Assistant-facing docs must align with the same distribution and product
  boundary story.

## Explicit Boundaries

- The quick path must not require readers to absorb architecture detail before
  they can run the tool.
- The advanced layer must not blur Boundline into a Canon wrapper or hosted
  service.
- Docs must not imply official bundled support for channels outside Homebrew
  and winget in this slice.