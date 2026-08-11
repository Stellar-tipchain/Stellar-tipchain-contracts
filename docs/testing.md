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

## Current suite

| Test | Behaviour pinned |
|---|---|
| `test_init` | `init` stores the token address |
| `test_init_twice_panics` | Second `init` panics with `"already initialised"` |
| `test_tip_credits_the_creator_total` | A tip increments the all-time counter |
| `test_tip_credits_the_withdrawable_balance` | A tip increments the escrow balance |
| `test_tips_accumulate_across_calls` | Repeated tips add up |
| `test_tips_from_multiple_supporters_aggregate` | Different senders credit one creator |
| `test_creator_balances_are_isolated` | One creator's tips never reach another |
| `test_tipped_funds_land_in_contract_escrow` | Tokens sit with the contract, not the creator |
| `test_tip_debits_the_supporter` | The sender pays exactly the tip amount |
| `test_zero_tip_amount_panics` | `amount == 0` is rejected |
| `test_negative_tip_amount_panics` | `amount < 0` is rejected |
| `test_get_total_tips_is_zero_for_unknown_creator` | Untouched addresses read as `0` |
| `test_tip_before_init_panics` | Tipping an unconfigured jar fails loudly |
| `test_tip_emits_a_tip_event` | Event topics and payload match the spec |
| `test_tip_requires_supporter_authorisation` | `require_auth` is recorded for the sender |
| `test_withdraw_is_not_implemented_yet` | `withdraw` fails loudly while unimplemented |

## Coverage targets

Every public function needs at least: one happy path, one rejection path, and
one "never touched before" path where storage is empty.
