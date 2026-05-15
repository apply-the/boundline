# C Guidelines

## Principi

C è potente ma non perdona. La disciplina su ownership, gestione memoria, error handling e API design è obbligatoria. Non esiste runtime che salvi da use-after-free, buffer overflow o risorse non rilasciate.

## Ownership esplicita

Ogni API deve chiarire chi possiede la memoria e chi deve liberarla.

### Da evitare

```c
char *read_name();
```

Non è chiaro se il chiamante debba fare `free`.

### Preferibile

```c
// Caller owns the returned pointer and must free it with free().
char *read_name_alloc(void);
```

Oppure usare buffer fornito dal chiamante:

```c
int read_name(char *buffer, size_t buffer_len);
```

## Inizializzazione e cleanup simmetrici

Per ogni funzione `init`, prevedere una funzione `destroy` o `deinit`.

```c
typedef struct {
    FILE *file;
    char *buffer;
} parser_t;

int parser_init(parser_t *parser, const char *path);
void parser_destroy(parser_t *parser);
```

Regole:

- `destroy` deve tollerare stato parzialmente inizializzato se possibile.
- Dopo `free`, impostare il puntatore a `NULL` quando riduce il rischio di riuso.
- Documentare ownership di ogni campo.

## Gestione errori

Usare codici di errore coerenti. Non mescolare convenzioni senza motivo.

```c
typedef enum {
    ORDER_OK = 0,
    ORDER_ERR_INVALID_INPUT,
    ORDER_ERR_NOT_FOUND,
    ORDER_ERR_IO
} order_result_t;
```

### Preferibile

```c
order_result_t order_save(order_repository_t *repository, const order_t *order);
```

Regole:

- controllare sempre i return code
- non usare `errno` per errori di dominio
- preservare contesto quando possibile
- non chiamare `exit` dentro librerie

## Cleanup con `goto`

In C, `goto cleanup` è spesso la soluzione più chiara per rilasciare risorse in ordine.

```c
int process_file(const char *path) {
    FILE *file = NULL;
    char *buffer = NULL;
    int result = -1;

    file = fopen(path, "r");
    if (file == NULL) {
        goto cleanup;
    }

    buffer = malloc(4096);
    if (buffer == NULL) {
        goto cleanup;
    }

    result = 0;

cleanup:
    free(buffer);
    if (file != NULL) {
        fclose(file);
    }
    return result;
}
```

Non è RAII, ma evita duplicazione e leak.

## Buffer e stringhe

Le stringhe C sono una fonte costante di bug.

### Regole

- passare sempre la lunghezza del buffer
- preferire `snprintf` a `sprintf`
- evitare `strcpy` e `strcat`
- controllare troncamenti
- non assumere che input esterni siano null-terminated

```c
int written = snprintf(buffer, buffer_len, "%s", value);
if (written < 0 || (size_t)written >= buffer_len) {
    return ORDER_ERR_INVALID_INPUT;
}
```

## Tipi semantici

Usare struct dedicate per identificativi e concetti importanti.

```c
typedef struct {
    char value[37];
} order_id_t;
```

Non passare `char *` ovunque senza semantica.

## API design

Preferire API che rendono chiari input, output e ownership.

```c
typedef struct {
    order_id_t id;
    customer_id_t customer_id;
} order_t;

order_result_t order_create(
    const customer_id_t *customer_id,
    order_t *out_order
);
```

Regole:

- usare `const` per input non modificati
- usare `out_` per output pointer
- validare puntatori in ingresso
- evitare side effect non documentati

## Testabilità

Separare logica pura da I/O. Passare dipendenze come function pointer o struct di callback quando serve.

```c
typedef struct {
    order_result_t (*save)(void *ctx, const order_t *order);
    void *ctx;
} order_repository_t;
```

Questo permette fake repository nei test.

## Logging

Non usare `printf` ovunque nella libreria. Passare un logger o callback dal chiamante.

```c
typedef void (*log_fn_t)(void *ctx, const char *message);
```

## Cose da evitare

- `gets`, `sprintf`, `strcpy`, `strcat`
- `malloc` senza ownership documentata
- `exit` dentro librerie
- global mutable state
- buffer senza lunghezza
- cast inutili da `malloc`
- ignorare return code
- macro complesse che nascondono side effect
- use-after-free e double-free non prevenuti da convenzioni chiare
