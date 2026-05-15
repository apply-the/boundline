# Robot Framework Guidelines

## Principi

Robot Framework è potente per acceptance test, keyword-driven testing e collaborazione QA/business. Il rischio è creare test verbosi, fragili e pieni di keyword generiche che nascondono troppo.

## Struttura

Separare:

- test cases
- resource files
- custom libraries
- variables
- page/resource keywords
- domain/business keywords

```text
tests/
resources/
libraries/
variables/
```

## Keyword design

Le keyword devono essere semantiche e leggibili.

### Debole

```robot
Click Button    xpath=//*[@id="x"]/div[2]/button
Input Text      id=email    test@example.com
Click Button    id=submit
```

### Migliore

```robot
Login As Valid User
Create Order With One Product
Order Should Be Created Successfully
```

## Livelli di keyword

Mantenere livelli chiari:

- low-level: click, input, HTTP call
- page-level: fill login form, submit order
- business-level: login as admin, approve invoice

Non mischiare livelli nello stesso test.

## Locator

Centralizzare locator fragili in resource/page object keyword.

Preferire locator stabili:

- data-testid
- role/name se supportato dalla library
- id stabili

Evitare xpath lunghi.

## Test data

Usare variabili e factory keyword.

Non hardcodare dati condivisi che causano collisioni.

## Setup e teardown

Ogni suite/test deve preparare e pulire il proprio stato.

```robot
Test Setup       Prepare Clean User
Test Teardown    Cleanup Test Data
```

## Assertions

Usare assert di business, non solo assert tecnici.

### Debole

```robot
Page Should Contain    Success
```

### Migliore

```robot
Order Status Should Be    ${order_id}    CONFIRMED
```

## Tags

Usare tag per organizzare:

- smoke
- regression
- critical
- api
- ui
- slow
- flaky-quarantined

## Custom libraries

Quando le keyword diventano troppo complesse, spostare logica in librerie Python/Java/Kotlin.

Robot deve orchestrare, non contenere algoritmi complessi.

## Anti-pattern

- keyword troppo generiche come `Do Login Stuff`
- xpath lunghi nei test case
- sleep fissi
- test case enormi
- logica complessa in Robot invece di library
- dati condivisi fra test
- keyword con troppi argomenti posizionali
