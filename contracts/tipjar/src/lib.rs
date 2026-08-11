#![no_std]
//! # TipJar
//!
//! A non-custodial tipping escrow for Stellar creators.
//!
//! Supporters call [`TipJar::tip`] to move tokens into the contract's own
//! account. The contract credits two independent per-creator counters: a
//! withdrawable balance and an all-time total. Creators call
//! [`TipJar::withdraw`] to release their balance. There is no admin role — once
//! [`TipJar::init`] has stored the token address, the contract is autonomous.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// Ledgers closed in roughly one day at the ~5 second Stellar ledger cadence.
const LEDGERS_PER_DAY: u32 = 17_280;

/// Persistent creator entries are bumped when they fall within 30 days of expiry.
const CREATOR_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;

/// Bumped entries are extended to live for a further 60 days.
const CREATOR_TTL_EXTEND_TO: u32 = LEDGERS_PER_DAY * 60;

#[contracttype]
pub enum DataKey {
    Token,
    CreatorBalance(Address),
    CreatorTotal(Address),
}

/// Reads the token address stored by [`TipJar::init`].
///
/// Panics with `"not initialised"` when `init` has never been called, so a
/// mis-ordered deployment fails loudly instead of transferring nothing.
/// Whether [`TipJar::init`] has already stored a token address.
fn is_initialised(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Token)
}

fn read_token(env: &Env) -> Address {
    match env.storage().instance().get(&DataKey::Token) {
        Some(token) => token,
        None => panic!("not initialised"),
    }
}

/// Current withdrawable balance for `creator`, defaulting to `0`.
///
/// A creator needs no registration step: an untouched address simply reads as
/// zero until the first tip creates the entry.
fn read_balance(env: &Env, creator: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::CreatorBalance(creator.clone()))
        .unwrap_or(0)
}

/// Overwrites the withdrawable balance for `creator`.
fn write_balance(env: &Env, creator: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::CreatorBalance(creator.clone()), &amount);
}

#[contract]
pub struct TipJar;

#[contractimpl]
impl TipJar {
    /// One-time initialisation: store the token contract address.
    pub fn init(env: Env, token: Address) {
        if is_initialised(&env) {
            panic!("already initialised");
        }
        env.storage().instance().set(&DataKey::Token, &token);
    }

    /// TODO: Transfer `amount` tokens from `sender` into escrow for `creator`.
    /// - Require sender auth
    /// - Validate amount > 0
    /// - Transfer tokens sender → contract
    /// - Update CreatorBalance and CreatorTotal
    /// - Emit ("tip", creator) event
    pub fn tip(_env: Env, _sender: Address, _creator: Address, _amount: i128) {
        unimplemented!("tip: not yet implemented")
    }

    /// TODO: Return cumulative total tips received by `creator`.
    pub fn get_total_tips(_env: Env, _creator: Address) -> i128 {
        unimplemented!("get_total_tips: not yet implemented")
    }

    /// TODO: Transfer creator's escrowed balance to their wallet.
    /// - Require creator auth
    /// - Validate balance > 0
    /// - Transfer tokens contract → creator
    /// - Reset CreatorBalance to 0
    /// - Emit ("withdraw", creator) event
    pub fn withdraw(_env: Env, _creator: Address) {
        unimplemented!("withdraw: not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _,
        token::StellarAssetClient,
        Address, Env,
    };

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TipJar, ());
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
        (env, contract_id, token_id)
    }

    #[test]
    fn test_init() {
        let (env, contract_id, token_id) = setup();
        TipJarClient::new(&env, &contract_id).init(&token_id);
        // Token address stored — init succeeded without panic
    }

    #[test]
    #[should_panic(expected = "already initialised")]
    fn test_init_twice_panics() {
        let (env, contract_id, token_id) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);
        client.init(&token_id); // must panic
    }

    // TODO: test_tip_and_totals — blocked on tip() implementation
    // TODO: test_withdraw       — blocked on withdraw() implementation
    // TODO: test_invalid_tip_amount — blocked on tip() implementation
}
