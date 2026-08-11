# Architecture

## Components

```
contracts/tipjar/src/lib.rs
├── DataKey            storage key enum
├── TipJar             contract type
└── impl TipJar        init / tip / get_total_tips / withdraw
```

A single deployed contract instance serves every creator. There is no factory,
no per-creator contract, and no registry — a creator "exists" the moment
someone tips them, because the persistent storage entry keyed by their address
is created on first tip.

## Why a single contract

| Option | Cost | Complexity | Chosen |
|---|---|---|---|
| One contract per creator | New deploy + init per creator | High | No |
| One shared contract, address-keyed storage | One deploy total | Low | Yes |

The shared contract keeps deployment cost fixed and makes off-chain indexing
simpler: a consumer watches one contract ID and filters events by the creator
address in the topic.

## Escrow account

The contract's own address (`env.current_contract_address()`) is the escrow
account. Tipped tokens are transferred to it, and withdrawals are transferred
out of it. Because the contract never calls `require_auth` on itself for
outgoing transfers, the only path out of escrow is `withdraw`, which is gated
on the creator's own signature.

## Trust assumptions

- The token contract passed to `init` behaves like a standard SEP-41 token.
- No admin key exists after `init`, so the deployer cannot rug the escrow.
- Storage entries must be kept alive (TTL bumped) or the ledger may archive
  them; the contract extends TTLs on write.
