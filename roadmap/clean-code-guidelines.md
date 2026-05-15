# Clean Code Guidelines

## Obiettivo

Clean Code non significa codice “bello” in senso estetico. Significa codice che può essere letto, modificato, testato e rilasciato con rischio basso anche mesi dopo che è stato scritto.

Il punto non è seguire regole religiose. Il punto è ridurre ambiguità, accoppiamento, duplicazione nascosta, stati invalidi, side effect imprevedibili e complessità accidentale.

## Principi guida

### 1. Il codice si legge più spesso di quanto si scriva

Ottimizzare solo per scrivere velocemente è miope. Una scorciatoia oggi può diventare costo ricorrente in ogni modifica futura.

Preferire codice che rende chiari:

- intenzione
- dominio
- flusso degli errori
- ownership delle risorse
- dipendenze
- side effect
- invarianti

### 2. Semplice non vuol dire semplicistico

Una soluzione semplice ha pochi concetti essenziali. Una soluzione semplicistica ignora vincoli reali e scarica il costo sul futuro.

### 3. Il codice deve rendere difficili gli errori comuni

Un buon design non si limita a documentare cosa non fare. Rende difficile farlo.

Esempio:

```rust
fn transfer(from: AccountId, to: AccountId, amount: Money) -> Result<(), TransferError>
```

è meglio di:

```rust
fn transfer(from: String, to: String, amount: f64) -> Result<(), String>
```

perché impedisce più classi di errori già a compile time.

---

# 1. Naming

## Nomi intenzionali

Un nome deve spiegare perché esiste una cosa, non solo che tipo tecnico ha.

### Da evitare

```ts
const data = await fetchUser();
const tmp = data.createdAt;
const flag = user.age > 18;
```

### Preferibile

```ts
const userProfile = await fetchUserProfile();
const accountCreationDate = userProfile.createdAt;
const isAdult = userProfile.age > 18;
```

## Nomi coerenti nel dominio

Scegliere un vocabolario e mantenerlo.

Se nel dominio si parla di `Customer`, non alternare:

- `Customer`
- `Client`
- `User`
- `Account`
- `Subscriber`

a meno che siano concetti davvero diversi.

## Nomi pronunciabili e cercabili

Evitare abbreviazioni criptiche.

### Da evitare

```go
func CalcUsrDisc(c Customer) Money
```

### Preferibile

```go
func CalculateCustomerDiscount(customer Customer) Money
```

## Evitare nomi ingannevoli

Non chiamare `list` qualcosa che è una mappa, non chiamare `cache` qualcosa che è la source of truth.

### Da evitare

```java
Map<String, User> userList;
```

### Preferibile

```java
Map<UserId, User> usersById;
```

## Nome della funzione: verbo + oggetto

Le funzioni dovrebbero comunicare azione e risultato.

Esempi buoni:

```text
calculateInvoiceTotal
parseUserProfile
sendPasswordResetEmail
reserveInventory
markOrderAsPaid
```

Esempi deboli:

```text
process
handle
manage
doWork
execute
run
```

Questi nomi non sono sempre sbagliati, ma spesso indicano che la funzione ha responsabilità confuse.

---

# 2. Magic Numbers, Magic Strings e valori impliciti

## No magic numbers

### Da evitare

```rust
if retry_count > 3 {
    return Err(Error::TooManyRetries);
}
```

### Preferibile

```rust
const MAX_RETRY_ATTEMPTS: u32 = 3;

if retry_count > MAX_RETRY_ATTEMPTS {
    return Err(Error::TooManyRetries);
}
```

## No magic strings

### Da evitare

```ts
if (user.role === "admin") {
  grantAccess();
}
```

### Preferibile

```ts
enum UserRole {
  Admin = "admin",
  Member = "member",
}

if (user.role === UserRole.Admin) {
  grantAccess();
}
```

## Preferire enum o union per stati chiusi

Se i valori ammessi sono finiti, modellali.

### Da evitare

```java
String paymentStatus;
```

### Preferibile

```java
enum PaymentStatus {
    PENDING,
    COMPLETED,
    FAILED
}
```

## Attenzione alle costanti inutili

Non ogni valore numerico è “magico”.

### Accettabile

```python
area = width * height
half = total / 2
```

