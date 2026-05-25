# Boundline Goal Template

Use this template before `/boundline:goal` when the goal is still broad or when the operator needs a brief that planning can reason over without guesswork.

## Goal

- What are we trying to change?

## Intended Outcome

- What should be true when this is done?

## Problem Domain

- What workflow, product area, or operator problem does this affect?

## Known Facts

- What is already known and verified?

## API Operations

- Which commands, endpoints, jobs, or user actions are in scope?

## Persistence Choice

- Which data store or persistence boundary is authoritative?

## Auth Boundary

- What authentication or authorization boundary matters here?

## Role Model Semantics

- Which actors can do what, and what must stay forbidden?

## Validation Target

- Which focused check proves the change is acceptable?

## Constraints

- What must stay bounded or unchanged?

## Unknowns

- Which facts are still missing?

## Assumptions

- Which assumptions are acceptable for now and should be called out?

## Brief Files

- List any repo files or external documents that should be passed with `--brief <path>`.

When this is filled, record it with `boundline orchestrate --goal "<goal>" --brief <path> --until phase-request --json-stream`.
