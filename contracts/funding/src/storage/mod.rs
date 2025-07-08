use soroban_sdk::{contracttype, Address};

/// Storage keys for funding contract data
#[contracttype]
pub enum DataKey {
    // Core contract data
    Admin,
    GovernanceContract,
    FNFTContract,

    // SAC Management
    AssetSAC(u64),       // asset_id → sac_contract_address
    SACToAsset(Address), // sac_address → asset_id (reverse lookup)

    // Analytics
    TotalDistributed(u64),  // asset_id → total_xlm_distributed
    DistributionCount(u64), // asset_id → number_of_distributions
}