### Da nominare

```python
timeout_seconds = 30
max_retry_attempts = 3
vat_rate = Decimal("0.22")
```

La regola pratica: se il valore rappresenta una decisione di dominio, configurazione o policy, dagli un nome.

---

# 3. Funzioni

## Piccole, ma non ridicole

“Funzioni sotto le 20 righe” può essere un buon odore, non una legge. Una funzione da 35 righe chiara è meglio di 12 funzioni artificiali da 3 righe che costringono a saltare avanti e indietro.

La domanda giusta è: la funzione ha una responsabilità chiara?

## Una funzione deve fare una cosa

### Da evitare

```ts
async function createOrder(request: Request) {
  const body = await request.json();
  validate(body);
  const order = buildOrder(body);
  await database.save(order);
  await emailClient.send(order.customerEmail, "Order created");
  logger.info({ orderId: order.id }, "Order created");
  return new Response(JSON.stringify(order));
}
```

Questa funzione fa parsing, validazione, dominio, persistenza, notifica, logging e HTTP response.

### Preferibile

```ts
async function createOrderHandler(request: Request): Promise<Response> {
  const command = await parseCreateOrderRequest(request);
  const result = await orderService.createOrder(command);

  return toCreateOrderResponse(result);
}
```

Il service potrà poi coordinare il caso d’uso, ma il boundary HTTP resta separato.

## Livello di astrazione coerente

Non mescolare nella stessa funzione concetti di alto livello e dettagli bassi.

### Da evitare

```java
void createOrder(CreateOrderRequest request) {
    if (request.lines().isEmpty()) {
        throw new ValidationException("empty order");
    }

    String sql = "insert into orders...";
    jdbcTemplate.update(sql);

    smtpClient.send(...);
}
```

### Preferibile

```java
void createOrder(CreateOrderCommand command) {
    Order order = Order.create(command);
    orderRepository.save(order);
    notificationService.notifyOrderCreated(order);
}
```

## Pochi argomenti

Molti parametri rendono fragile l’ordine, complicano i test e spesso rivelano un concetto mancante.

### Da evitare

```go
func CreateUser(name string, email string, age int, country string, newsletter bool, source string) error
```

### Preferibile

```go
type CreateUserCommand struct {
    Name string
    Email EmailAddress
    Age int
    Country CountryCode
    NewsletterConsent bool
    Source RegistrationSource
}

func CreateUser(command CreateUserCommand) error
```

## Evitare boolean flag nei parametri

Un boolean spesso nasconde due comportamenti.

### Da evitare

```csharp
invoice.CalculateTotal(true);
```

Cosa significa `true`?

### Preferibile

```csharp
invoice.CalculateTotalIncludingTaxes();
invoice.CalculateTotalExcludingTaxes();
```

Oppure:

```csharp
invoice.CalculateTotal(TaxMode.IncludeTaxes);
```

## Command Query Separation

Una funzione dovrebbe idealmente o modificare stato, o restituire dati. Fare entrambe le cose crea sorprese.

### Da evitare

```ruby
def valid?
  @validated_at = Time.now
  errors.empty?
end
```

### Preferibile

```ruby
def valid?
  errors.empty?
end

def mark_validated(clock:)
  @validated_at = clock.now
end
```

## Side effect espliciti

Se una funzione scrive su database, invia eventi, manda email, modifica file o aggiorna stato globale, deve essere evidente dal nome o dal contesto.

### Debole

```python
process_order(order)
```

### Migliore

```python
reserve_inventory_and_publish_order_created(order)
```

Meglio ancora: separare le responsabilità.

---

# 4. Struttura del codice

## Separare boundary, application logic e dominio

Una struttura sana distingue:

- input/output: HTTP, CLI, consumer Pub/Sub, filesystem
- application service: orchestration dei casi d’uso
- dominio: regole, invarianti, decisioni
- infrastruttura: database, API esterne, code, cache
- mapping: DTO verso domain model e viceversa

### Esempio

```text
src/
  domain/
    order.rs
    payment.rs
  application/
    create_order.rs
  infrastructure/
    postgres_order_repository.rs
    stripe_payment_client.rs
  interfaces/
    http/
      create_order_handler.rs
```

Il naming cambia per linguaggio, ma la separazione resta utile.

