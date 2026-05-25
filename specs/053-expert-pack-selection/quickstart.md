# Quickstart: Expert Pack Selection

## 1. Prepare Bounded Workspace Cues

Start from a workspace that already has domain templates and any reviewer-role
routes configured through normal Boundline setup.

Expected result:
- effective local configuration can resolve at least one bounded domain family or reviewer role.

## 2. Capture A Narrow Goal

Run:

```bash
boundline goal --goal "review the React auth changes in src/auth"
```

Expected result:
- the session stores the authored goal and bounded target cues.

## 3. Build The Plan

Run:

```bash
boundline plan
```

Expected result:
- Boundline computes an expert-pack selection outcome before planning continues.
- selected packs and suggested runtime roles are attached to the planning context.
- if Canon expertise input is absent or incompatible, the local-only path remains explicit.

## 4. Inspect Session-Native Projection

Run:

```bash
boundline status
boundline next
```

Expected result:
- `status` surfaces the expert-selection summary, selected packs, suggested runtime roles, and any rejected candidates.
- `next` stays aligned with the same bounded selection outcome instead of recomputing hidden role choices.

## 5. Inspect The Trace

Run:

```bash
boundline inspect
```

Expected result:
- the trace summary distinguishes local expert-selection cues from optional Canon expertise inputs.
- rejection reasons remain operator-visible when no candidate or role was credible.
