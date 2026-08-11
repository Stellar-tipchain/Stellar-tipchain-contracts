# Changelog

## [Unreleased] — ~60% complete

### Done
- Workspace and crate scaffolding (`Cargo.toml`, `contracts/tipjar/Cargo.toml`)
- `DataKey` storage enum (`Token`, `CreatorBalance`, `CreatorTotal`)
- `init(token)` — one-time token address configuration with double-init guard
- Unit tests: `test_init`, `test_init_twice_panics`
- `tip(sender, creator, amount)` — requires supporter auth, rejects non-positive
  amounts, transfers tokens into contract escrow, and credits both the
  withdrawable balance and the all-time total
- `get_total_tips(creator)` — reads the all-time counter, returning `0` for an
  address that has never been tipped
- `("tip", creator)` event carrying `(sender, amount)`
- Persistent TTL extension on both creator entries whenever a tip is received
- Unit tests grew from 2 to 16, covering accumulation, multi-supporter and
  multi-creator isolation, escrow custody, supporter debit, event payload,
  authorisation, both rejection paths, and the uninitialised guard

### Tooling and docs
- CI workflow: WASM build, unit tests, rustfmt and clippy gates
- `Makefile`, `rustfmt.toml`, `.editorconfig`, `.gitattributes`, `.gitignore`
- Committed `Cargo.lock` with `ed25519-dalek` pinned to 2.1.1
- MIT `LICENSE`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue and PR templates
- `docs/` — architecture, storage model, events, testing, dependencies, roadmap

### Pending
- `withdraw(creator)` — release escrowed balance to creator
- `("withdraw", creator)` event
- Withdrawal test coverage, including the zero-before-transfer ordering
- Testnet deploy script (`scripts/deploy.sh`)
