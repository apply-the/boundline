# Mobile Frameworks

Conventions for mobile development including React Native, Flutter/Dart, and native iOS/Android.

## Architecture

Separate UI (views, components) from state management from domain logic from data/network layers. Keep platform-specific code behind abstraction boundaries.

## State Management

Use unidirectional data flow. Keep state immutable where possible. Isolate side effects (network, storage, sensors) from pure UI rendering.

Flutter: BLoC, Riverpod, or Provider with clear separation.
React Native: Redux Toolkit, Zustand, or React Query with clear boundaries.

## Navigation

Keep navigation logic out of business code. Use typed routes. Handle deep linking at the routing layer.

## Platform Integration

Wrap platform APIs behind interfaces. Test business logic without platform dependencies. Use dependency injection for platform services.

## Offline And Sync

Design for offline-first where connectivity is unreliable. Keep local state consistent. Handle conflict resolution explicitly. Do not assume network availability.

## Testing

Test business logic in isolation from UI framework. Use widget/component tests for UI behavior. Use integration tests sparingly for critical user journeys.

## Anti-Patterns

- Business logic in UI components or widgets
- Direct platform API calls from domain layer
- Mutable global state for app-wide data
- Missing offline handling for network-dependent features
- Navigation logic coupled to business decisions
- Untested state management logic

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction (platform deps in domain)
- `clean_code`: no-mixed-responsibilities, no-hidden-side-effects
- `testability`: untestable-design
