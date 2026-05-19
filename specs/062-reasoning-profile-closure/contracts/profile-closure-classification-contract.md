# Contract: Profile Closure Classification

## Purpose

Define the authoritative shipped-status classification for the first-wave
reasoning capabilities closed by `062-reasoning-profile-closure`.

## Classification Table

| Capability | Final Classification | Runtime Evidence Required | Operator Claim Boundary |
|------------|----------------------|---------------------------|-------------------------|
| `bounded_self_consistency` | `shipped_profile` | Existing `061` runtime evidence remains valid | May be described as a shipped concrete profile |
| `independent_pair_review` | `shipped_profile` | Must have positive-path runtime evidence and bounded non-success handling | May be described as a shipped concrete profile |
| `heterogeneous_security_review` | `shipped_profile` | Must have dedicated runtime activation, inspect, trace, and confidence evidence | May be described as a shipped concrete profile |
| `bounded_reflexion` | `shipped_profile` | Must have real runtime activation and bounded interruption or exhaustion handling | May be described as a shipped concrete profile |
| `debate` | `bounded_substrate` | Runtime evidence is optional and only as supporting bounded substrate | MUST NOT be described as a shipped standalone profile |
| `adjudication` | `shared_primitive` | Runtime evidence is optional and only as a shared disagreement-resolution primitive | MUST NOT be described as a shipped standalone profile |

## Contract Rules

- A capability classified as `shipped_profile` MUST have a concrete profile id,
  one representative session-native runtime story, one operator-visible
  projection story, and one aligned trace story.
- A capability classified as `bounded_substrate` MAY appear in trace or outcome
  vocabulary as bounded supporting behavior but MUST NOT be presented as a
  standalone shipped profile id.
- A capability classified as `shared_primitive` MAY appear in trace or outcome
  vocabulary as shared disagreement-resolution behavior but MUST NOT be
  presented as a standalone shipped profile id.
- Roadmap, changelog, validation, and documentation surfaces MUST use this same
  classification language.
- Any future promotion of a shared primitive to a standalone profile requires a
  new explicit feature spec rather than silent documentation drift.