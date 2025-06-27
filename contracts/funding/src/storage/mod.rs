use soroban_sdk::contracttype;

/// Storage keys for funding contract data
#[contracttype]
pub enum DataKey {
    // Core contract data
    Admin,
    FNFTContract, // Address of the FNFT contract
    XLMToken,     // Address of the XLM token contract (SAC)

    // Fund tracking
    AssetFunds(u64),       // asset_id -> total_xlm_available
    TotalDistributed(u64), // asset_id -> total_xlm_distributed

    // Distribution tracking (optional for analytics)
    DistributionCount(u64), // asset_id -> number_of_distributions
}