## Dipendenze verso l’interno

Il dominio non dovrebbe dipendere da HTTP, database, framework o SDK esterni.

### Da evitare

```java
class Order {
    @JsonProperty("order_id")
    private String id;

    @Entity
    @Table(name = "orders")
    ...
}
```

Il modello di dominio diventa vincolato a JSON e database.

### Preferibile

- DTO HTTP separati
- entity persistence separate quando necessario
- mapper espliciti
- domain model libero da framework

## Composition root

Le dipendenze vanno costruite in un punto esplicito dell’applicazione, non sparse ovunque.

### Da evitare

```ts
class OrderService {
  private repository = new PostgresOrderRepository();
  private paymentClient = new StripePaymentClient();
}
```

### Preferibile

```ts
const repository = new PostgresOrderRepository(database);
const paymentClient = new StripePaymentClient(httpClient);
const orderService = new OrderService(repository, paymentClient);
```

---

# 5. Modellazione del dominio

## Primitive obsession

Usare primitive per tutto rende facile confondere concetti diversi.

### Da evitare

```kotlin
fun findOrder(customerId: String, orderId: String)
```

### Preferibile

```kotlin
@JvmInline
value class CustomerId(val value: String)

@JvmInline
value class OrderId(val value: String)

fun findOrder(customerId: CustomerId, orderId: OrderId)
```

## Value object

Un value object rappresenta un concetto con regole proprie.

Esempi:

- `EmailAddress`
- `Money`
- `OrderId`
- `Percentage`
- `CountryCode`
- `DateRange`
- `NonEmptyString`

### Esempio

```java
public record EmailAddress(String value) {
    public EmailAddress {
        if (value == null || !value.contains("@")) {
            throw new IllegalArgumentException("Invalid email address");
        }
    }
}
```

## Rendere impossibili gli stati invalidi

### Da evitare

```ts
type Payment = {
  isPending: boolean;
  isCompleted: boolean;
  isFailed: boolean;
  transactionId?: string;
  failureReason?: string;
};
```

Può rappresentare stati impossibili.

### Preferibile

```ts
type Payment =
  | { kind: "pending" }
  | { kind: "completed"; transactionId: TransactionId }
  | { kind: "failed"; reason: string };
```

## Validare alla costruzione

Non permettere che oggetti invalidi circolino.

### Da evitare

```php
$order = new Order($lines);
if (count($order->lines) === 0) {
    throw new InvalidOrderException();
}
```

### Preferibile

```php
$order = Order::create($lines);
```

e `Order::create` rifiuta liste vuote.

## DTO non è dominio

Un DTO descrive formato dati a un boundary. Non dovrebbe contenere logica di dominio e non dovrebbe essere passato ovunque.

### Regola pratica

- DTO: shape dell’input/output
- Command: richiesta applicativa validata
- Domain model: regole e invarianti
- Persistence model: come salvo/carico

---

# 6. Error handling

## Distinguere bug, errori attesi ed errori infrastrutturali

Non tutti gli errori sono uguali.

### Bug

Esempi:

- invariant impossibile violata
- branch logicamente irraggiungibile
- dato interno corrotto

Può essere accettabile fallire forte.

### Errore atteso

Esempi:

- utente non trovato
- ordine vuoto
- pagamento rifiutato
- input non valido

Va modellato e gestito.

### Errore infrastrutturale

Esempi:

- database non disponibile
- timeout HTTP
- disco pieno
- broker irraggiungibile

Va propagato con contesto, loggato al boundary giusto e osservato.

## Non usare errori generici

### Da evitare

```rust
Result<Order, String>
```

### Preferibile

```rust
Result<Order, FindOrderError>
```

## Non ingoiare errori

### Da evitare

```python
try:
    send_email(message)
except Exception:
    pass
```

### Preferibile

```python
try:
    send_email(message)
except EmailDeliveryError as error:
    logger.warning("Failed to send email", extra={"error": str(error)})
    return EmailResult.failed(error)
```

## Non loggare e rilanciare ovunque

### Da evitare

```java
try {
    repository.save(order);
} catch (Exception e) {
    logger.error("Failed to save order", e);
    throw e;
}
```

Se ogni layer fa così, si ottengono log duplicati e rumorosi.

