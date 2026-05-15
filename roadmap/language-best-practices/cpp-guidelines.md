# C++ Guidelines

## Principi

C++ deve essere scritto usando RAII, value semantics e ownership esplicita. Scrivere C++ moderno come C con classi porta a leak, lifetime ambigui e codice fragile.

## RAII prima di tutto

Ogni risorsa deve essere posseduta da un oggetto il cui distruttore la rilascia.

```cpp
class FileHandle {
public:
    explicit FileHandle(const std::filesystem::path& path)
        : file_(std::fopen(path.string().c_str(), "r")) {
        if (file_ == nullptr) {
            throw std::runtime_error("failed to open file");
        }
    }

    ~FileHandle() {
        if (file_ != nullptr) {
            std::fclose(file_);
        }
    }

    FileHandle(const FileHandle&) = delete;
    FileHandle& operator=(const FileHandle&) = delete;

private:
    std::FILE* file_;
};
```

Quando possibile, usare tipi standard già RAII: `std::vector`, `std::string`, `std::unique_ptr`, `std::shared_ptr`, `std::lock_guard`, `std::jthread`.

## Ownership

Usare smart pointer solo quando serve ownership dinamica.

### Regole

- `std::unique_ptr<T>` per ownership esclusiva
- `std::shared_ptr<T>` solo per ownership realmente condivisa
- reference o pointer non owning per dipendenze non possedute
- evitare `new` e `delete` manuali
- evitare raw owning pointers

```cpp
class OrderService {
public:
    explicit OrderService(OrderRepository& repository)
        : repository_(repository) {}

private:
    OrderRepository& repository_;
};
```

## Rule of zero

Preferire classi che non definiscono destructor, copy/move constructor o assignment operator perché delegano ownership a membri RAII.

```cpp
class Order {
public:
    Order(OrderId id, std::vector<OrderLine> lines)
        : id_(std::move(id)), lines_(std::move(lines)) {}

private:
    OrderId id_;
    std::vector<OrderLine> lines_;
};
```

## Tipi semantici

Evitare primitive obsession.

```cpp
class OrderId {
public:
    explicit OrderId(std::string value) : value_(std::move(value)) {
        if (value_.empty()) {
            throw std::invalid_argument("OrderId cannot be empty");
        }
    }

    const std::string& value() const {
        return value_;
    }

private:
    std::string value_;
};
```

Usare `explicit` sui costruttori single-argument per evitare conversioni implicite.

## Error handling

Non usare un solo meccanismo ovunque per dogma. Scegliere in base al contesto.

### Linee guida

- eccezioni per errori eccezionali e invarianti violate
- `std::optional<T>` per assenza
- `std::expected<T, E>` o equivalente per errori attesi
- error codes per boundary C o hot path specifici

```cpp
std::expected<Order, FindOrderError> find_order(OrderId id);
```

Non lanciare eccezioni dai distruttori.

## Const correctness

Usare `const` in modo rigoroso.

```cpp
const Order& order;
std::string_view name;
```

Regole:

- metodi che non modificano stato devono essere `const`
- passare oggetti grandi per `const&`
- usare `std::string_view` per viste non owning, con attenzione al lifetime

## Concurrency

Usare RAII anche per lock.

```cpp
std::lock_guard<std::mutex> lock(mutex_);
```

Preferire `std::jthread` a `std::thread` quando disponibile, perché gestisce join e cancellation cooperativa tramite stop token.

Regole:

- non usare lock manuale/unlock manuale
- evitare data race
- documentare ownership condivisa
- minimizzare sezioni critiche
- non chiamare codice esterno mantenendo lock se evitabile

## Dependency injection

Passare dipendenze nel costruttore.

```cpp
class OrderService {
public:
    OrderService(OrderRepository& repository, PaymentClient& payment_client)
        : repository_(repository), payment_client_(payment_client) {}

private:
    OrderRepository& repository_;
    PaymentClient& payment_client_;
};
```

Usare interfacce astratte solo quando servono polimorfismo o test seam reali.

## Logging

Usare logger iniettato o infrastruttura centralizzata. Evitare `std::cout` in codice applicativo.

```cpp
logger.info("order created", {{"order_id", order_id.value()}});
```

## Cose da evitare

- `new` e `delete` manuali
- owning raw pointers
- distruttori che lanciano eccezioni
- conversioni implicite indesiderate
- `using namespace std;` negli header
- macro per logica applicativa
- lock/unlock manuali
- `std::shared_ptr` usato come default
- lifetime non documentati con `std::string_view`
