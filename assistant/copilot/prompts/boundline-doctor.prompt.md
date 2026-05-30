---
description: "Verify Boundline and Canon installation readiness"
handoffs:
  - label: Set Goal
    agent: boundline-goal
    prompt: Define the active session goal
  - label: Check Status
    agent: boundline-status
    prompt: Show current session status
---

# Command: /boundline-doctor

Shared guidance: `assistant/README.md`

Run `boundline doctor --install`. Summarize `canon_path`, `canon_governance_surface`, `canon_modes`, `canon_companion`, `companion_state`, and repair actions. Treat failed Canon surface checks as blockers before governed work starts.