### Preferibile

- aggiungere contesto quando si propaga
- loggare al boundary applicativo o dove si prende una decisione
- evitare duplicazione inutile

## Aggiungere contesto

### Go

```go
if err != nil {
    return fmt.Errorf("save order %s: %w", orderID, err)
}
```

### Rust

```rust
repository
    .save(order)
    .await
    .context("save order")?;
```

## Panic, exit e crash

`panic`, `exit`, `fatal` e simili non devono comparire nella logica applicativa normale.

Accettabili:

- `main`
- test
- bootstrap irrecuperabile
- invariant impossibile e documentata
- tool CLI dove fallire è il comportamento atteso

Non accettabili:

- repository
- service
- domain logic
- librerie riusabili
- handler HTTP

---

# 7. Gestione risorse

## Cleanup deterministico

Le risorse devono essere rilasciate anche in caso di errore.

Usare il meccanismo idiomatico del linguaggio:

| Linguaggio | Meccanismo |
| --- | --- |
| Rust | ownership, `Drop` |
| C++ | RAII |
| C | `goto cleanup`, init/destroy |
| Go | `defer` |
| Java | `try-with-resources` |
| C# | `using`, `await using` |
| Python | context manager |
| Kotlin | `use` |
| Scala | `Using`, `Resource`, `ZIO.acquireRelease` |
| Ruby | block API |
| Shell | `trap` |
| PowerShell | `try/finally` |

## Acquisizione vicina al cleanup

### Go

```go
file, err := os.Open(path)
if err != nil {
    return err
}
defer file.Close()
```

## Non nascondere ownership

Se una funzione ritorna una risorsa da chiudere o memoria da liberare, deve essere evidente dal tipo, nome o documentazione.

---

# 8. DRY, WET e astrazioni

## DRY non significa eliminare ogni somiglianza

Duplicazione di codice e duplicazione di conoscenza non sono la stessa cosa.

Due blocchi simili possono rappresentare concetti diversi che oggi coincidono ma domani divergeranno.

### Duplicazione tollerabile

```ts
validateBillingAddress(...)
validateShippingAddress(...)
```

Se le regole possono divergere, non fonderle troppo presto.

## Wrong Abstraction è peggio di duplicazione

Un’astrazione sbagliata crea coupling artificiale.

### Segnali di astrazione sbagliata

- tanti flag booleani
- nomi generici
- parametri opzionali crescenti
- casi speciali ovunque
- chiamanti costretti ad adattarsi
- test difficili da leggere

## Rule of Three

Prima occorrenza: scrivi semplice.

Seconda: nota la somiglianza.

Terza: valuta l’astrazione.

Non è una legge, ma evita astrazioni premature.

---

# 9. Coupling e coesione

## Alta coesione

Un modulo dovrebbe contenere cose che cambiano per la stessa ragione.

### Debole

```text
utils/
  date.ts
  money.ts
  retry.ts
  validation.ts
  email.ts
```

### Migliore

```text
billing/
  money.ts
  invoice.ts
notifications/
  email.ts
resilience/
  retry.ts
```

## Basso accoppiamento

Un modulo dovrebbe conoscere il minimo necessario degli altri moduli.

## Legge di Demetra

Evita catene lunghe.

### Da evitare

```java
order.getCustomer().getAccount().getBillingProfile().getAddress().getCountry()
```

### Preferibile

```java
order.billingCountry()
```

o spostare la logica dove appartiene.

## Nascondere dettagli interni

Esporre solo ciò che serve.

### Rust

```rust
pub struct Order {
    id: OrderId,
    lines: Vec<OrderLine>,
}
```

Non rendere `pub` ogni campo per comodità.

---

# 10. Commenti e documentazione

## I commenti non compensano codice confuso

### Da evitare

```csharp
// Check if the user is active
if (user.Status == 1) {
    ...
}
```

### Preferibile

```csharp
if (user.IsActive) {
    ...
}
```

## Commentare il perché, non il cosa

### Buon commento

```go
// The provider may send duplicate events during retries.
// Store the event id before processing to make the handler idempotent.
```

### Cattivo commento

```go
// Increment i by one.
i++
```

## Documentare decisioni architetturali

Usare ADR o note tecniche per decisioni importanti:

