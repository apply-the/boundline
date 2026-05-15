# Zig Guidelines

## Principi

Zig rende espliciti allocatori, errori e controllo del flusso. Usarlo bene significa non nascondere queste decisioni dietro astrazioni premature.

## Allocator esplicito

Le funzioni che allocano devono ricevere un allocator o documentare chiaramente l’ownership.

```zig
pub fn readName(allocator: std.mem.Allocator) ![]u8 {
    return try allocator.dupe(u8, "example");
}
```

Il chiamante deve sapere che deve liberare la memoria.

```zig
const name = try readName(allocator);
defer allocator.free(name);
```

## Cleanup con `defer` ed `errdefer`

Usare `defer` per cleanup normale e `errdefer` per cleanup solo in caso di errore.

```zig
const buffer = try allocator.alloc(u8, 4096);
errdefer allocator.free(buffer);
```

Per risorse acquisite:

```zig
var file = try std.fs.cwd().openFile(path, .{});
defer file.close();
```

## Error handling

Usare error unions invece di crash.

```zig
const CreateOrderError = error{
    InvalidCustomer,
    EmptyOrder,
    RepositoryUnavailable,
};

pub fn createOrder(command: CreateOrderCommand) CreateOrderError!OrderId {
    // ...
}
```

Regole:

- usare `try` per propagare errori con chiarezza
- evitare `catch unreachable` salvo invarianti vere e documentate
- non usare `@panic` per errori attesi
- mantenere error set significativi

## Tipi semantici

Usare struct dedicate per concetti di dominio.

```zig
const OrderId = struct {
    value: []const u8,
};
```

Per valori con invariant, usare funzioni di costruzione.

```zig
const EmailAddress = struct {
    value: []const u8,

    pub fn parse(value: []const u8) !EmailAddress {
        if (std.mem.indexOfScalar(u8, value, '@') == null) {
            return error.InvalidEmailAddress;
        }

        return .{ .value = value };
    }
};
```

## No hidden allocation

Non nascondere allocazioni in funzioni che sembrano pure o leggere. In Zig l’allocatore esplicito è parte del contratto.

### Da evitare

```zig
pub fn formatOrder(order: Order) ![]u8 {
    // allocates internally using a global allocator
}
```

### Preferibile

```zig
pub fn formatOrder(allocator: std.mem.Allocator, order: Order) ![]u8 {
    // ...
}
```

## Testing

Usare `std.testing.allocator` per rilevare leak nei test.

```zig
test "creates valid order id" {
    const allocator = std.testing.allocator;
    const id = try OrderId.parse(allocator, "order-1");
    defer id.deinit(allocator);

    try std.testing.expectEqualStrings("order-1", id.value);
}
```

## API design

Usare `comptime` quando migliora sicurezza e performance, non come esercizio di stile.

Regole:

- distinguere slice owned e borrowed nei nomi o nella documentazione
- fornire `deinit` per tipi che possiedono risorse
- accettare `[]const u8` per input string-like
- non restituire slice a memoria locale
- documentare lifetime

## Logging

Usare `std.log` o logger coerente del progetto.

```zig
std.log.info("order created: {s}", .{order_id.value});
```

Non loggare segreti.

## Cose da evitare

- allocator globali nascosti
- `@panic` per errori recuperabili
- `catch unreachable` usato per comodità
- memoria allocata senza `defer` o ownership chiara
- deinit mancante su tipi owning
- slice con lifetime ambiguo
- ignorare errori con `catch {}`
