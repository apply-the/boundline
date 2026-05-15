# RSpec and Minitest Guidelines

## Principi

Ruby test deve rimanere leggibile e diretto. RSpec è espressivo ma può diventare troppo magico. Minitest è semplice ma può diventare ripetitivo senza helper curati.

## RSpec

```ruby
RSpec.describe OrderService do
  it "rejects empty orders" do
    result = service.create_order(empty_order_command)

    expect(result).to be_failure
  end
end
```

## Minitest

```ruby
class OrderServiceTest < Minitest::Test
  def test_rejects_empty_orders
    result = service.create_order(empty_order_command)

    assert result.failure?
  end
end
```

## let e subject

In RSpec, non abusare di `let`, `subject` e contesti annidati.

### Da evitare

- cinque livelli di `context`
- `let` ridefiniti ovunque
- test dove bisogna saltare su e giù per capire il setup

### Preferibile

Setup esplicito nel test quando aumenta leggibilità.

## Factories

FactoryBot utile, ma evitare factory con callback e side effect nascosti.

## Rails

- model spec per invariant
- request spec per API
- service spec per use case
- system spec solo flussi critici

## Mock

Usare verifying doubles quando possibile.

```ruby
instance_double(OrderRepository)
```

Preferire fake per repository semplici.

## Anti-pattern

- RSpec troppo DSL/magico
- callback FactoryBot pesanti
- test dipendenti dal DB senza isolamento
- `allow_any_instance_of`
- expectation su dettagli interni
