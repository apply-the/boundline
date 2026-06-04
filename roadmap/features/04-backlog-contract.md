# Boundline Backlog Contract

## Summary

Turn Canon backlog output into an explicit Boundline planning-readiness gate.
This slice does not add a `/boundline-tasks` command. Canon remains the
governed backlog producer, and Boundline decides whether the emitted backlog
packet is credible enough to approach execution.

## Delivered Slice

Released in `0.69.0`.

The shipped slice:

- validates the Canon multi-document backlog packet instead of the legacy
  checklist-style `backlog.md` heuristic
- blocks closure-limited backlog packets that contain only overview plus risks
- uses `clarification_required` when a full packet still lacks governed
  execution-handoff evidence
- preserves `backlog_quality_state`, findings, task count, MVP scope, and
  unmapped items across status, inspect, orchestration, traces, and assistant
  plan or run surfaces

## Boundaries

- No `/boundline-tasks` command
- No Canon schema ownership inside Boundline
- No hidden fallback sequencing or execution synthesis
- No replacement for the later plan-analysis or completion-proof slices
