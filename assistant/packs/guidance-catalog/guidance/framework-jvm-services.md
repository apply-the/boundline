# JVM Service Frameworks

Conventions for JVM server frameworks including Spring Boot, Quarkus, and Micronaut.

## Architecture

Separate controller/resource (transport) from service (application) from domain (entities, value objects) from repository (infrastructure). Framework annotations belong at the transport and infrastructure layers, not in domain code.

## Spring Boot

Use constructor injection exclusively. Keep `@RestController` handlers thin. Use `@Service` for application orchestration. Keep domain classes annotation-free.

```java
@RestController
@RequestMapping("/orders")
public class OrderController {
    private final OrderService orderService;

    public OrderController(OrderService orderService) {
        this.orderService = orderService;
    }
}
```

## Request Validation

Use Bean Validation (`@Valid`, `@NotNull`, `@Size`) on DTOs at the controller boundary. Do not use validation annotations on domain objects.

## Error Handling

Use `@ControllerAdvice` or framework-specific exception handlers. Map domain exceptions to HTTP status codes at the transport boundary. Do not expose stack traces.

## Database Access

Use Spring Data repositories or JPA. Keep queries in repository interfaces. Use explicit transactions (`@Transactional`) with clear boundaries. Avoid lazy loading surprises.

## Anti-Patterns

- Business logic in controllers
- JPA entities used as API responses
- Field injection (`@Autowired` on fields)
- Domain objects with Spring annotations
- Missing transaction boundaries on write operations
- N+1 queries from lazy loading
- `@ComponentScan` catching unintended beans

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, public-contract-stability
- `clean_code`: no-mixed-responsibilities
- `security_boundary`: input validation at request boundary
