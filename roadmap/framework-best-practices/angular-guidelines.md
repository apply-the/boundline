# Angular Guidelines

## Principi

Angular funziona bene quando si rispettano dependency injection, moduli/standalone components coesi, reactive forms, RxJS usato con disciplina e smart/dumb components dove utile.

## Componenti

Separare:

- container components: orchestrano dati e casi d’uso UI
- presentational components: ricevono input ed emettono eventi
- services: I/O e application logic
- domain logic fuori dai componenti

## Standalone components

Nei nuovi progetti, preferire standalone components se lo stack lo consente.

```ts
@Component({
  standalone: true,
  selector: "app-order-card",
  templateUrl: "./order-card.component.html",
})
export class OrderCardComponent {}
```

## Dependency injection

Usare DI nativa. Non creare direttamente servizi dentro componenti.

### Da evitare

```ts
private service = new OrderService();
```

### Preferibile

```ts
constructor(private readonly orderService: OrderService) {}
```

## RxJS

Usare RxJS con intenzione.

Regole:

- evitare subscribe annidati
- preferire pipe con operatori
- gestire errori
- usare `async` pipe quando possibile
- cancellare subscription manuali o usare `takeUntilDestroyed`
- evitare Subject come variabile globale mutabile

### Da evitare

```ts
this.userService.getUser().subscribe(user => {
  this.orderService.getOrders(user.id).subscribe(orders => {
    this.orders = orders;
  });
});
```

### Preferibile

```ts
orders$ = this.userService.getUser().pipe(
  switchMap(user => this.orderService.getOrders(user.id)),
);
```

## Forms

Usare reactive forms per form complessi.

Regole:

- validatori sincroni/asincroni chiari
- errori accessibili
- mapping DTO separato
- non mettere tutta la business logic nel component

## Change detection

Preferire `OnPush` dove possibile.

```ts
changeDetection: ChangeDetectionStrategy.OnPush
```

## State management

Non introdurre NgRx o store complessi senza necessità.

Usare:

- component state per locale
- service state per feature piccole
- NgRx/Akita/Signal Store per stato globale complesso
- query/caching library per server state se adatta

## Signals

Usare signals per stato locale e derivato quando coerente con la versione Angular adottata. Non mischiare paradigmi senza regole di team.

## Testing

Testare componenti con TestBed quando serve integrazione Angular. Testare servizi e logica come unit test normali.

## Anti-pattern

- componenti con troppa logica
- subscribe annidati
- subscription non cancellate
- servizi singleton con stato mutabile incontrollato
- moduli shared enormi
- template con logica complessa
- NgRx per ogni checkbox
