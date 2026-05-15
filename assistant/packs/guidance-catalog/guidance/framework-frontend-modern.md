# Frontend Modern Frameworks

Conventions for modern frontend frameworks including React, Vue, Svelte, and Angular.

## Architecture

Separate UI rendering from state management from data fetching from domain logic. Keep framework-specific code at the edges. Business rules should be testable without the framework.

## Component Design

Keep components focused on one responsibility. Prefer composition over prop drilling. Use hooks/composables for reusable behavior. Keep side effects in dedicated layers.

## State Management

Use the simplest state solution that fits:
- Local state for component-scoped data
- Shared state (stores) for cross-component coordination
- Server state (React Query, SWR, TanStack Query) for remote data

Avoid duplicating server state in client stores.

## Data Fetching

Fetch at the boundary (page/route level or dedicated hooks). Handle loading, error, and empty states explicitly. Use caching strategies appropriate to data freshness requirements.

## Form Handling

Validate on the client for UX; validate on the server for security. Use schema-based validation (Zod, Yup) shared between client and server where possible.

## Accessibility

Semantic HTML first. ARIA only when semantics are insufficient. Keyboard navigation for all interactive elements. Screen reader testing for critical flows.

## Anti-Patterns

- Business logic in components
- Prop drilling through many levels instead of composition
- Manual cache management duplicating server state
- Missing loading/error states in UI
- Inline styles for design-system-governed elements
- Direct DOM manipulation bypassing the framework
- Missing accessibility on interactive elements

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction (business logic in UI)
- `clean_code`: no-mixed-responsibilities, no-hidden-side-effects
- `testability`: untestable-design (when logic is coupled to framework)
