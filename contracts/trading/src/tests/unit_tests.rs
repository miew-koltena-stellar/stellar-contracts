#![cfg(test)]

use crate::contract::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

// Import the FNFT contract for testing
mod fnft {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/fractcore.wasm");
}
const DEFAULT_SALE_DURATION: u64 = 604800; // 1 week default

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    TradingContractClient<'static>,
    fnft::Client<'static>,
    token::Client<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy FNFT contract
    let fnft_contract_id = env.register(fnft::WASM, ());
    let fnft_client = fnft::Client::new(&env, &fnft_contract_id);

    // Create XLM token using Stellar Asset Contract
    let xlm_sac = env.register_stellar_asset_contract_v2(Address::generate(&env));
    let xlm_contract_id = xlm_sac.address();

    // Deploy Trading contract
    let trading_contract_id = env.register(TradingContract, ());
    let trading_client = TradingContractClient::new(&env, &trading_contract_id);

    // Initialize contracts
    let admin = Address::generate(&env);
    fnft_client.initialize(&admin);
    trading_client.initialize(&admin, &fnft_contract_id, &xlm_contract_id);

    // Use token::Client for XLM operations
    let xlm_client = token::Client::new(&env, &xlm_contract_id);

    (
        env,
        admin,
        fnft_contract_id,
        xlm_contract_id,
        trading_client,
        fnft_client,
        xlm_client,
    )
}

// Helper function to mint XLM for testing
fn mint_xlm_for_user(env: &Env, xlm_contract_id: &Address, user: &Address, amount: i128) {
    let token_admin = token::StellarAssetClient::new(env, xlm_contract_id);
    token_admin.mint(user, &amount);
}

// === Basic Functionality Tests ===

#[test]
fn test_initialize_trading_contract() {
    let (
        _env,
        _admin,
        fnft_contract_id,
        xlm_contract_id,
        trading_client,
        _fnft_client,
        _xlm_client,
    ) = setup();

    // Verify contract was initialized correctly
    assert_eq!(trading_client.get_fnft_contract_address(), fnft_contract_id);
    assert_eq!(
        trading_client.get_xlm_contract_address_public(),
        xlm_contract_id
    );
    assert_eq!(trading_client.get_trade_count(), 0);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_double_initialization() {
    let (env, _admin, fnft_contract_id, xlm_contract_id, trading_client, _fnft_client, _xlm_client) =
        setup();
    let new_admin = Address::generate(&env);

    // Second initialization should panic
    trading_client.initialize(&new_admin, &fnft_contract_id, &xlm_contract_id);
}

#[test]
fn test_withdraw_sale() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Setup: Create asset and proposal
    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // Verify proposal exists
    assert!(trading_client.sale_exists(&seller, &buyer, &asset_id));

    // Withdraw sale
    trading_client.withdraw_sale(&seller, &buyer, &asset_id);

    // Verify proposal was removed
    assert!(!trading_client.sale_exists(&seller, &buyer, &asset_id));

    // Verify lists were cleaned up
    let seller_sales = trading_client.get_seller_sales(&seller);
    assert_eq!(seller_sales.len(), 0);

    let buyer_offers = trading_client.get_buyer_offers(&buyer);
    assert_eq!(buyer_offers.len(), 0);

    // Verify no trade history was created
    assert_eq!(trading_client.get_trade_count(), 0);
}

// === Error Condition Tests ===

#[test]
#[should_panic(expected = "Token amount must be > 0")]
fn test_confirm_sale_zero_tokens() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &0,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
}

#[test]
#[should_panic(expected = "Price must be > 0")]
fn test_confirm_sale_zero_price() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &0, &DEFAULT_SALE_DURATION);
}

#[test]
#[should_panic(expected = "Cannot trade with yourself")]
fn test_confirm_sale_self_trade() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(
        &seller,
        &seller,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
}

#[test]
#[should_panic(expected = "Asset does not exist")]
fn test_confirm_sale_nonexistent_asset() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        _fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    trading_client.confirm_sale(&seller, &buyer, &999, &100, &5000, &DEFAULT_SALE_DURATION);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_confirm_sale_insufficient_balance() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &50); // Only 50 tokens
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    ); // Trying to sell 100
}

#[test]
#[should_panic(expected = "Sale proposal already exists - withdraw first")]
fn test_confirm_sale_duplicate_proposal() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);

    // First proposal
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // Second proposal (should fail)
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &200,
        &10000,
        &DEFAULT_SALE_DURATION,
    );
}

#[test]
#[should_panic(expected = "Sale proposal not found")]
fn test_finish_transaction_no_proposal() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        _fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    trading_client.finish_transaction(&buyer, &seller, &999);
}

