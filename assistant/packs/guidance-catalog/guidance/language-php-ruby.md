# PHP And Ruby

Modern PHP and Ruby can be robust with strict types, value objects, dependency injection, and boundary validation. Avoid global state, untyped associative arrays/hashes, and static service access.

## PHP

### Strict Types

Enable strict types in every application file:

```php
<?php
declare(strict_types=1);
```

### Domain Modeling

Use `readonly` classes and type declarations:

```php
final readonly class OrderId
{
    public function __construct(public string $value)
    {
        if ($value === '') {
            throw new InvalidArgumentException('OrderId cannot be empty');
        }
    }
}
```

Avoid associative arrays as domain models. Use typed command/query objects instead.

### Dependency Injection

Use constructor injection. Keep the DI container at the composition root. Avoid service locators and static access.

### Error Handling

Use specific exceptions. Never catch generic `\Exception` without re-throwing or handling. Preserve context in exception chains.

## Ruby

### Domain Modeling

Use frozen value objects or data classes (Ruby 3.2+):

```ruby
OrderId = Data.define(:value) do
  def initialize(value:)
    raise ArgumentError, "OrderId cannot be empty" if value.empty?
    super
  end
end
```

### Dependency Injection

Pass collaborators through constructors. Avoid relying on global constants or class-level state for dependencies.

### Error Handling

Use specific error classes. Avoid rescuing `StandardError` broadly. Preserve cause chains.

## Recommended Ecosystem Libraries

### PHP

| Category | Package | Purpose |
|----------|---------|---------|
| Testing | PHPUnit | Standard test framework |
| Static analysis | PHPStan or Psalm | Type-level verification |
| Logging | Monolog | PSR-3 structured logging |
| ORM | Doctrine or Eloquent | Database abstraction |
| HTTP | Guzzle or Symfony HttpClient | HTTP client with middleware |
| DI container | Symfony DI or Laravel Container | Constructor injection |
| Linting | PHP-CS-Fixer or Pint | Code style enforcement |

### Ruby

| Category | Gem | Purpose |
|----------|-----|---------|
| Testing | RSpec or Minitest | Test framework |
| Linting | RuboCop | Style and complexity checks |
| Types | Sorbet or RBS | Gradual type checking |
| ORM | ActiveRecord or Sequel | Database access |
| HTTP client | Faraday | Middleware-based HTTP |
| Background jobs | Sidekiq | Redis-backed job processing |
| API serialization | Alba or Blueprinter | Object-to-JSON mapping |

## Common Anti-Patterns

- Untyped arrays/hashes flowing through business logic
- Global state or static service access
- Catching all exceptions silently
- Missing strict types (PHP) or frozen objects (Ruby)
- Framework coupling in domain code
- Missing boundary validation on external input
- Monkey patching as standard test strategy

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-hidden-side-effects
- `architecture_boundary`: dependency-direction
- `testability`: untestable-design
