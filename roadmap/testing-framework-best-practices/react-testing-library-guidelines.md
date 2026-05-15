# React Testing Library Guidelines

## Principi

React Testing Library spinge a testare componenti come li usa l’utente. Non testare state interno, metodi privati o dettagli del framework.

## Query

Preferire query accessibili in questo ordine:

1. `getByRole`
2. `getByLabelText`
3. `getByPlaceholderText`
4. `getByText`
5. `getByDisplayValue`
6. `getByTestId` come ultima scelta

```tsx
expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
```

## User interactions

Usare `userEvent`, non `fireEvent`, salvo casi specifici.

```tsx
await user.click(screen.getByRole("button", { name: "Submit" }));
```

## Async UI

Usare `findBy...` o `waitFor`.

```tsx
expect(await screen.findByText("Order created")).toBeInTheDocument();
```

Non usare `sleep`.

## Provider setup

Creare helper `renderWithProviders`.

```tsx
renderWithProviders(<OrderPage />, {
  user: activeUser(),
  queryClient,
});
```

Il setup deve essere esplicito e leggibile.

## Mock API

Preferire MSW per mock HTTP a livello di rete.

Questo evita di mockare hook e API client interni.

## Testare stati importanti

- loading
- error
- empty
- success
- form validation
- disabled state
- permission state

## Anti-pattern

- `container.querySelector`
- test su classi CSS
- test su state interno
- snapshot grandi
- mockare custom hook invece di simulare comportamento
- `data-testid` ovunque
