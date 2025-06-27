use soroban_sdk::{Address, Env, Vec};
use crate::storage::{DataKey, SaleProposal, TradeHistory};
use crate::interfaces::FNFTClient;
use crate::methods::utils;

/// Get the XLM contract address (public view function)
pub fn get_xlm_contract_address_public(env: Env) -> Address {
    utils::get_xlm_contract_address(env)
}

/// Get a specific sale proposal
pub fn get_sale_proposal(
    env: Env,
    seller: Address,
    buyer: Address,
    asset_id: u64,
) -> SaleProposal {
    env.storage()
        .persistent()
        .get(&DataKey::SaleProposal(seller, buyer, asset_id))
        .unwrap_or_else(|| panic!("Sale proposal not found"))
}

/// Check if a sale proposal exists
pub fn sale_exists(env: Env, seller: Address, buyer: Address, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::SaleProposal(seller, buyer, asset_id))
}

/// Get all active sales for a seller
pub fn get_seller_sales(env: Env, seller: Address) -> Vec<(Address, u64)> {
    env.storage()
        .persistent()
        .get(&DataKey::SellerSales(seller))
        .unwrap_or(Vec::new(&env))
}

/// Get all offers for a buyer
pub fn get_buyer_offers(env: Env, buyer: Address) -> Vec<(Address, u64)> {
    env.storage()
        .persistent()
        .get(&DataKey::BuyerOffers(buyer))
        .unwrap_or(Vec::new(&env))
}

/// Get trade history by trade ID
pub fn get_trade_history(env: Env, trade_id: u32) -> TradeHistory {
    env.storage()
        .persistent()
        .get(&DataKey::TradeHistory(trade_id))
        .unwrap_or_else(|| panic!("Trade not found"))
}

/// Get total number of completed trades
pub fn get_trade_count(env: Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::TradeCounter)
        .unwrap_or(0)
}

/// Get all trades for a specific asset
pub fn get_asset_trades(env: Env, asset_id: u64) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::AssetTrades(asset_id))
        .unwrap_or(Vec::new(&env))
}

/// Get the FNFT contract address
pub fn get_fnft_contract_address(env: Env) -> Address {
    utils::get_fnft_contract(&env)
}

/// Get time until sale proposal expires
pub fn time_until_expiry(env: Env, seller: Address, buyer: Address, asset_id: u64) -> u64 {
    let proposal = get_sale_proposal(env.clone(), seller, buyer, asset_id);
    let current_time = env.ledger().timestamp();

    if current_time >= proposal.expires_at {
        0
    } else {
        proposal.expires_at - current_time
    }
}

/// Check current allowance for a seller-asset pair
pub fn get_current_allowance(env: Env, seller: Address, asset_id: u64) -> u64 {
    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    let trading_contract_id = env.current_contract_address();

    fnft_client.allowance(&seller, &trading_contract_id, &asset_id)
}
