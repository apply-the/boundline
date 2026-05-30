# Quickstart: Guidance And Guardian Capabilities

## 1. Prepare A Shared Or Workspace-Local Capability Source

Start from a workspace that already uses the normal Boundline session-native flow.
Optionally add local overrides:

```bash
mkdir -p .boundline/guidance .boundline/guardians
printf '%s
' '# Local Rust guidance' 'Prefer typed error values over panic-based flow.' > .boundline/guidance/rust-local.md
cat > .boundline/guardians/magic-values.toml <<'EOF'
[guardians.magic_values]
title = "Magic Value Guardian"
kind = "deterministic"
applies_to = ["implementation", "review"]
rules = ["magic_numbers", "magic_strings"]
severity_floor = "concern"
command = "scripts/check-magic-values.sh"
EOF
```

Expected result:
- the workspace contains explicit override inputs that can shadow shared or Canon-governed sources.

## 2. Capture A Narrow Goal

Run:

```bash
boundline goal --goal "tighten the Rust retry policy and keep the flow inspectable"
```

Expected result:
- the session stores the authored goal and bounded target cues.

## 3. Build The Plan

Run:

```bash
boundline plan
```

Expected result:
- Boundline resolves guidance sources for the active planning context.
- the plan persists a capability-resolution summary with loaded and skipped source provenance.
- if Canon-governed standards are absent, the local-first path remains explicit.

## 4. Run A Bounded Implementation Step

Run:

```bash
boundline run
```

Expected result:
- after the bounded implementation work completes, Boundline runs any guardians that apply to the active phase in deterministic-before-LLM order.
- each guardian execution ends with structured findings, a degraded outcome, or an explicit failure record.

## 5. Inspect Session-Native Projection

Run:

```bash
boundline status
boundline next
```

Expected result:
- `status` surfaces the loaded guidance and guardian sources, skipped-source reasons, and finding summary.
- `next` stays aligned with the same persisted capability and finding story instead of recomputing hidden behavior.

## 6. Inspect The Trace

Run:

```bash
boundline inspect
```

Expected result:
- the trace summary distinguishes workspace overrides, Canon-governed standards, shared packs, and built-ins.
- guardian order, findings, and any routing degradation remain operator-visible.
- if no suitable runtime route existed for a semantic guardian, the trace shows an explicit degraded outcome instead of a silent fallback.
