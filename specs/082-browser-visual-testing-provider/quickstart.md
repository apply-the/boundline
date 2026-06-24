# Quickstart: Browser And Visual Testing Provider

**Feature Branch**: `082-browser-visual-testing-provider` | **Date**: 2026-06-20

## Prerequisites

- Rust 1.96.0+ with edition 2024
- Existing Boundline workspace with `.boundline/` directory
- A browser provider binary (e.g., `boundline-browser-provider`) installed and on `PATH`
- Playwright or equivalent browser automation library installed (managed by the provider, not Boundline)

## 1. Register the Browser Provider

```bash
# Register a Playwright-backed browser provider
boundline provider add browser-playwright \
  --kind browser \
  --command boundline-browser-provider \
  --args "serve,--stdio"

# Verify activation
boundline provider status --verbose
# Output includes:
#   browser-playwright (browser) — active
#   Capabilities: screenshot, console, readiness_locators, dom_inspection, accessibility
```

## 2. Configure the Provider

Add to `.boundline/config.toml`:

```toml
[providers.browser-playwright]
kind = "browser"
transport = "command"
command = "boundline-browser-provider"
args = ["serve", "--stdio"]
enabled = true
max_concurrency = 2
max_queue_size = 10
queue_timeout_seconds = 60
execution_timeout_seconds = 120

[providers.browser-playwright.environment]
inherit = ["PATH", "DISPLAY", "HTTP_PROXY", "HTTPS_PROXY", "NO_PROXY"]
```

## 3. Run a Basic Browser Validation

```bash
# Single URL check with screenshot + console capture
boundline validate browser --url http://localhost:3000

# With readiness locator for SPA pages
boundline validate browser \
  --url http://localhost:3000/dashboard \
  --readiness-selector "[data-testid='dashboard-ready']" \
  --readiness-state visible \
  --readiness-timeout 20
```

## 4. Inspect Results

```bash
# View evidence packet summary
boundline inspect browser --run browser-run-abc123

# View specific artifacts
boundline inspect browser --run browser-run-abc123 --artifacts

# View findings only
boundline inspect browser --run browser-run-abc123 --findings
```

## 5. Run Accessibility Checks

```bash
boundline validate browser \
  --url http://localhost:3000 \
  --accessibility
```

## 6. Run Scripted Interaction Flow

```bash
boundline validate browser \
  --url http://localhost:3000/login \
  --script steps.json
```

Where `steps.json`:

```json
{
  "steps": [
    { "action": "type", "selector": "#username", "text": "admin" },
    { "action": "type", "selector": "#password", "text": "secret" },
    { "action": "click", "selector": "#submit" },
    { "action": "screenshot", "label": "after-login" }
  ]
}
```

## 7. Visual Diff Against Baseline

```bash
# First run creates the baseline
boundline validate browser --url http://localhost:3000 --baseline dashboard-v1

# Subsequent runs compare against the baseline
boundline validate browser --url http://localhost:3000 --baseline dashboard-v1
# If visual difference > threshold, reports visual_diff_detected finding
```

## 8. Check Provider Health

```bash
boundline provider health browser-playwright
# Output: active, queue_depth=0, capabilities={...}
```

## Key Behaviors

| Scenario | Behavior |
|----------|----------|
| No provider configured | `boundline validate browser` reports provider not found |
| Provider binary missing | Provider marked as blocked with health failure |
| URL unreachable | `page_load_timeout` finding returned |
| Console errors on page | `console_error` findings in evidence packet |
| Readiness selector not found | `browser_readiness_timeout` with diagnostic screenshot |
| Network request to non-allowlisted domain | `network_access_violation` finding (non-blocking) |
| Queue at capacity | `browser_queue_full` — request rejected immediately |
| Queue timeout | `browser_queue_timeout` — request rejected after waiting |
| Provider crashes mid-step | `provider_unavailable` finding, queued requests failed |
| Visual diff within tolerance | Pass finding with actual diff percentage |

## Artifact Locations

```
.boundline/sessions/<session-id>/browser/<run-id>/
├── evidence.json          # Normalized evidence packet
├── screenshots/
│   ├── final.png          # Final page screenshot
│   └── failure.png        # Diagnostic screenshot on failure
├── console.json           # Console log (structured JSON)
├── network.json           # Network trace (if captured)
├── dom.html               # DOM snapshot (if requested)
└── accessibility.json     # Accessibility audit output
```
