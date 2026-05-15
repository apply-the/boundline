# Rails And Laravel

Conventions for Ruby on Rails and PHP Laravel frameworks.

## Architecture

Both frameworks encourage convention over configuration. Use that strength but keep domain logic separable from framework magic.

Rails: use service objects, form objects, and query objects to keep controllers and models thin.
Laravel: use actions, form requests, and query builders to avoid fat controllers and god models.

## Model Layer

Keep ActiveRecord/Eloquent models focused on persistence and associations. Extract complex business rules into service objects or domain classes.

```ruby
# Rails: thin model
class Order < ApplicationRecord
  belongs_to :customer
  validates :status, presence: true
end

# Business logic in service
class CreateOrder
  def call(command)
    # orchestration here
  end
end
```

## Request Validation

Rails: use strong parameters and form objects.
Laravel: use Form Requests with explicit validation rules.

Validate at the boundary; do not repeat validation deep in domain code.

## Background Jobs

Use framework job systems (Sidekiq, Horizon) for async work. Make jobs idempotent. Handle failures explicitly. Use dead-letter patterns for poison messages.

## Database

Use migrations for schema changes. Follow expand/contract for breaking changes. Use database-level constraints for critical invariants. Avoid N+1 queries.

## Anti-Patterns

- Business logic in controllers (fat controllers)
- Business logic in models (god models)
- Callbacks with hidden side effects
- Missing background job idempotency
- N+1 queries from lazy loading
- Directly using framework helpers in domain logic
- Missing request validation

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, public-contract-stability
- `clean_code`: no-mixed-responsibilities, no-hidden-side-effects
- `testability`: untestable-design (when logic is in callbacks)
