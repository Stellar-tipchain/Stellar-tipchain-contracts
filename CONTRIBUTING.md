# Contributing to Stellar-tipchain-contracts

## Branching Strategy

- `main` — stable, deployable code only
- `feat/<name>` — new features
- `fix/<name>` — bug fixes
- `chore/<name>` — tooling, deps, docs

## Coding Standards

- Rust edition 2021, `#![no_std]` for contract crates
- Run `cargo fmt` and `cargo clippy -- -D warnings` before committing
- Keep contract functions minimal; move helpers to separate modules if needed

## Test Requirements

Every PR must include unit tests covering:
- The happy path for any new function
- At least one failure/panic case per function that can panic

Run tests with:
```
cargo test -p tipjar
```

## Pull Request Checklist

- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test -p tipjar` passes
- [ ] Contract builds to WASM: `cargo build -p tipjar --target wasm32v1-none --release`
- [ ] PR description explains *what* and *why*
- [ ] New storage keys documented in README if added

## Local checks

Run everything CI runs before opening a pull request:

```bash
make check        # fmt --check, clippy -D warnings, unit tests
make build        # wasm32v1-none release build
```

CI runs the same three jobs: `build`, `test`, and `lint`. A pull request that
fails any of them will not be merged.

## Dependencies

`Cargo.lock` is committed and the build is expected to be reproducible from it.
If you need to change a pinned version, explain why in the pull request and
update `docs/dependencies.md` — one pin (`ed25519-dalek`) is deliberate and
documented there.

## Definition of done

A feature is only "done" when all of the following are true:

1. The function is implemented with its panic paths.
2. Unit tests cover the happy path, every rejection path, and the empty-storage
   path.
3. Rustdoc on the function states its panics and authorisation requirements.
4. The README progress tracker, function reference, and `CHANGELOG.md` are
   updated in the same pull request.
