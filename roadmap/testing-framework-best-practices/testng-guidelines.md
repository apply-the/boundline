# TestNG Guidelines

## Principi

TestNG è utile per suite complesse, data provider, gruppi e test integration. La flessibilità però può creare suite difficili da capire e dipendenti dall’ordine.

## Test indipendenti

Anche se TestNG supporta dipendenze tra test, usarle con molta cautela.

### Da evitare

```java
@Test(dependsOnMethods = "createUser")
public void updateUser() {}
```

Meglio preparare il dato necessario nel setup del test.

## DataProvider

Usare DataProvider per casi tabellari.

```java
@DataProvider
public Object[][] invalidEmails() {
    return new Object[][] {
        {""},
        {"invalid"},
        {"missing-at.example.com"}
    };
}
```

## Groups

Usare gruppi per classificazione:

- unit
- integration
- smoke
- regression
- slow

Non usare gruppi per gestire dipendenze fragili.

## Setup/teardown

Tenere setup minimo e prevedibile.

- `@BeforeMethod` per stato isolato
- `@BeforeClass` per risorse costose immutabili
- cleanup sempre affidabile

## Parallel execution

Se si abilita parallelismo:

- niente stato statico mutabile
- test data univoci
- risorse isolate
- thread safety verificata

## Anti-pattern

- test dipendenti da ordine
- uso eccessivo di `dependsOnMethods`
- DataProvider enormi e illeggibili
- stato statico condiviso
- gruppi usati per nascondere lentezza della suite
