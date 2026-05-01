# Contract: Adaptive Selection Evidence

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## Purpose

Define the persisted adaptive evidence required when validation failure changes the next bounded repair attempt.

## Required Evidence Shape

- Selection evidence must preserve goal terms, validation terms, and an explicit validation-guided rationale for the chosen bounded candidate.
- Attempt lineage must preserve the previous attempt identifier, current attempt identifier, transition kind, and the reason the new attempt became credible.
- Workspace-slice summaries must remain bounded to selected targets and scored candidates; they must not expand to arbitrary repository search.

## Required Behavioral Guarantees

- Validation-guided evidence must come only from the latest validation record and active failure context.
- Candidate-signature tracking must still prevent repeated failed bounded attempts unless a new credible reason is visible.
- When no new bounded candidate remains, the evidence must explain the terminal stop instead of disappearing from the final state.