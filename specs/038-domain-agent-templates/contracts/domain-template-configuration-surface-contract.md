# Contract: Domain Template Configuration Surface

**Feature**: 038-domain-agent-templates  
**Date**: 2026-05-03

## Purpose

Define how Synod exposes domain-family activation, standards layering, and
effective source attribution through initialization and configuration surfaces.

## Required Surface

- `init` must be able to seed active domain families for a workspace and record
  workspace-scoped standards or external bindings when the operator supplies
  them.
- `config show` must expose workspace, cluster, global, and effective domain
  settings with source attribution for the winning standards layer.
- Post-init configuration commands must be able to enable or disable a family,
  update scoped standards text, and bind or unbind external context inputs.

## Explicit Boundaries

- The surface must not hide whether effective guidance came from built-in,
  global, cluster, or workspace scope.
- The surface must not require operators to edit raw config files to perform
  normal post-init customization.
- The surface must not imply that enabling a domain family automatically makes
  every external input available.