#[test]
#[should_panic(expected = "Buyer has insufficient XLM funds")]
fn test_finish_transaction_insufficient_xlm() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);

    // Buyer has no XLM
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
    trading_client.finish_transaction(&buyer, &seller, &asset_id);
}

#[test]
#[should_panic(expected = "Seller has insufficient token balance")]
fn test_finish_transaction_seller_insufficient_tokens() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let other = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer, 10000);

    // Create proposal
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // Seller transfers tokens away
    fnft_client.transfer(&seller, &other, &asset_id, &950); // Now seller only has 50 tokens

    // Transaction should fail
    trading_client.finish_transaction(&buyer, &seller, &asset_id);
}

#[test]
#[should_panic(expected = "Sale proposal not found")]
fn test_withdraw_sale_unauthorized() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // Unauthorized user tries to withdraw with their own address as seller
    // This should fail because no such proposal exists
    trading_client.withdraw_sale(&unauthorized, &buyer, &asset_id);
}

#[test]
fn test_exact_balance_transfer() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Seller has exactly the amount being sold
    let asset_id = fnft_client.mint(&seller, &100);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer, 5000);

    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
    trading_client.finish_transaction(&buyer, &seller, &asset_id);

    // Verify seller has 0 tokens left
    assert_eq!(fnft_client.balance_of(&seller, &asset_id), 0);
    assert_eq!(fnft_client.balance_of(&buyer, &asset_id), 100);
}

#[test]
fn test_view_functions() {
    let (env, _admin, fnft_contract_id, xlm_contract_id, trading_client, _fnft_client, _xlm_client) =
        setup();

    // Test view functions with empty state
    assert_eq!(trading_client.get_trade_count(), 0);
    assert_eq!(trading_client.get_fnft_contract_address(), fnft_contract_id);
    assert_eq!(
        trading_client.get_xlm_contract_address_public(),
        xlm_contract_id
    );

    // Test with non-existent data
    let dummy_user = Address::generate(&env);
    let seller_sales = trading_client.get_seller_sales(&dummy_user);
    assert_eq!(seller_sales.len(), 0);

    let buyer_offers = trading_client.get_buyer_offers(&dummy_user);
    assert_eq!(buyer_offers.len(), 0);

    let asset_trades = trading_client.get_asset_trades(&999);
    assert_eq!(asset_trades.len(), 0);
}

#[test]
#[should_panic(expected = "Trade not found")]
fn test_get_nonexistent_trade_history() {
    let (
        _env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        _fnft_client,
        _xlm_client,
    ) = setup();

    trading_client.get_trade_history(&999);
}

// === Expiration Tests ===

#[test]
#[should_panic(expected = "Duration must be between 1 hour and 1 week")]
fn test_confirm_sale_invalid_duration_too_short() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &1800);
    // 30 minutes
}

#[test]
#[should_panic(expected = "Duration must be between 1 hour and 1 week")]
fn test_confirm_sale_invalid_duration_too_long() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &1209600);
    // 2 weeks
}

#[test]
#[should_panic(expected = "Sale proposal has expired")]
fn test_finish_transaction_expired() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer, 10000);

    // Create sale with 1 hour duration
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &3600);

    // Fast forward time past expiration
    let current_ledger = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 3601,
        protocol_version: current_ledger.protocol_version,
        sequence_number: current_ledger.sequence_number,
        network_id: current_ledger.network_id,
        base_reserve: current_ledger.base_reserve,
        min_temp_entry_ttl: current_ledger.min_temp_entry_ttl,
        min_persistent_entry_ttl: current_ledger.min_persistent_entry_ttl,
        max_entry_ttl: current_ledger.max_entry_ttl,
    });

    // Should fail
    trading_client.finish_transaction(&buyer, &seller, &asset_id);
}

#[test]
fn test_cleanup_expired_sale() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);

    // Create sale with 1 hour duration
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &3600);

    // Verify sale exists
    assert!(trading_client.sale_exists(&seller, &buyer, &asset_id));

    // Fast forward time past expiration
    let current_ledger = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 3601,
        protocol_version: current_ledger.protocol_version,
        sequence_number: current_ledger.sequence_number,
        network_id: current_ledger.network_id,
        base_reserve: current_ledger.base_reserve,
        min_temp_entry_ttl: current_ledger.min_temp_entry_ttl,
        min_persistent_entry_ttl: current_ledger.min_persistent_entry_ttl,
        max_entry_ttl: current_ledger.max_entry_ttl,
    });

    // Anyone can clean up expired sale
    trading_client.cleanup_expired_sale(&seller, &buyer, &asset_id);

    // Verify sale was cleaned up
    assert!(!trading_client.sale_exists(&seller, &buyer, &asset_id));
}

