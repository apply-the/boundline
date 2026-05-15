# Testing Frontend

Testing conventions for frontend applications using Testing Library, Playwright, and related tools.

## Test Organization

- Component tests: co-located `*.test.tsx` or `*.spec.tsx`
- Integration tests: test user flows through multiple components
- E2E tests: Playwright or Cypress for critical user journeys

## Component Testing

Use Testing Library. Query by accessible roles, labels, or text content. Test behavior visible to users, not implementation details.

```tsx
render(<OrderForm onSubmit={mockSubmit} />);
await userEvent.type(screen.getByLabelText("Name"), "Alice");
await userEvent.click(screen.getByRole("button", { name: "Submit" }));
expect(mockSubmit).toHaveBeenCalledWith({ name: "Alice" });
```

## Accessibility Testing

Include accessibility assertions in component tests. Use `axe-core` integration. Test keyboard navigation for interactive elements.

## Async Testing

Use `waitFor` for async state changes. Avoid `setTimeout` in tests. Test loading, success, and error states.

## E2E Testing

Keep E2E tests focused on critical paths. Use page object pattern for maintainability. Run against realistic environments. Keep E2E suite fast by limiting scope.

## Recommended Tools

| Tool | Purpose |
|------|---------|
| `vitest` or `jest` | Test runner |
| `@testing-library/react` (or vue/svelte) | Accessible DOM queries |
| `msw` | Service worker HTTP mocks (browser + Node) |
| `Playwright` | Cross-browser E2E testing |
| `Cypress` | E2E and component testing |
| `axe-core` / `jest-axe` | Automated accessibility checks |
| `Storybook` | Component isolation and visual testing |
| `Chromatic` | Visual regression detection |

## Anti-Patterns

- Querying by CSS class or test ID when accessible queries exist
- Testing internal component state
- Snapshot tests as the primary testing strategy
- E2E tests for logic that can be tested at component level
- Missing async handling (`act` warnings)
- Tests coupled to specific rendering framework internals

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `clean_code`: test readability
