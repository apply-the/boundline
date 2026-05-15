# Cucumber and BDD Guidelines

## Principi

BDD non significa scrivere test in inglese. Significa creare esempi condivisi del comportamento atteso. Cucumber è utile solo se Gherkin resta leggibile da business, QA e sviluppo.

## Feature file

Scrivere scenari a livello di comportamento, non di UI dettaglio.

### Da evitare

```gherkin
Scenario: Login
  Given I open Chrome
  And I click the input with id "email"
  And I type "user@example.com"
  And I click the button with class "btn-primary"
```

### Preferibile

```gherkin
Scenario: Successful login
  Given a registered user exists
  When the user logs in with valid credentials
  Then the user should see their dashboard
```

## Given, When, Then

- Given: contesto
- When: azione
- Then: risultato osservabile

Non usare Given per azioni principali.

## Step riusabili ma non troppo generici

Troppa riusabilità rende gli step ambigui e fragili.

### Debole

```gherkin
When I click "Submit"
```

### Migliore

```gherkin
When the customer submits the order
```

## Scenario Outline

Usare per variazioni piccole e leggibili.

```gherkin
Scenario Outline: Invalid login
  Given a registered user exists
  When the user logs in with "<password>"
  Then the login should be rejected

Examples:
  | password |
  | wrong    |
  | empty    |
```

## Step definitions

Step definitions devono delegare a helper/page objects/service client. Non contenere logica enorme.

## Test data

Creare dati via API o factory, non manualmente via UI se non è il comportamento testato.

## BDD anti-pattern

- Gherkin usato come linguaggio di scripting UI
- feature file scritti solo dagli sviluppatori e mai letti dal business
- step con regex troppo generiche
- duplicazione semantica di step simili
- scenari lunghi e fragili
- usare Cucumber per unit test
