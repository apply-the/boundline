# S20 - Browser And Visual Testing Provider

## Owner

Boundline via External Capability Provider Protocol

## Status

B-level, after S10

## Strategic Role

This feature adds validation surfaces that code-only inspection cannot cover.

Frontend and web workflows often require visual evidence, browser state, accessibility checks, screenshots, and interaction traces.

## Problem

CLI-only validation misses:

- broken UI flows
- layout regressions
- accessibility failures
- auth redirect loops
- client/server runtime errors
- browser console errors
- visual differences
- frontend integration bugs

## Core Scope

- Browser provider through S10
- Playwright or Browsergym-style adapter
- Screenshot artifact capture
- Console log capture
- Network trace capture
- Accessibility scan hooks
- E2E evidence packet
- Visual validation findings
- Sandboxed browser execution where possible

## Provider Capabilities

- open URL
- perform scripted actions
- capture screenshot
- inspect DOM
- capture console errors
- capture network failures
- run accessibility checks
- produce evidence packet
- compare screenshot if baseline exists

## Suggested Technology

Start with:

- Playwright provider
- JSON stdio provider adapter
- screenshot artifact folder
- trace refs

Later:

- Browsergym provider
- visual diff provider
- remote browser sandbox

## Acceptance Criteria

- Boundline can invoke browser provider for a bounded validation step.
- Browser provider produces screenshots and logs as artifacts.
- Findings are normalized into Boundline structured findings.
- Evidence can be linked to Canon verification packet.
- Provider obeys network and path permissions.
- Failures are visible in inspect and S8.

## Risks

- Browser tests are flaky.
- Visual evidence becomes noisy.
- Provider requires heavy dependencies.
- Network access policy is too broad.

## Hard Rule

Browser automation is a provider capability, not core Boundline runtime.