- scelta database
- strategia retry
- consistency model
- trade-off di performance
- vincoli di sicurezza
- compromessi temporanei

## TODO

Un TODO senza owner o contesto è debito invisibile.

### Da evitare

```ts
// TODO fix this
```

### Preferibile

```ts
// TODO(order-import): remove fallback after legacy importer is decommissioned.
```

---

# 11. Formatting e stile

## Usare formatter automatici

Non discutere in code review di spazi, indentazione e virgole.

Usare strumenti come:

- `rustfmt`
- `gofmt`
- `prettier`
- `black`
- `ktlint`
- `scalafmt`
- `clang-format`
- `php-cs-fixer`
- `rubocop`

## Stile coerente batte stile personale

Un codebase deve sembrare scritto da un team, non da dieci individui scollegati.

## File ordinati

Un file leggibile spesso segue un ordine coerente:

1. tipi pubblici
2. costruttori/factory
3. metodi pubblici
4. metodi privati
5. test

La convenzione varia, ma deve essere prevedibile.

---

# 12. Testabilità

## Testabilità come proprietà del design

Se testare richiede patch globali, sleep, reflection o bootstrap completo del mondo, il design sta mandando un segnale.

## Separare logica pura da side effect

### Preferibile

```python
def calculate_total(lines: list[OrderLine], tax_policy: TaxPolicy) -> Money:
    ...
```

piuttosto che:

```python
def calculate_total(order_id: str) -> Money:
    order = database.load(order_id)
    tax_policy = http_client.get_tax_policy(order.country)
    ...
```

## Dipendenze iniettate

Passare dipendenze dall’esterno:

- repository
- client HTTP
- clock
- ID generator
- random generator
- logger
- config
- message publisher

## Clock fittizio

### Da evitare

```java
Instant.now()
```

sparso nella logica.

### Preferibile

```java
class ExpirationPolicy {
    private final Clock clock;

    ExpirationPolicy(Clock clock) {
        this.clock = clock;
    }
}
```

## Testare comportamento, non implementazione

### Da evitare

- verificare ogni metodo privato
- test accoppiati all’ordine interno delle chiamate
- mock di ogni dipendenza anche quando una fake sarebbe più chiara

### Preferibile

- input chiaro
- output o stato osservabile
- fake esplicite
- test di casi limite
- test di error path

## Pyramid sensata

Una base ampia di test veloci e pochi test end-to-end costosi.

Indicativamente:

- molti unit test su logica pura
- alcuni integration test su database, code, API client
- pochi end-to-end test sui flussi critici

---

# 13. Concorrenza e async

## Structured concurrency

Task, goroutine, thread e coroutine devono avere owner chiaro e modo di terminare.

### Da evitare

```kotlin
GlobalScope.launch {
    processOrder(order)
}
```

### Preferibile

```kotlin
coroutineScope {
    launch {
        processOrder(order)
    }
}
```

## Non perdere errori async

### TypeScript

```ts
void backgroundTask().catch((error) => {
  logger.error({ error }, "Background task failed");
});
```

## Timeout e cancellation

Ogni chiamata remota dovrebbe avere:

- timeout
- cancellation
- retry policy se appropriata
- logging/tracing
- idempotenza dove serve

## Non bloccare runtime async

Esempi:

- non usare `Thread.sleep` in coroutine
- non usare I/O blocking nel reactor/event loop
- non tenere lock attraverso `await`
- non fare CPU intensive work su thread pool I/O

---

# 14. Logging e osservabilità

## Structured logging

### Da evitare

```ts
logger.info(`Created order ${orderId} for customer ${customerId}`);
```

### Preferibile

```ts
logger.info({ orderId, customerId }, "Order created");
```

## Loggare decisioni, non rumore

Log utili:

- richiesta ricevuta
- comando importante eseguito
- chiamata esterna fallita
- retry esauriti
- evento pubblicato
- stato inconsistente rilevato

Log poco utili:

- “entered function”
- “x = 1”
- log in loop ad alta frequenza
- duplicazione dello stesso errore in ogni layer

## Correlation ID

Ogni log di request o message processing dovrebbe essere correlabile.

Campi tipici:

