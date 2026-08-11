# Testing

All tests run inside Soroban's in-process test environment (`Env::default()`).
No network, no CLI, no deployed contract required.

```bash
cargo test -p tipjar
```

## Test harness

`setup()` builds the world every test needs:

1. `Env::default()` with `mock_all_auths()` so `require_auth` calls succeed.
2. The `TipJar` contract registered, yielding its contract ID.
3. A Stellar asset contract registered as the token, with a generated admin.

## Conventions

- One behaviour per test; the test name states the behaviour, not the function.
- Panics are asserted with `#[should_panic(expected = "...")]` against the exact
  message the contract panics with, so a reworded panic fails the test.
- Token balances are asserted through `token::Client`, not through contract
  storage, whenever the assertion is about funds actually moving.
- Storage-only assertions are wrapped in `env.as_contract(&id, || ...)`.

## Coverage targets

Every public function needs at least: one happy path, one rejection path, and
one "never touched before" path where storage is empty.
