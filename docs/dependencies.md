# Dependency Notes

## Why `Cargo.lock` is committed

The contract compiles to a WASM binary whose hash is what actually gets
deployed. A floating dependency graph would change that hash between machines,
so the lock file is committed and CI builds from it.

## Pinned transitive dependency: `ed25519-dalek`

`soroban-env-host` declares `ed25519-dalek = ">=2.0.0"`, an open-ended
requirement that resolves to `3.0.0`. Version 3 moved `CryptoRng` to the
`rand_core` 0.9 semantics, so `SigningKey::generate(chacha)` inside the host's
test utilities no longer satisfies its trait bound and the crate fails to
compile:

```
error[E0277]: the trait bound `ChaCha20Rng: ed25519_dalek::rand_core::CryptoRng`
              is not satisfied
```

The lock file therefore pins `ed25519-dalek` to `2.1.1`:

```bash
cargo update -p ed25519-dalek --precise 2.1.1
```

This affects the test host only — it is not linked into the deployed WASM. Drop
the pin once `soroban-env-host` tightens its requirement to `^2`.
