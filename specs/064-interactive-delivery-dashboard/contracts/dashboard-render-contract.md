# Contract: Dashboard Render

## Purpose

Define the dashboard rendering states that must remain stable enough to test without depending on a particular terminal frame buffer.

## Render Modes

| Mode | Trigger | Required Output |
|------|---------|-----------------|
| `interactive` | Terminal supports interactive rendering and workspace state is readable | Full dashboard with summary, timeline, panels, actions, and wordmark |
| `compact` | Terminal width is limited but interactive rendering is available | Summary, next action, blocking reason, and focused panel navigation |
| `monochrome` | Color is disabled or unavailable | Same content as interactive or compact with no color dependency |
| `degraded` | Interactive rendering cannot start or state cannot be read safely | Reason, valid fallback command, and no misleading dashboard state |

## Required First-Screen Regions

- Brand wordmark using terminal-safe text.
- Active workspace and session summary.
- Current stage and current step.
- Execution condition and latest status.
- Next bounded action or blocking reason.
- Recent timeline summary.
- Available action list.
- Degraded notice when applicable.

## Panel Requirements

- Goal plan panel must show state, revision, targets, and verification strategy when available.
- Evidence panel must show selected evidence and omitted or degraded context when available.
- Findings panel must show status, severity, and evidence references when available.
- Checkpoint panel must show latest checkpoint refs and recovery commands when available.
- Governed reference panel must show read-only readiness, provenance, and approval cues when available.
- Empty panels must distinguish "none" from "unavailable".

## Branding Rules

- The dashboard must render a simple `boundline` ASCII wordmark.
- The wordmark may use color when color is available.
- The fallback is plain `boundline`.
- The render path must not require SVG files, raster images, or wide ANSI banner art.

## Layout Refusal Rules

- If terminal width cannot display meaningful state, render degraded or compact mode rather than clipping critical text.
- If terminal height cannot display all regions, keep summary, blocking reason, and next action visible before secondary panels.
- If color support is unavailable, content semantics must remain unchanged.
