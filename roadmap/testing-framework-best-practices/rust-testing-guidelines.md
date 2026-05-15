# Rust Testing Guidelines

## Principi

Rust supporta unit test, integration test e doc test in modo nativo. I test devono sfruttare type system, ownership e Result invece di `unwrap` indiscriminato.

## Unit test

Mettere test vicini al codice quando testano dettagli del modulo.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_order() {
        let result = Order::new(vec![]);

        assert!(matches!(result, Err(OrderError::Empty)));
    }
}
```

## Integration test

Usare `tests/` per test del contratto pubblico della crate.

```text
tests/
  create_order.rs
```

## Test che ritornano Result

Per test con `?`, usare return `Result`.

```rust
#[test]
fn parses_valid_order_id() -> Result<(), OrderIdError> {
    let id = OrderId::parse("order-1")?;

    assert_eq!(id.as_str(), "order-1");
    Ok(())
}
```

## unwrap nei test

`unwrap` è accettabile nei test piccoli, ma `expect` con messaggio aiuta diagnosi.

```rust
let id = OrderId::parse("order-1").expect("valid order id should parse");
```

## Property-based testing

Usare strumenti come proptest per logica con molti input.

```rust
proptest! {
    #[test]
    fn total_is_never_negative(lines in order_lines_strategy()) {
        // ...
    }
}
```

## Async tests

Usare runtime test macro coerente.

```rust
#[tokio::test]
async fn creates_order() {
    // ...
}
```

Non tenere lock sincroni attraverso `.await`.

## Testcontainers

Per integration test con DB/broker, preferire container reali a mock del driver.

## Anti-pattern

- test integration che dipendono da servizi manuali
- `unwrap` in test lunghi senza contesto
- test che verificano implementazione privata quando basta contratto pubblico
- sleep per async
- feature flag/test cfg caotici
