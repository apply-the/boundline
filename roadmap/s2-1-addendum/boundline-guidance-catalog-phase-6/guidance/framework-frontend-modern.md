# Modern Frontend Framework Guidance

## Purpose

This guidance defines cross-framework frontend practices beyond React, covering Angular, Vue, Svelte, Next.js-style full-stack frontend boundaries, and modern UI application concerns.

It applies to frontend applications, component libraries, design-system consumers, and full-stack frontend frameworks.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, design-system policy, architecture decision, or Canon-governed standard.

## Framework Posture

This guidance is not a mandate to migrate frameworks.

It helps Boundline and guardians reason about modern frontend patterns across:

- Angular
- Vue
- Svelte
- Next.js-style routing/data frameworks
- component-driven UI systems
- design system integrations

Framework-specific preferences should follow repository policy.

## Angular Guidance

Modern Angular applications should prefer:

- standalone components over NgModule-heavy designs where repo version supports it
- signals for local reactivity where appropriate
- explicit dependency boundaries
- typed reactive forms where used
- route-level lazy loading for significant features

Guardians should watch for:

- legacy module sprawl in new code
- service classes becoming global mutable state
- business policy inside components
- subscriptions without cleanup or framework-supported lifecycle handling
- untyped form values crossing domain boundaries

## Vue Guidance

Modern Vue applications should prefer:

- Composition API for new complex logic
- explicit composables for reusable behavior
- component props and emits with clear typing
- Pinia or equivalent for application state where needed
- server-state separation from client UI state

Guardians should watch for:

- Options API additions in codebases that standardized on Composition API
- hidden side effects in composables
- shared mutable state without ownership
- untyped event payloads
- API payloads used directly as UI domain model everywhere

## Svelte Guidance

Modern Svelte applications should prefer:

- framework-current reactivity model where repo version supports it
- simple local state for local UI behavior
- explicit stores or state modules for shared ownership
- accessible markup by default
- clear server/client boundary where using SvelteKit

Guardians should watch for:

- global stores for local state
- reactive statements hiding side effects
- server-only data leaking to client
- form actions without validation ownership
- untestable reactive chains

## Next.js And Full-Stack Frontend Boundaries

For Next.js-style applications:

Clarify:
- server component vs client component
- server action vs API route
- static vs dynamic rendering
- cache and invalidation policy
- auth boundary
- data ownership
- mutation strategy

Guardians should flag:

- secrets imported into client code
- unnecessary client components
- cache invalidation missing after mutation
- auth logic split inconsistently between middleware, server action, and handler
- API payload shape leaked everywhere

## State Management

Classify state:

- local UI state
- server state
- URL state
- form state
- global application state
- persisted client state

Do not use one global state mechanism for all categories.

## Accessibility

Frontend code should preserve:

- semantic HTML
- keyboard navigation
- focus behavior
- accessible names
- form labels
- meaningful error messages
- reduced reliance on ARIA when native HTML is sufficient

Accessibility regressions should be treated as product correctness issues, not polish.

## Testing Guidance

Use framework-appropriate tools, but prefer behavior over implementation.

Recommended:
- Testing Library style queries where applicable
- Playwright for high-value E2E
- component tests for design-system behavior
- accessibility checks for critical flows
- MSW or equivalent for HTTP boundaries

Avoid:
- snapshot-only confidence
- brittle component internals
- fixed sleeps
- testing framework plumbing instead of user behavior
- ignoring keyboard-only flows

## Anti-Patterns

- global store for every state category
- framework components containing business policy
- API response used as long-lived UI model without mapping
- server-only imports in client code
- cache invalidation ignored
- inaccessible custom controls
- reactive chain with hidden side effects
- snapshot-only tests
- permanent workaround components without owner

## Guardian Hooks

Recommended guardians:
- frontend-reactivity-guardian
- frontend-state-ownership-guardian
- frontend-accessibility-guardian
- frontend-server-client-boundary-guardian
- frontend-cache-invalidation-guardian
- frontend-test-validity-guardian

## Structured Finding Example

```json
{
  "guardian": "frontend-state-ownership",
  "rule": "server-state-in-global-ui-store",
  "disposition": "concern",
  "summary": "Server-owned account data is copied into a global UI store without freshness or invalidation rules.",
  "evidence_refs": ["src/stores/account.ts", "src/routes/account/+page.ts"],
  "recommended_action": "Use the repository's server-state mechanism or document cache ownership and invalidation."
}
```

## Lifecycle Usage

Planning:
- classify state ownership, routing, and data boundaries

Architecture:
- define server/client boundary and design-system responsibility

Implementation:
- guide framework idioms, accessibility, and state strategy

Testing:
- guide behavior-level UI tests and accessibility checks

Review:
- check reactivity misuse, state ownership, cache invalidation, and accessibility regressions