- `trace_id`
- `span_id`
- `request_id`
- `correlation_id`
- `user_id`, solo se consentito
- `tenant_id`, se rilevante

## Non loggare segreti

Mai loggare:

- password
- token
- API key
- cookie sessione
- dati personali non necessari
- payload completi contenenti dati sensibili

---

# 15. Configurazione

## Config letta una volta

Leggere environment/configuration al bootstrap, validare, trasformare in oggetto tipizzato e passare alle dipendenze.

### Da evitare

```go
func HandleRequest() {
    timeout := os.Getenv("PAYMENT_TIMEOUT")
}
```

### Preferibile

```go
type PaymentConfig struct {
    Timeout time.Duration
}

func NewPaymentClient(config PaymentConfig) *PaymentClient {
    ...
}
```

## Fail fast su config invalida

Meglio fallire in avvio che scoprire dopo tre ore che una variabile era sbagliata.

## Non mischiare config e dominio

Il dominio dovrebbe ricevere policy già interpretate, non leggere environment.

---

# 16. Sicurezza

## Validare input esterno

Ogni input esterno è non fidato:

- HTTP request
- messaggio da queue
- file
- CSV
- variabile ambiente
- webhook
- dati da database legacy
- risposta di API esterna

## Least privilege

Ogni componente deve avere i permessi minimi necessari.

## Evitare concatenazione in query e comandi

### SQL

Usare query parametrizzate.

### Shell

Non costruire comandi concatenando stringhe con input utente.

## Gestione segreti

- non hardcodare segreti
- non committare `.env`
- non loggare segreti
- usare secret manager o meccanismo di piattaforma
- ruotare credenziali

---

# 17. Performance e clean code

## Clean code non significa ignorare performance

Codice pulito deve anche rendere visibili i costi.

### Rendere espliciti

- chiamate remote
- query database
- allocazioni grandi
- loop su dataset grandi
- serializzazione/deserializzazione
- lock
- retry
- cache

## Evitare micro-ottimizzazioni premature

Prima misurare, poi ottimizzare.

## Ma evitare inefficienze ovvie

### Da evitare

```python
for user in users:
    orders = repository.find_orders_by_user(user.id)
```

Se produce N+1 query, è un problema reale, non micro-ottimizzazione.

## Performance-sensitive code

Quando il codice è volutamente meno leggibile per performance, commentare il perché e coprirlo con test/benchmark.

---

# 18. Refactoring

## Boy Scout Rule

Lasciare il codice leggermente migliore di come lo si è trovato.

Non significa riscrivere mezzo sistema durante una bugfix. Significa miglioramenti piccoli e sicuri:

- rinominare una variabile confusa
- estrarre una funzione
- aggiungere un test su un bug
- rimuovere duplicazione locale
- sostituire magic number
- ridurre nesting

## Refactoring sicuro

Prima di cambiare struttura:

1. aggiungere test se mancano
2. fare piccoli passi
3. mantenere comportamento
4. evitare refactor enormi mischiati a feature
5. usare strumenti automatici quando possibile

## Non rifattorizzare per gusto personale

Ogni refactoring deve ridurre rischio, complessità o costo futuro.

---

# 19. Code review

## Cosa guardare

- naming
- responsabilità
- error handling
- testabilità
- duplicazione
- stati invalidi
- side effect nascosti
- logging
- sicurezza
- performance ovvia
- coerenza con il dominio
- backward compatibility

## Commenti utili

### Debole

```text
Non mi piace.
```

### Utile

```text
Questo metodo fa sia validazione sia persistenza. Lo separerei in `ValidateCreateOrderCommand` e `OrderRepository.Save`, così possiamo testare la validazione senza database.
```

## Non trasformare la review in guerra di stile

Formatter, linter e linee guida devono assorbire discussioni meccaniche.

---

# 20. Anti-pattern frequenti

## God object

Una classe o modulo che sa tutto e fa tutto.

Segnali:

- centinaia o migliaia di righe
- molte dipendenze
- metodi non correlati
- test difficili
- cambi frequenti per motivi diversi

## Anemic domain model

Oggetti pieni solo di getter/setter e tutta la logica nei service.

Non sempre è sbagliato, ma spesso porta a service enormi e dominio poco protetto.

## Service locator

Nasconde dipendenze e rende i test peggiori.

