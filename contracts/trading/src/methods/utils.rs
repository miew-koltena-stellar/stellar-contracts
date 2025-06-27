use soroban_sdk::{Address, Env, Vec};
use crate::storage::{DataKey, SaleProposal, TradeHistory};

/// Get the FNFT contract address
pub fn get_fnft_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FNFTContract)
        .unwrap()
}

/// Get the XLM contract address
pub fn get_xlm_contract_address(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::XLMContract)
        .unwrap_or_else(|| panic!("XLM contract address not configured"))
}

/// Get a sale proposal by seller, buyer, and asset ID
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

/// Record trade history and return new trade ID
pub fn record_trade_history(env: &Env, proposal: &SaleProposal) -> u32 {
    let trade_id: u32 = env
        .storage()
        .instance()
        .get(&DataKey::TradeCounter)
        .unwrap_or(0);

    let new_trade_id = trade_id + 1;

    let history = TradeHistory {
        seller: proposal.seller.clone(),
        buyer: proposal.buyer.clone(),
        asset_id: proposal.asset_id,
        token_amount: proposal.token_amount,
        price: proposal.price,
        timestamp: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::TradeHistory(new_trade_id), &history);

    env.storage()
        .instance()
        .set(&DataKey::TradeCounter, &new_trade_id);

    new_trade_id
}

/// Add a sale to seller's active sales list
pub fn add_to_seller_sales(env: &Env, seller: Address, buyer: Address, asset_id: u64) {
    let mut sales: Vec<(Address, u64)> = env
        .storage()
        .persistent()
        .get(&DataKey::SellerSales(seller.clone()))
        .unwrap_or(Vec::new(env));

    sales.push_back((buyer, asset_id));
    env.storage()
        .persistent()
        .set(&DataKey::SellerSales(seller), &sales);
}

/// Remove a sale from seller's active sales list
pub fn remove_from_seller_sales(env: &Env, seller: Address, buyer: Address, asset_id: u64) {
    let sales: Vec<(Address, u64)> = env
        .storage()
        .persistent()
        .get(&DataKey::SellerSales(seller.clone()))
        .unwrap_or(Vec::new(env));

    let mut new_sales = Vec::new(env);
    for i in 0..sales.len() {
        let (current_buyer, current_asset_id) = sales.get(i).unwrap();
        if !(current_buyer == buyer && current_asset_id == asset_id) {
            new_sales.push_back((current_buyer, current_asset_id));
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::SellerSales(seller), &new_sales);
}

/// Add an offer to buyer's offers list
pub fn add_to_buyer_offers(env: &Env, buyer: Address, seller: Address, asset_id: u64) {
    let mut offers: Vec<(Address, u64)> = env
        .storage()
        .persistent()
        .get(&DataKey::BuyerOffers(buyer.clone()))
        .unwrap_or(Vec::new(env));

    offers.push_back((seller, asset_id));
    env.storage()
        .persistent()
        .set(&DataKey::BuyerOffers(buyer), &offers);
}

/// Remove an offer from buyer's offers list
pub fn remove_from_buyer_offers(env: &Env, buyer: Address, seller: Address, asset_id: u64) {
    let offers: Vec<(Address, u64)> = env
        .storage()
        .persistent()
        .get(&DataKey::BuyerOffers(buyer.clone()))
        .unwrap_or(Vec::new(env));

    let mut new_offers = Vec::new(env);
    for i in 0..offers.len() {
        let (current_seller, current_asset_id) = offers.get(i).unwrap();
        if !(current_seller == seller && current_asset_id == asset_id) {
            new_offers.push_back((current_seller, current_asset_id));
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::BuyerOffers(buyer), &new_offers);
}

/// Add a trade ID to asset's trade history
pub fn add_to_asset_trades(env: &Env, asset_id: u64, trade_id: u32) {
    let mut trades: Vec<u32> = env
        .storage()
        .persistent()
        .get(&DataKey::AssetTrades(asset_id))
        .unwrap_or(Vec::new(env));

    trades.push_back(trade_id);
    env.storage()
        .persistent()
        .set(&DataKey::AssetTrades(asset_id), &trades);
}
