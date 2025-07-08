#![cfg(test)]

use crate::contract::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

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

// === Integration Tests for Complete Trading Workflows ===

#[test]
fn test_complete_trading_flow() {
    let (env, _admin, _fnft_contract_id, xlm_contract_id, trading_client, fnft_client, xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Setup: Create asset and give seller some tokens
    let asset_id = fnft_client.mint(&seller, &1000);

    // Setup: Give buyer some XLM
    mint_xlm_for_user(&env, &xlm_contract_id, &buyer, 10000);

    let token_amount = 100u64;
    let price = 5000u128; // 0.5 XLM

    // Step 1: Seller confirms sale
    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &token_amount,
        &price,
        &DEFAULT_SALE_DURATION,
    );

    // Verify proposal was created
    assert!(trading_client.sale_exists(&seller, &buyer, &asset_id));
    let proposal = trading_client.get_sale_proposal(&seller, &buyer, &asset_id);
    assert_eq!(proposal.seller, seller);
    assert_eq!(proposal.buyer, buyer);
    assert_eq!(proposal.asset_id, asset_id);
    assert_eq!(proposal.token_amount, token_amount);
    assert_eq!(proposal.price, price);
    assert!(proposal.is_active);

    // Verify seller's sales list updated
    let seller_sales = trading_client.get_seller_sales(&seller);
    assert_eq!(seller_sales.len(), 1);
    assert_eq!(seller_sales.get(0).unwrap(), (buyer.clone(), asset_id));

    // Verify buyer's offers list updated
    let buyer_offers = trading_client.get_buyer_offers(&buyer);
    assert_eq!(buyer_offers.len(), 1);
    assert_eq!(buyer_offers.get(0).unwrap(), (seller.clone(), asset_id));

    // Step 2: Buyer finishes transaction
    trading_client.finish_transaction(&buyer, &seller, &asset_id, &token_amount, &price);

    // Verify tokens were transferred
    assert_eq!(fnft_client.balance_of(&seller, &asset_id), 900); // 1000 - 100
    assert_eq!(fnft_client.balance_of(&buyer, &asset_id), 100);

    // Verify XLM payment was transferred
    assert_eq!(xlm_client.balance(&buyer), 5000); // 10000 - 5000
    assert_eq!(xlm_client.balance(&seller), 5000);

    // Verify proposal was removed
    assert!(!trading_client.sale_exists(&seller, &buyer, &asset_id));

    // Verify lists were cleaned up
    let seller_sales_after = trading_client.get_seller_sales(&seller);
    assert_eq!(seller_sales_after.len(), 0);

    let buyer_offers_after = trading_client.get_buyer_offers(&buyer);
    assert_eq!(buyer_offers_after.len(), 0);

    // Verify trade history was recorded
    assert_eq!(trading_client.get_trade_count(), 1);
    let trade_history = trading_client.get_trade_history(&1);
    assert_eq!(trade_history.seller, seller);
    assert_eq!(trade_history.buyer, buyer);
    assert_eq!(trade_history.asset_id, asset_id);
    assert_eq!(trade_history.token_amount, token_amount);
    assert_eq!(trade_history.price, price);

    // Verify asset trades list
    let asset_trades = trading_client.get_asset_trades(&asset_id);
    assert_eq!(asset_trades.len(), 1);
    assert_eq!(asset_trades.get(0).unwrap(), 1u32);
}

