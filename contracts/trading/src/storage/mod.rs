use soroban_sdk::{contracttype, Address};

// Data structures for trading
#[contracttype]
#[derive(Clone)]
pub struct SaleProposal {
    pub seller: Address,
    pub buyer: Address,
    pub asset_id: u64,
    pub token_amount: u64,
    pub price: u128,
    pub is_active: bool,
    pub timestamp: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TradeHistory {
    pub seller: Address,
    pub buyer: Address,
    pub asset_id: u64,
    pub token_amount: u64,
    pub price: u128,
    pub timestamp: u64,
}

// Storage keys for trading contract
#[contracttype]
pub enum DataKey {
    // Core contract data
    Admin,
    FNFTContract,
    XLMContract, // Address of the XLM contract for payments

    // Active sale proposals: (seller, buyer, asset_id) -> SaleProposal
    SaleProposal(Address, Address, u64),

    // Trade history counter and records
    TradeCounter,
    TradeHistory(u32), // trade_id -> TradeHistory

    // User's active sales (for querying)
    SellerSales(Address), // seller -> Vec<(Address, u64)> (buyer, asset_id pairs)
    BuyerOffers(Address), // buyer -> Vec<(Address, u64)> (seller, asset_id pairs)

    // Asset trade activity
    AssetTrades(u64), // asset_id -> Vec<u32> (trade_ids)
}

// Constants
pub const MIN_SALE_DURATION: u64 = 3600; // 1 hour
pub const MAX_SALE_DURATION: u64 = 604800; // 1 week
