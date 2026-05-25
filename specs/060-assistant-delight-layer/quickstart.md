# S7 Assistant Delight Quickstart

**Version**: 0.60.0  
**Audience**: Boundline maintainers, assistant-pack authors, operators using S7 explanations  
**Purpose**: Exercise the implemented S7 runtime path in 10 minutes

---

## What ships now

Boundline 060 exposes S7 through the existing session-native runtime. There is
no separate explanation engine to bootstrap.

US1 surfaces:
- `/boundline:why`
- `/boundline:risk`
- `/boundline:evidence`
- `/boundline:next-best`

US2 surfaces:
- `/boundline:assumptions`
- `/boundline:hidden-impact`
- `/boundline:challenge`
- `/boundline:explain-plan`

All of them read the same authoritative data already exposed by `status` and
`inspect` against `.boundline/session.json` and `.boundline/traces/`.

Canonical rule:
- Boundline-owned runtime evidence, Canon-governed signals, and missing inputs
  must remain visibly separate.

---

## 1. Bootstrap A Session

```bash
cargo run --bin boundline -- start --workspace <workspace>
cargo run --bin boundline -- goal --workspace <workspace> --goal "explain the active bounded plan"
cargo run --bin boundline -- plan --workspace <workspace>
```

If the assistant host can run shell commands, prefer the same commands with
`--json` for `status` and `inspect`.

---

## 2. Read The First-Response S7 Surface

Use `status` for session-backed guidance and `inspect` for trace-backed detail.

```bash
cargo run --bin boundline -- status --workspace <workspace> --json
cargo run --bin boundline -- inspect --workspace <workspace> --json
```

The implemented US1 labels to preserve are:
- `why_summary`
- `risk_summary`
- `evidence_summary`
- `source_attribution`
- `fallback_disclosure`
- `confidence_level`
- `next_best_action`

Example zero-config output shape:

```text
why_summary: bounded runtime evidence points to the context router
risk_summary: Canon confirmation is missing, so risk remains bounded by runtime-only evidence.
source_attribution: runtime=session_state, authored_input; canon=none; missing=canon_input
fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only
next_best_action: boundline inspect
```

---

## 3. Use The Deeper Cognitive Lenses

US2 is backed by the same `status` and `inspect` surfaces. The assistant packs
now read these additional labels directly:

- `assumptions_summary`
- `assumption_group`
- `hidden_impact_summary`
- `hidden_impact_*`
- `hidden_impact_fallback_disclosure`
- `challenge_*`
- `explain_plan_*`

Example runtime-backed output shape:

```text
assumptions_summary: validation(1)
assumption_group: validation -> src/context_router.rs [explicit] source=workspace risk=low the matching test file names the same target
hidden_impact_summary: missing_tests(1)
hidden_impact_missing_tests: tests/context_router.rs [open/medium] add or refresh the focused regression test
challenge_strongest_objection: missing test coverage is still open for tests/context_router.rs
challenge_required_review: governance packet .canon/runs/canon-run-security remains authoritative
explain_plan_summary: goal=Plan with bounded context; stages=bug-fix/implement; risks=missing_tests(1); assumptions=validation(1)
```

---

## 4. Verify The Two Fallback Modes

### Canon Missing

When Canon-governed input is absent, S7 must stay useful and say so explicitly.

Expected wording:

```text
fallback_disclosure: Canon input not yet available; using Boundline runtime evidence only
```

### Advanced Context Semantic Fallback

When structured retrieval is available but semantic acceleration is not, S7
must disclose the fallback instead of pretending deeper inference succeeded.

Expected wording:

```text
hidden_impact_fallback_disclosure: higher-order impact inference is unavailable because semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval
```

---

## 5. Assistant Command Mapping

The host command packs now map as follows:

- `why`, `risk`, `evidence`, `assumptions`, `hidden-impact`, and `challenge`:
  prefer `inspect --json`
- `next-best` and `explain-plan`: prefer `status --json`

All host prompt assets preserve `next_command` as the authoritative follow-up.
When `challenge_required_review` or `challenge_council_required` appears,
assistant narration must keep that governance boundary explicit.

---

## 6. Validation Commands

Run these commands from the repository root to validate the implemented US1 and
US2 slice:

```bash
cargo test --test unit s7_
cargo test --test integration s7_
cargo test --test contract s7_
cargo test --test assistant_plugin_packages
```

---

## 7. Canon Boundary

Boundline 060 remains the consumer-side implementation. Canon 057 remains the
provider-side contract for governed input classes and degradation semantics.

What is true now:
- the implemented US1 and US2 surfaces consume existing Canon-backed session and
  trace fields when present
- no new Canon input class was required for this slice
- the independent Canon<->Boundline review is still a later gate after Canon
  task `T031`

## Troubleshooting

### "Why does S7 say 'missing evidence'?"

Canon input class you need has not been promoted yet. Options:
1. **Wait**: Is it in progress? (e.g., security scan running)
2. **Run it yourself**: Can you trigger the assessment? (e.g., `boundline security run`)
3. **Proceed anyway**: Accept lower confidence if evidence is optional
4. **Escalate**: If evidence is critical, contact governance authority

### "Why does S7 give different answers when I ask again?"

Possible reasons:
- **Canon input became stale**: Previously fresh, now expired. Refresh it.
- **Conflict was resolved**: Canon and Boundline now agree (or new conflict emerged)
- **Task state changed**: New evidence available in runtime (new traces, updated status)
- **Contract version changed**: New S7 contract version released with different rules

### "Why does S7 surface a conflict?"

When Boundline and Canon assess the same dimension differently:
- Example: Runtime says "proceed" but readiness says "NOT READY"
- Action: Investigate with responsible authority (e.g., readiness assessment team)
- Authority: Canon governance takes precedence for decision-making

### "How do I know if an answer is trustworthy?"

1. **Check sources**: Does it cite both Boundline and Canon? Or one + missing?
2. **Check confidence**: Is confidence High (all evidence present) or Low (evidence gaps)?
3. **Check degradation**: Any stale, missing, or conflicting signals?
4. **Ask maintainers**: If uncertain, reach out to Boundline or Canon team

---

## Documentation Reference

| Document | Purpose | Audience |
|----------|---------|----------|
| `assistant-delight-contract.md` | Contract boundaries | Maintainers, architects |
| `assistant-delight-explanation-vocabulary.md` | Standardized terms | Operators, implementers |
| `assistant-delight-input-classes.schema.json` | Input class schema | Validation tooling, canonical reference |
| `assistant-delight-degradation-modes.md` | Failure handling | Implementers |
| `assistant-delight-extension-procedures.md` | Amendment process | Maintainers proposing changes |
| `data-model.md` | Data structures | Implementers, system designers |

---

## Key Takeaways

1. ✅ **S7 answers always show their sources** — Never assume, always attribute
2. ✅ **Degradation is visible** — Missing/stale/incompatible inputs are explicit
3. ✅ **Canon is authoritative but not dictating** — Canon governance takes precedence, but Boundline owns how to present it
4. ✅ **The contract evolves deliberately** — Changes require bilateral review, documented amendments
5. ✅ **Operators remain in control** — No silent magic; all decisions are visible and justified

---

## Getting Help

- **Questions about the contract**: See the full contract documents in `contracts/` directory
- **Adding new S7 capabilities**: Follow `assistant-delight-extension-procedures.md`
- **Troubleshooting S7 answers**: Check degradation signals and recency of sources
- **Reaching maintainers**: Boundline team (@boundline-team) and Canon team (@canon-team)
