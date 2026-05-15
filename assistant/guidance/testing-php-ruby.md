# PHP And Ruby Testing Guidance

Use PHPUnit, Pest, RSpec, and Minitest to verify behavior with readable setup and explicit boundaries.

- Name tests by scenario and behavior so failures explain what changed.
- Prefer factories and builders that read like domain data instead of opaque fixture blobs.
- Keep unit tests on domain logic fast and isolated.
- Use request or integration tests for routing, serialization, authorization, and persistence boundaries.
- Avoid global fixtures, hidden database state, and time-coupled assertions.
- Reserve end-to-end flows for a small number of critical user journeys.