#[test]
#[should_panic(expected = "Sale has not expired yet")]
fn test_cleanup_non_expired_sale() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &3600);

    // Try to cleanup before expiration (should fail)
    trading_client.cleanup_expired_sale(&seller, &buyer, &asset_id);
}

#[test]
fn test_time_until_expiry() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    let start_time = env.ledger().timestamp();

    trading_client.confirm_sale(&seller, &buyer, &asset_id, &100, &5000, &3600);

    // Should have close to 3600 seconds left
    let time_left = trading_client.time_until_expiry(&seller, &buyer, &asset_id);
    assert!(time_left <= 3600 && time_left > 3590);

    // Fast forward time by 1800 seconds
    let current_ledger = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: start_time + 1800,
        protocol_version: current_ledger.protocol_version,
        sequence_number: current_ledger.sequence_number,
        network_id: current_ledger.network_id,
        base_reserve: current_ledger.base_reserve,
        min_temp_entry_ttl: current_ledger.min_temp_entry_ttl,
        min_persistent_entry_ttl: current_ledger.min_persistent_entry_ttl,
        max_entry_ttl: current_ledger.max_entry_ttl,
    });

    // Should have about 1800 seconds left
    let time_left = trading_client.time_until_expiry(&seller, &buyer, &asset_id);
    assert!(time_left <= 1800 && time_left > 1790);

    // Fast forward past expiration
    env.ledger().set(LedgerInfo {
        timestamp: start_time + 3601,
        protocol_version: current_ledger.protocol_version,
        sequence_number: current_ledger.sequence_number,
        network_id: current_ledger.network_id,
        base_reserve: current_ledger.base_reserve,
        min_temp_entry_ttl: current_ledger.min_temp_entry_ttl,
        min_persistent_entry_ttl: current_ledger.min_persistent_entry_ttl,
        max_entry_ttl: current_ledger.max_entry_ttl,
    });

    // Should return 0
    let time_left = trading_client.time_until_expiry(&seller, &buyer, &asset_id);
    assert_eq!(time_left, 0);
}

// === Allowance Security Tests ===

#[test]
fn test_emergency_reset_allowance() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);

    // Create multiple sales (allowance should accumulate)
    trading_client.confirm_sale(
        &seller,
        &buyer1,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
    trading_client.confirm_sale(
        &seller,
        &buyer2,
        &asset_id,
        &200,
        &8000,
        &DEFAULT_SALE_DURATION,
    );

    // Check accumulated allowance
    let allowance = trading_client.get_current_allowance(&seller, &asset_id);
    assert_eq!(allowance, 300); // 100 + 200

    // Emergency reset
    trading_client.emergency_reset_allowance(&seller, &asset_id);

    // Verify allowance is reset to 0
    let final_allowance = trading_client.get_current_allowance(&seller, &asset_id);
    assert_eq!(final_allowance, 0);
}

#[test]
fn test_allowance_security_scenario() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _xlm_contract_id,
        trading_client,
        fnft_client,
        _xlm_client,
    ) = setup();
    let seller = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);

    // Scenario: Seller creates multiple sales
    trading_client.confirm_sale(&seller, &buyer1, &asset_id, &100, &5000, &3600); // 1 hour
    trading_client.confirm_sale(&seller, &buyer2, &asset_id, &200, &8000, &3600); // 1 hour

    // Check allowance accumulation
    assert_eq!(
        trading_client.get_current_allowance(&seller, &asset_id),
        300
    );

    // Time passes, sales expire
    let current_ledger = env.ledger().get();
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + 3601,
        protocol_version: current_ledger.protocol_version,
        sequence_number: current_ledger.sequence_number,
        network_id: current_ledger.network_id,
        base_reserve: current_ledger.base_reserve,
        min_temp_entry_ttl: current_ledger.min_temp_entry_ttl,
        min_persistent_entry_ttl: current_ledger.min_persistent_entry_ttl,
        max_entry_ttl: current_ledger.max_entry_ttl,
    });

    // Anyone can clean up expired proposals
    trading_client.cleanup_expired_sale(&seller, &buyer1, &asset_id);
    trading_client.cleanup_expired_sale(&seller, &buyer2, &asset_id);

    // But allowance remains! (This is the security issue we're documenting)
    assert_eq!(
        trading_client.get_current_allowance(&seller, &asset_id),
        300
    );

    // Seller must manually reset for security
    trading_client.emergency_reset_allowance(&seller, &asset_id);

    // Now it's secure
    assert_eq!(trading_client.get_current_allowance(&seller, &asset_id), 0);
}
