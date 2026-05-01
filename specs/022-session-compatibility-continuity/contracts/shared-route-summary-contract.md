# Contract: Shared Route Summary Surface

**Feature**: 022-session-compatibility-continuity  
**Date**: 2026-05-01

## Purpose

Define which summary concepts must use aligned wording across native and compatibility follow-up surfaces.

## Shared Summary Guarantees

- Routing, execution condition, terminal or recovery condition, adaptive summary, review summary, and governance summary must use aligned operator-facing wording where the concepts overlap.
- Shared wording must not remove route attribution; native and compatibility outputs must still name which route actually ran.
- Missing route-specific concepts must be omitted rather than guessed from unrelated task or trace state.

## Explicit Boundaries

- Summary alignment does not mean the underlying routes share the same resumability or ownership model.
- Compatibility summaries must not imply workflow-owned or session-native execution when the route remained explicit compatibility.