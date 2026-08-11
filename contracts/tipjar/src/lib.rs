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

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env};

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

/// Whether [`TipJar::init`] has already stored a token address.
fn is_initialised(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Token)
}

/// Reads the token address stored by [`TipJar::init`].
///
/// Panics with `"not initialised"` when `init` has never been called, so a
/// mis-ordered deployment fails loudly instead of transferring nothing.
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

/// All-time cumulative tips received by `creator`, defaulting to `0`.
///
/// This counter is never decremented — withdrawing does not erase history.
fn read_total(env: &Env, creator: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::CreatorTotal(creator.clone()))
        .unwrap_or(0)
}

/// Overwrites the all-time total for `creator`.
fn write_total(env: &Env, creator: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::CreatorTotal(creator.clone()), &amount);
}

/// Pushes both persistent creator entries further from archival.
///
/// Must be called only after the entries exist — extending a missing key
/// panics. Both counters are bumped together so they always expire together.
fn extend_creator_ttl(env: &Env, creator: &Address) {
    let storage = env.storage().persistent();
    storage.extend_ttl(
        &DataKey::CreatorBalance(creator.clone()),
        CREATOR_TTL_THRESHOLD,
        CREATOR_TTL_EXTEND_TO,
    );
    storage.extend_ttl(
        &DataKey::CreatorTotal(creator.clone()),
        CREATOR_TTL_THRESHOLD,
        CREATOR_TTL_EXTEND_TO,
    );
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

    /// Tips `creator` with `amount` tokens, held in escrow until withdrawal.
    ///
    /// The tokens move from `sender` into the contract's own account. Both the
    /// creator's withdrawable balance and their all-time total are incremented,
    /// and a `("tip", creator)` event carrying `(sender, amount)` is emitted.
    ///
    /// # Panics
    ///
    /// - `"amount must be positive"` if `amount <= 0`.
    /// - `"not initialised"` if `init` has not been called.
    /// - Whatever the token contract panics with if `sender` lacks the balance.
    ///
    /// # Authorisation
    ///
    /// `sender` must sign the transaction. No authorisation is required from
    /// `creator` — anyone may be tipped without opting in.
    pub fn tip(env: Env, sender: Address, creator: Address, amount: i128) {
        // The supporter's wallet must sign: tokens are about to leave it.
        sender.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token = read_token(&env);

        // Move the tokens into the contract's own account — the escrow.
        token::Client::new(&env, &token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        // Credit the creator's withdrawable balance.
        let balance = read_balance(&env, &creator) + amount;
        write_balance(&env, &creator, balance);

        // Credit the all-time counter, which withdrawals never reset.
        let total = read_total(&env, &creator) + amount;
        write_total(&env, &creator, total);

        // Both entries now exist, so they are safe to bump.
        extend_creator_ttl(&env, &creator);

        env.events()
            .publish((symbol_short!("tip"), creator), (sender, amount));
    }

    /// All-time tips received by `creator`, in the token's smallest unit.
    ///
    /// Read-only and unauthenticated — anyone may query any creator. Returns
    /// `0` for an address that has never been tipped, and is unaffected by
    /// withdrawals, so it always reflects lifetime earnings rather than the
    /// current withdrawable balance.
    pub fn get_total_tips(env: Env, creator: Address) -> i128 {
        read_total(&env, &creator)
    }

    /// Releases the creator's escrowed balance to their wallet.
    ///
    /// Not yet implemented — this is the next milestone. The storage helpers it
    /// needs ([`read_balance`], [`write_balance`]) already exist, so the
    /// remaining work is the ordering and the event:
    ///
    /// 1. `creator.require_auth()`.
    /// 2. Read the balance; panic with `"nothing to withdraw"` when it is `0`.
    /// 3. Zero the balance *before* transferring — reentrancy safety.
    /// 4. `token::Client::transfer(contract → creator, balance)`.
    /// 5. Emit `("withdraw", creator)` with the amount.
    ///
    /// `CreatorTotal` is deliberately left untouched.
    pub fn withdraw(_env: Env, _creator: Address) {
        unimplemented!("withdraw: not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events},
        token::{Client as TokenClient, StellarAssetClient},
        vec, Address, Env, IntoVal,
    };

    /// Starting token balance handed to every supporter in the tests.
    const SUPPORTER_FUNDS: i128 = 10_000;

    /// Registers the contract plus a fresh Stellar asset contract to tip with.
    ///
    /// Returns the environment, the TipJar contract id, the token id, and the
    /// token admin (needed to mint balances for supporters).
    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TipJar, ());
        let token_admin = Address::generate(&env);
        let token_id = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();
        (env, contract_id, token_id, token_admin)
    }

    /// Generates an address holding `amount` of the test token.
    fn funded_supporter(env: &Env, token_id: &Address, admin: &Address, amount: i128) -> Address {
        let supporter = Address::generate(env);
        StellarAssetClient::new(env, token_id)
            .mock_all_auths()
            .mint(&supporter, &amount);
        let _ = admin;
        supporter
    }

    #[test]
    fn test_init() {
        let (env, contract_id, token_id, _admin) = setup();
        TipJarClient::new(&env, &contract_id).init(&token_id);
        // Token address stored — init succeeded without panic
    }

    #[test]
    #[should_panic(expected = "already initialised")]
    fn test_init_twice_panics() {
        let (env, contract_id, token_id, _admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);
        client.init(&token_id); // must panic
    }

    #[test]
    fn test_tip_credits_the_creator_total() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &500);

        assert_eq!(client.get_total_tips(&creator), 500);
    }

    #[test]
    fn test_tip_credits_the_withdrawable_balance() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &750);

        let balance = env.as_contract(&contract_id, || read_balance(&env, &creator));
        assert_eq!(balance, 750);
    }

    #[test]
    fn test_tips_accumulate_across_calls() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &100);
        client.tip(&supporter, &creator, &250);
        client.tip(&supporter, &creator, &50);

        assert_eq!(client.get_total_tips(&creator), 400);
    }

    #[test]
    fn test_tips_from_multiple_supporters_aggregate() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let first = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let second = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&first, &creator, &300);
        client.tip(&second, &creator, &200);

        assert_eq!(client.get_total_tips(&creator), 500);
    }

    #[test]
    fn test_creator_balances_are_isolated() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.tip(&supporter, &alice, &900);

        assert_eq!(client.get_total_tips(&alice), 900);
        assert_eq!(client.get_total_tips(&bob), 0);
    }

    #[test]
    fn test_tipped_funds_land_in_contract_escrow() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &1_200);

        let token = TokenClient::new(&env, &token_id);
        assert_eq!(token.balance(&contract_id), 1_200);
        // The creator is credited in storage, not paid out yet.
        assert_eq!(token.balance(&creator), 0);
    }

    #[test]
    fn test_tip_debits_the_supporter() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &400);

        let token = TokenClient::new(&env, &token_id);
        assert_eq!(token.balance(&supporter), SUPPORTER_FUNDS - 400);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_zero_tip_amount_panics() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &0);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_negative_tip_amount_panics() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &-100);
    }

    #[test]
    fn test_get_total_tips_is_zero_for_unknown_creator() {
        let (env, contract_id, token_id, _admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let stranger = Address::generate(&env);

        assert_eq!(client.get_total_tips(&stranger), 0);
    }

    #[test]
    #[should_panic(expected = "not initialised")]
    fn test_tip_before_init_panics() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        // Deliberately skip init.

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &100);
    }

    #[test]
    fn test_tip_emits_a_tip_event() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &640);

        // The token contract emits its own transfer event, so select ours.
        let mut ours = vec![&env];
        for event in env.events().all().iter() {
            if event.0 == contract_id {
                ours.push_back(event);
            }
        }

        assert_eq!(
            ours,
            vec![
                &env,
                (
                    contract_id.clone(),
                    (symbol_short!("tip"), creator).into_val(&env),
                    (supporter, 640_i128).into_val(&env),
                ),
            ],
        );
    }

    #[test]
    fn test_tip_requires_supporter_authorisation() {
        let (env, contract_id, token_id, admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let supporter = funded_supporter(&env, &token_id, &admin, SUPPORTER_FUNDS);
        let creator = Address::generate(&env);

        client.tip(&supporter, &creator, &250);

        let auths = env.auths();
        assert!(
            auths.iter().any(|(address, _)| address == &supporter),
            "tip must require the supporter to sign",
        );
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_withdraw_is_not_implemented_yet() {
        let (env, contract_id, token_id, _admin) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);

        let creator = Address::generate(&env);

        // Pins the current 60% state: withdraw must fail loudly, never silently
        // succeed while funds stay in escrow.
        client.withdraw(&creator);
    }

    // TODO: test_withdraw — replace test_withdraw_is_not_implemented_yet once
    // withdraw() releases escrowed funds.
}