#[test]
fn test_multiple_sales_same_seller() {
    let (env, _admin, _fnft_contract_id, _xlm_contract_id, trading_client, fnft_client, xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    // Create assets
    let asset1 = fnft_client.mint(&seller, &1000);
    let asset2 = fnft_client.mint(&seller, &2000);

    // Give buyers XLM
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer1, 10000);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer2, 20000);

    // Create multiple sales
    trading_client.confirm_sale(
        &seller,
        &buyer1,
        &asset1,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );
    trading_client.confirm_sale(
        &seller,
        &buyer2,
        &asset2,
        &200,
        &10000,
        &DEFAULT_SALE_DURATION,
    );

    // Verify seller has 2 active sales
    let seller_sales = trading_client.get_seller_sales(&seller);
    assert_eq!(seller_sales.len(), 2);

    // Complete both transactions
    trading_client.finish_transaction(&buyer1, &seller, &asset1, &100, &5000);
    trading_client.finish_transaction(&buyer2, &seller, &asset2, &200, &10000);

    // Verify all completed
    assert_eq!(trading_client.get_trade_count(), 2);
    let seller_sales_after = trading_client.get_seller_sales(&seller);
    assert_eq!(seller_sales_after.len(), 0); // Should be cleaned up

    // Verify final balances
    assert_eq!(fnft_client.balance_of(&buyer1, &asset1), 100);
    assert_eq!(fnft_client.balance_of(&buyer2, &asset2), 200);
    assert_eq!(xlm_client.balance(&seller), 15000); // 5000 + 10000
}

#[test]
fn test_multiple_buyers_same_asset() {
    let (env, _admin, _fnft_contract_id, _xlm_contract_id, trading_client, fnft_client, xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer1, 10000);
    mint_xlm_for_user(&env, &_xlm_contract_id, &buyer2, 10000);

    // Create sales to different buyers for same asset
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

    // Verify both proposals exist
    assert!(trading_client.sale_exists(&seller, &buyer1, &asset_id));
    assert!(trading_client.sale_exists(&seller, &buyer2, &asset_id));

    // Complete first transaction
    trading_client.finish_transaction(&buyer1, &seller, &asset_id, &100, &5000);

    // Second transaction should still work
    trading_client.finish_transaction(&buyer2, &seller, &asset_id, &200, &8000);

    // Verify final state
    assert_eq!(fnft_client.balance_of(&seller, &asset_id), 700); // 1000 - 100 - 200
    assert_eq!(fnft_client.balance_of(&buyer1, &asset_id), 100);
    assert_eq!(fnft_client.balance_of(&buyer2, &asset_id), 200);
    assert_eq!(xlm_client.balance(&seller), 13000); // 5000 + 8000
}

#[test]
#[should_panic(expected = "Token amount mismatch")]
fn test_buyer_protection_token_amount_mismatch() {
    let (env, _admin, _fnft_contract_id, xlm_contract_id, trading_client, fnft_client, _xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &xlm_contract_id, &buyer, 10000);

    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // This should panic because buyer expects 200 tokens but proposal has 100
    trading_client.finish_transaction(&buyer, &seller, &asset_id, &200, &5000);
}

#[test]
#[should_panic(expected = "Price mismatch")]
fn test_buyer_protection_price_mismatch() {
    let (env, _admin, _fnft_contract_id, xlm_contract_id, trading_client, fnft_client, _xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &xlm_contract_id, &buyer, 10000);

    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // This should panic because buyer expects 1000 price but proposal has 5000
    trading_client.finish_transaction(&buyer, &seller, &asset_id, &100, &1000);
}

#[test]
fn test_buyer_protection_correct_terms_succeed() {
    let (env, _admin, _fnft_contract_id, xlm_contract_id, trading_client, fnft_client, xlm_client) =
        setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let asset_id = fnft_client.mint(&seller, &1000);
    mint_xlm_for_user(&env, &xlm_contract_id, &buyer, 10000);

    trading_client.confirm_sale(
        &seller,
        &buyer,
        &asset_id,
        &100,
        &5000,
        &DEFAULT_SALE_DURATION,
    );

    // This should succeed because terms match exactly
    trading_client.finish_transaction(&buyer, &seller, &asset_id, &100, &5000);

    // Verify transaction completed successfully
    assert_eq!(fnft_client.balance_of(&seller, &asset_id), 900);
    assert_eq!(fnft_client.balance_of(&buyer, &asset_id), 100);
    assert_eq!(xlm_client.balance(&seller), 5000);
}
