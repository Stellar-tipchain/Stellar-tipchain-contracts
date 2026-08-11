# Events

Soroban events carry a **topic** tuple (indexed, used for filtering) and a
**data** payload.

## `tip` — implemented

| Field | Value |
|---|---|
| Topics | `(Symbol("tip"), creator: Address)` |
| Data | `(sender: Address, amount: i128)` |

Emitted at the end of a successful `tip` call, after the token transfer and
both storage writes have succeeded. If any earlier step panics, the whole
transaction reverts and no event is published.

## `withdraw` — not yet emitted

| Field | Value |
|---|---|
| Topics | `(Symbol("withdraw"), creator: Address)` |
| Data | `amount: i128` |

Planned: emitted after the escrowed balance has been zeroed and transferred
out. `withdraw` is still unimplemented, so no such event exists on chain yet.

## Consuming events

Off-chain consumers filter on the contract ID plus the first topic symbol, and
optionally on the creator address in the second topic:

```bash
stellar events \
  --start-ledger <LEDGER> \
  --id <CONTRACT_ID> \
  --topic '["AAAADwAAAAN0aXA="]' \
  --network testnet
```

Typical consumers: a notification bot that pings a creator when tipped, an
analytics job that builds a leaderboard, or an explorer that renders a tip
feed.
