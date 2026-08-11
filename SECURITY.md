# Security Policy

## Reporting a vulnerability

Do not open a public issue for a vulnerability that affects funds held in
escrow. Report it privately to the maintainer listed in `Cargo.toml`
(`repository` field → GitHub Security Advisories) and allow time for a fix
before disclosure.

## Threat model

The contract holds tipped tokens in its own account until a creator withdraws.
The properties that must hold:

1. **Only a creator can withdraw their own balance.** `withdraw` calls
   `creator.require_auth()`, and the transfer target is always the same address
   whose balance is read.
2. **No admin, no escape hatch.** After `init` there is no privileged role. The
   token address is immutable and no function can move another creator's funds.
3. **Balances cannot go negative.** `tip` rejects non-positive amounts;
   `withdraw` rejects an empty balance.
4. **Reentrancy safety.** `withdraw` zeroes the stored balance *before* calling
   `token.transfer`, so a reentrant call sees a zero balance.
5. **Accounting integrity.** The sum of all `CreatorBalance` entries never
   exceeds the contract's token balance.

## Out of scope

- Behaviour of a malicious or non-standard token contract passed to `init`.
- Ledger entry expiry caused by an unfunded/unarchived contract instance.
