# Storage Model

## Keys

```rust
pub enum DataKey {
    Token,                     // instance
    CreatorBalance(Address),   // persistent
    CreatorTotal(Address),     // persistent
}
```

| Key | Tier | Type | Written by | Read by |
|---|---|---|---|---|
| `Token` | Instance | `Address` | `init` | `tip`, `withdraw` |
| `CreatorBalance(a)` | Persistent | `i128` | `tip`, `withdraw` | `withdraw` |
| `CreatorTotal(a)` | Persistent | `i128` | `tip` | `get_total_tips` |

## Tier choice

**Instance** storage shares the lifetime and TTL of the contract instance
itself. It is the right home for `Token`: one small value, read by nearly every
call, never changing after `init`.

**Persistent** storage entries live independently of the instance and survive
archival as long as their TTL is extended. Creator balances must never be lost,
so they belong here. Temporary storage is deliberately unused — nothing in this
contract is safe to drop.

## Defaults

Both persistent counters are read with a default of `0`, so a creator who has
never been tipped needs no initialisation step. This keeps `tip` a single-call
operation for a brand new creator.

## Amount type

`i128` matches the Soroban token interface. Values are in the token's smallest
unit (stroops for XLM); the contract performs no decimal scaling.
