# Detox and Appium Guidelines

## Principi

Detox e Appium servono per E2E mobile. Sono preziosi per fiducia sui flussi critici, ma costosi da mantenere. La suite deve essere piccola, stabile e ben isolata.

## Detox

Buono per React Native e test gray-box.

Regole:

- testare flussi critici
- usare testID stabili
- reset app state tra test
- evitare dipendenza da ordine
- controllare permessi e deep link
- evitare sleep

## Appium

Buono per black-box mobile cross-platform.

Regole:

- locator stabili
- evitare XPath fragili
- capability gestite per ambiente
- device farm configurata
- timeout espliciti
- test data isolati

## Selettori

Usare accessibility id/testID.

### Da evitare

- XPath lunghi
- coordinate tap
- testo fragile se localizzato

## Stato app

Ogni test deve partire da stato noto:

- reinstall/reset app
- clear storage
- seeded backend
- auth state controllato

## Rete

Gestire:

- backend test environment
- dati univoci
- retry controllati
- offline test quando richiesto

## Flakiness

Cause comuni:

- animazioni
- timing device/emulator
- rete
- permission dialog
- keyboard
- dati condivisi
- locator fragili

## Anti-pattern

- E2E mobile per ogni caso
- XPath fragile
- tap per coordinate
- sleep
- ambiente backend condiviso sporco
- test non parallel-safe
