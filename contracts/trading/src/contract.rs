use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use crate::storage::{SaleProposal, TradeHistory};
use crate::methods::{initialization, queries, sales, transactions};

#[contract]
pub struct TradingContract;

#[contractimpl]
impl TradingContract {
    /// Initialize the trading contract
    pub fn initialize(env: Env, admin: Address, fnft_contract: Address, xlm_contract: Address) {
        initialization::initialize(env, admin, fnft_contract, xlm_contract);
    }

    /// Step 1: Seller confirms/proposes a sale
    pub fn confirm_sale(
        env: Env,
        seller: Address,
        buyer: Address,
        asset_id: u64,
        token_amount: u64,
        price: u128,
        duration_seconds: u64,
    ) {
        sales::confirm_sale(env, seller, buyer, asset_id, token_amount, price, duration_seconds);
    }

    /// Step 2: Buyer finishes the transaction
    pub fn finish_transaction(env: Env, buyer: Address, seller: Address, asset_id: u64) {
        transactions::finish_transaction(env, buyer, seller, asset_id);
    }

    /// Clean up expired sale proposals
    pub fn cleanup_expired_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
        sales::cleanup_expired_sale(env, seller, buyer, asset_id);
    }

    /// Seller withdraws sale proposal
    pub fn withdraw_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
        sales::withdraw_sale(env, seller, buyer, asset_id);
    }

    /// Emergency function to reset allowances
    pub fn emergency_reset_allowance(env: Env, seller: Address, asset_id: u64) {
        sales::emergency_reset_allowance(env, seller, asset_id);
    }

    /// Get XLM contract address
    pub fn get_xlm_contract_address_public(env: Env) -> Address {
        queries::get_xlm_contract_address_public(env)
    }

    /// Get a specific sale proposal
    pub fn get_sale_proposal(
        env: Env,
        seller: Address,
        buyer: Address,
        asset_id: u64,
    ) -> SaleProposal {
        queries::get_sale_proposal(env, seller, buyer, asset_id)
    }

    /// Check if a sale proposal exists
    pub fn sale_exists(env: Env, seller: Address, buyer: Address, asset_id: u64) -> bool {
        queries::sale_exists(env, seller, buyer, asset_id)
    }

    /// Get all active sales for a seller
    pub fn get_seller_sales(env: Env, seller: Address) -> Vec<(Address, u64)> {
        queries::get_seller_sales(env, seller)
    }

    /// Get all offers for a buyer
    pub fn get_buyer_offers(env: Env, buyer: Address) -> Vec<(Address, u64)> {
        queries::get_buyer_offers(env, buyer)
    }

    /// Get trade history by trade ID
    pub fn get_trade_history(env: Env, trade_id: u32) -> TradeHistory {
        queries::get_trade_history(env, trade_id)
    }

    /// Get total number of completed trades
    pub fn get_trade_count(env: Env) -> u32 {
        queries::get_trade_count(env)
    }

    /// Get all trades for a specific asset
    pub fn get_asset_trades(env: Env, asset_id: u64) -> Vec<u32> {
        queries::get_asset_trades(env, asset_id)
    }

    /// Get the FNFT contract address
    pub fn get_fnft_contract_address(env: Env) -> Address {
        queries::get_fnft_contract_address(env)
    }

    /// Get time until sale proposal expires
    pub fn time_until_expiry(env: Env, seller: Address, buyer: Address, asset_id: u64) -> u64 {
        queries::time_until_expiry(env, seller, buyer, asset_id)
    }

    /// Check current allowance for a seller-asset pair
    pub fn get_current_allowance(env: Env, seller: Address, asset_id: u64) -> u64 {
        queries::get_current_allowance(env, seller, asset_id)
    }
}
