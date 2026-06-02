# Frequently Asked Questions (FAQ)

## Navigation & Getting Started

### How do I get a 2-minute setup?
Use the [Quick Start](/guide/getting-started) guide for the fastest installation and first run.

### I want a guided walkthrough, where do I start?
Start with [Getting Started](/guide/getting-started). It explains individual commands and how to verify your installation.

### How do I use Boundline with my assistant (Copilot, Claude, etc.)?
Read [Assistant Integrations](/guide/core-concepts). It explains the difference between global bootstrap commands and repository-local packages.

## Configuration & Operations

### How do I configure routing, hosts, or expert packs?
Use the [Configuration Reference](/reference/configuration) for workspace config, routing defaults, and guidance/guardian overrides.

### I am running a larger initiative, how does Boundline handle it?
Read [Project-Scale Delivery](/guide/core-concepts). Boundline handles complex work through bounded sessions and stages rather than unbounded runs.

## Troubleshooting & Inspection

### How do I understand a stop, failure, or finding?
Use [Traces And Inspectability](/guide/core-concepts) and [Troubleshooting](/guide/introduction). Treat `status`, `next`, `inspect`, and checkpoint output as the source of truth.

### Is Boundline dependent on Canon?
**No.** Boundline is Canon-aware but not Canon-dependent. Most local delivery work can run without Canon. When Canon is present, Boundline consumes its governed knowledge as authoritative evidence.
