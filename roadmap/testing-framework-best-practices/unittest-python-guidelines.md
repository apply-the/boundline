# Python unittest Guidelines

## Principi

`unittest` è stabile e integrato nella standard library. È più verboso di pytest, quindi serve disciplina per mantenere test leggibili.

## TestCase

```python
class OrderServiceTest(unittest.TestCase):
    def test_rejects_empty_order(self) -> None:
        ...
```

## setUp

Usare `setUp` per setup breve e comune.

Se `setUp` diventa lungo, spostare in factory/helper.

## Assertions

Usare assert specifici.

```python
self.assertEqual(expected, actual)
self.assertRaises(InvalidOrderError)
```

## Mock

`unittest.mock` è potente ma facile da abusare.

Regole:

- patchare dove il simbolo viene usato, non dove è definito
- preferire dependency injection
- evitare patch globali complesse
- usare fake per repository semplici

## subTest

Usare `subTest` per casi tabellari.

```python
for value in invalid_values:
    with self.subTest(value=value):
        self.assertFalse(is_valid_email(value))
```

## Cleanup

Usare `addCleanup` per cleanup affidabile.

## Anti-pattern

- `setUp` enorme
- patch annidate difficili da leggere
- mock di tutto
- test dipendenti da ordine
- assert generici e poco informativi
