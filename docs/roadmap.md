# Roadmap

Percentages track the nine features in the README progress tracker.

## Shipped — 30%

- Workspace and crate scaffolding
- `DataKey` storage enum
- `init(token)` with a double-init guard
- `test_init`, `test_init_twice_panics`

## In progress — target 60%

- `tip(sender, creator, amount)` escrow transfer and dual balance update
- `get_total_tips(creator)`
- `tip` event
- Unit tests covering tipping, accumulation, isolation and rejection paths

## Next — target 100%

- `withdraw(creator)` releasing escrowed funds
- `withdraw` event
- Withdrawal test coverage, including the reentrancy ordering guarantee
- `scripts/deploy.sh` testnet deployment helper

## Explicitly out of scope

- Admin roles, pausing, or fee taking — the contract stays non-custodial.
- Multi-token support in a single deployment; deploy one instance per token.
