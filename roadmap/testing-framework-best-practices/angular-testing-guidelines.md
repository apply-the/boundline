# Angular Testing Guidelines

## Principi

I test Angular devono distinguere chiaramente fra unit test di logica TypeScript, component test con TestBed e integration test del framework.

## TestBed con criterio

Usare TestBed quando serve:

- template rendering
- dependency injection Angular
- pipe/directive integration
- change detection
- component lifecycle

Non usarlo per testare funzioni pure o service semplici senza dipendenze Angular.

## Component test

Testare comportamento visibile.

```ts
it("disables submit when the form is invalid", () => {
  // arrange
  // act
  // assert
});
```

Preferire Angular Testing Library quando si vuole approccio user-centric.

## Services

Per service con logica pura, istanziare direttamente.

```ts
const service = new PriceCalculator();
```

Per service con HttpClient, usare `HttpTestingController`.

## RxJS

Testare stream con:

- marble testing per logica Rx complessa
- `firstValueFrom` per casi semplici
- fakeAsync/tick con criterio

## fakeAsync

Usare `fakeAsync` quando si controllano timer o microtask.

Non mischiare male `fakeAsync`, `async/await` e promise reali.

## HTTP

```ts
const req = httpMock.expectOne("/api/orders");
expect(req.request.method).toBe("GET");
req.flush(orderResponse);
```

Verificare che non restino richieste pendenti.

## Anti-pattern

- TestBed per tutto
- test accoppiati a classi CSS
- subscribe senza assert
- subscription non gestite
- fakeAsync usato senza capire task queue
- snapshot invece di assert comportamentali
