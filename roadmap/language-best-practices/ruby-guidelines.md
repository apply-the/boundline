# Ruby Guidelines

## Principi

Ruby favorisce espressività e velocità, ma senza disciplina può diventare difficile da mantenere. Rendere espliciti contratti, dipendenze e side effect è fondamentale.

## Oggetti piccoli e dipendenze esplicite

Iniettare dipendenze nel costruttore.

```ruby
class OrderService
  def initialize(repository:, payment_client:)
    @repository = repository
    @payment_client = payment_client
  end
end
```

Evitare dipendenze recuperate da costanti globali o singleton dentro la logica.

## Value object

Usare oggetti semantici per concetti di dominio.

```ruby
class OrderId
  attr_reader :value

  def initialize(value)
    raise ArgumentError, "OrderId cannot be empty" if value.nil? || value.empty?

    @value = value.freeze
    freeze
  end
end
```

## Immutabilità dove utile

Ruby è mutabile di default. Congelare value object e costanti riduce bug.

```ruby
SUPPORTED_CURRENCIES = ["EUR", "USD"].freeze
```

Per array/hash passati all’interno di oggetti, duplicare e congelare quando serve proteggere invarianti.

## Error handling

Usare eccezioni specifiche.

```ruby
class OrderNotFoundError < StandardError
end
```

Regole:

- non fare `rescue StandardError` senza gestire o rilanciare
- non usare `rescue nil`
- non ingoiare errori
- aggiungere contesto quando rilanci
- non usare eccezioni per flusso ordinario ad alta frequenza

## Nil handling

Non lasciare che `nil` attraversi tutto il sistema.

### Da evitare

```ruby
customer.id
```

quando `customer` può essere `nil`.

### Preferibile

Validare ai boundary o usare oggetti risultato.

```ruby
Result.failure(:customer_not_found)
```

## Resource management

Usare block API per risorse.

```ruby
File.open(path) do |file|
  file.read
end
```

Per risorse custom, esporre API a blocco.

```ruby
connection_pool.with_connection do |connection|
  # ...
end
```

## Logging

Usare logger strutturato se disponibile.

```ruby
logger.info({ message: "Order created", order_id: order_id.value })
```

Non usare `puts` in codice applicativo.

## Testabilità

Separare logica pura da framework, database e rete. Evitare test che richiedono bootstrap completo se non sono integration test.

```ruby
service = OrderService.new(
  repository: FakeOrderRepository.new,
  payment_client: FakePaymentClient.new
)
```

## Metaprogramming

Usare metaprogramming con moderazione.

Regole:

- evitare DSL opache per logica di dominio critica
- preferire codice esplicito quando il comportamento deve essere letto rapidamente
- testare bene macro/metodi generati
- documentare convenzioni implicite

## Cose da evitare

- monkey patching globale
- `rescue nil`
- dipendenze nascoste in costanti globali
- `puts` in servizi
- oggetti hash non validati che attraversano il dominio
- callback framework con troppa logica
- metaprogramming non necessario
- mutazione di input ricevuti dal chiamante
