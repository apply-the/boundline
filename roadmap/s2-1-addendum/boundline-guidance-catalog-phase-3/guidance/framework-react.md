# React Guidance

## Purpose

This guidance defines React and modern frontend practices for AI-assisted planning, implementation, testing, review, and refactoring.

It applies to React applications, component libraries, Next.js applications, and UI-heavy web systems.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, architecture decision, or Canon-governed standard.

## Version Posture

Active support window:
- React 18+
- TypeScript where repository standard allows

Target excellence:
- Server Components where the framework supports them and the architecture benefits
- explicit server/client boundaries
- TanStack Query or equivalent for server state where appropriate
- user-centric testing

Legacy warnings:
- class components in new code without strong reason
- effects used for derivable state
- global mutable client state for server-owned data
- component trees with hidden data ownership

## Core Design Principles

### Server State Is Not Client State

Data fetched from a server is server state.

Server state concerns:
- cache invalidation
- freshness
- retries
- loading states
- optimistic updates
- background refresh

Do not store server state in ad hoc local component state unless the lifecycle is truly local.

Recommended:
- TanStack Query
- framework-native data loaders
- server components where applicable
- explicit mutation invalidation

### Derive Before You Store

Avoid storing values that can be derived from props, URL state, or other state.

Bad:
- state duplicated from props
- derived booleans updated through effects
- recalculated display state stored separately without reason

Prefer:
- derived variables
- memoization when expensive
- reducer when state transitions are meaningful

### Effects Are For Synchronization

Effects should synchronize React with external systems.

Good uses:
- subscriptions
- imperative third-party APIs
- browser APIs
- analytics events with clear lifecycle
- external resource synchronization

Bad uses:
- deriving state
- copying props into state
- hiding business logic
- chaining complex workflows

### Component Boundaries

Components should have clear responsibility.

Separate:
- presentation
- data loading
- mutation orchestration
- domain-specific UI behavior
- framework routing concerns

Avoid:
- giant page components
- deeply nested prop drilling for domain state
- hidden side effects in presentational components
- mixing API payload shape directly into UI state

### Server/Client Boundary

In Next.js or similar frameworks:

Keep server code:
- data fetching
- secure secrets
- privileged operations
- server-only dependencies

Keep client code:
- browser interaction
- local UI state
- event handling
- client-only effects

Guardians should flag:
- secrets in client components
- server-only APIs imported into client code
- unnecessary client boundaries
- hydration-sensitive behavior

### Forms And Validation

Validate at the boundary.

Use:
- schema validation
- form libraries where complexity justifies them
- server validation for authoritative rules
- client validation for feedback only

Avoid:
- trusting client validation
- duplicating schemas manually
- silently accepting invalid server responses

### Accessibility

UI work should preserve accessibility.

Check:
- semantic HTML
- labels
- keyboard navigation
- focus management
- ARIA only when necessary
- color contrast
- screen-reader meaningful names

### Performance

Common frontend performance concerns:
- unnecessary client components
- excessive re-rendering
- large bundle imports
- unbounded lists
- over-fetching
- waterfall data loading
- expensive effects

Do not optimize prematurely, but do not introduce obvious performance regressions.

## Testing Guidance

Recommended:
- Testing Library for behavior tests
- Playwright for E2E
- MSW for network boundaries
- user-centric queries such as `getByRole`
- avoid implementation-detail selectors

Avoid:
- overusing snapshots
- testing private hooks instead of visible behavior
- relying on fixed sleeps
- mocking components under test
- bypassing accessibility semantics

## Anti-Patterns

- effects used for derived state
- local state used as server cache
- secrets in client bundle
- server-only imports in client components
- page components with business policy
- API response objects used directly everywhere
- uncontrolled prop drilling for domain workflows
- snapshot-only coverage
- inaccessible interactive elements

## Guardian Hooks

Recommended guardians:
- react-server-client-boundary-guardian
- react-server-state-guardian
- react-effect-misuse-guardian
- react-accessibility-guardian
- react-component-boundary-guardian
- react-test-validity-guardian

## Structured Finding Example

```json
{
  "guardian": "react-server-state",
  "rule": "server-state-in-local-component-state",
  "disposition": "concern",
  "summary": "Fetched account data is copied into local state and manually invalidated after mutation.",
  "evidence_refs": ["src/app/accounts/AccountEditor.tsx"],
  "recommended_action": "Use the existing server-state mechanism and invalidate the account query after mutation."
}
```

## Lifecycle Usage

Planning:
- identify server/client boundary and data ownership

Implementation:
- guide component boundaries, effects, validation, and accessibility

Testing:
- guide behavior-level tests and E2E coverage

Review:
- check server-state misuse, effect misuse, accessibility regressions, and client/server boundary leaks

Refactoring:
- split oversized components without changing behavior