### Da evitare

```csharp
var repository = ServiceLocator.Get<IOrderRepository>();
```

## Global mutable state

Rende il comportamento dipendente dall’ordine di esecuzione.

## Boolean blindness

Boolean senza significato nel punto di chiamata.

```java
sendEmail(user, true, false);
```

## Shotgun surgery

Una modifica richiede piccoli cambi in molti file. Indica duplicazione di conoscenza o accoppiamento eccessivo.

## Feature envy

Una funzione usa più dati di un altro oggetto che del proprio. Forse appartiene all’altro oggetto.

## Temporal coupling

Un oggetto deve essere usato in un ordine specifico non espresso dal tipo.

### Da evitare

```java
builder.setCustomer(customer);
builder.validate();
builder.calculate();
builder.save();
```

Se l’ordine è obbligatorio, modellarlo con API più sicura.

---

# 21. Checklist pratica

## Naming

- Il nome spiega l’intenzione?
- I termini sono coerenti con il dominio?
- Ci sono abbreviazioni inutili?
- Ci sono nomi generici come `data`, `info`, `manager`, `helper`, `processor`?

## Funzioni

- La funzione fa una cosa chiara?
- Ha troppi parametri?
- Ha boolean flag?
- Mescola livelli di astrazione?
- Nasconde side effect?

## Dominio

- Ci sono primitive che dovrebbero essere tipi semantici?
- Gli stati invalidi sono rappresentabili?
- La validazione avviene ai boundary o alla costruzione?
- DTO e dominio sono separati?

## Errori

- Gli errori attesi sono modellati?
- Ci sono `panic`, `unwrap`, `exit`, `fatal` o equivalenti fuori dai punti ammessi?
- Gli errori sono propagati con contesto?
- Ci sono catch generici o fallback silenziosi?

## Risorse

- Le risorse vengono sempre rilasciate?
- Il cleanup è vicino all’acquisizione?
- Ownership e lifetime sono chiari?

## Test

- La logica può essere testata senza rete/database?
- Clock, random e ID generator sono controllabili?
- I test verificano comportamento?
- Ci sono sleep o timing fragili?

## Logging

- I log sono strutturati?
- Esiste correlation ID?
- Si loggano segreti?
- Si logga lo stesso errore troppe volte?

## Architettura

- Il dominio dipende da framework o infrastruttura?
- Le dipendenze sono esplicite?
- C’è global mutable state?
- I moduli sono coesi?

---

# 22. Regole sintetiche da mettere in una policy interna

1. Usa nomi che esprimono intenzione, non implementazione casuale.
2. Niente magic numbers o magic strings per regole di dominio.
3. Preferisci enum, union, sealed class o value object per stati e concetti chiusi.
4. Evita primitive obsession.
5. Valida input esterni ai boundary.
6. Non far circolare DTO come modello di dominio.
7. Le funzioni devono avere responsabilità chiara e pochi argomenti.
8. Evita boolean flag nei parametri pubblici.
9. Rendi espliciti side effect e dipendenze.
10. Inietta dipendenze dal costruttore o dalla composition root.
11. Non creare client infrastrutturali dentro la business logic.
12. Non usare global mutable state.
13. Gestisci errori attesi con tipi o eccezioni specifiche.
14. Non ingoiare errori.
15. Non loggare e rilanciare lo stesso errore a ogni livello.
16. Non usare crash, panic o exit fuori da main/bootstrap/test/invariant documentate.
17. Usa cleanup deterministico secondo l’idioma del linguaggio.
18. Se una risorsa viene acquisita, deve essere chiaro chi la rilascia.
19. Separa logica pura da I/O.
20. Usa formatter e linter automatici.
21. Scrivi test su comportamento, non su dettagli interni.
22. Evita sleep nei test.
23. Usa logging strutturato con correlation ID.
24. Non loggare segreti.
25. Evita astrazioni premature.
26. Elimina duplicazione di conoscenza, non somiglianza superficiale.
27. Preferisci refactoring piccoli e sicuri.
28. Lascia il codice un po’ migliore di come lo hai trovato.
29. Documenta il perché delle decisioni non ovvie.
30. Quando una regola peggiora il codice, spiega il trade-off e rendilo esplicito.
