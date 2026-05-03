# Contract: Active Session Protection

## Purpose

Define how direct native run behaves when the workspace already contains
meaningful active session state.

## Contract

- Direct `run --goal` must not silently replace active captured, planned, or
  in-flight delivery state.
- When active session protection blocks bootstrap, Boundline must stop explicitly
  and tell the operator what action is needed next.
- Session safety must be preserved without hiding the existing session or
  inventing a merged delivery story.

## Required Visible Outcomes

- Operators do not lose active session context by accident.
- Blocked bootstrap outcomes remain inspectable and actionable.
- Existing session continuity stays authoritative until the operator resets or
  continues it explicitly.

## Boundary Conditions

- This contract does not require a multi-session manager.
- This contract does not auto-merge two goals into one session.
- This contract does not permit destructive reset without explicit operator
  intent.