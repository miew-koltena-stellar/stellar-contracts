use soroban_sdk::{Address, Env, Vec};
use crate::storage::DataKey;

/// NEW FUNCTION: Owner count per asset
/// Replaces expensive queries from Solidity tree structures
pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0)
}

/// NEW FUNCTION: Fast ownership verification
/// Replaces complex iterations from RegistryNestedTree
pub fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerExists(asset_id, owner))
        .unwrap_or(false)
}

/// NEW FUNCTION: Check if owner has any asset
/// Helper functionality for queries
pub fn has_assets(env: Env, owner: Address, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerAssetExists(owner, asset_id))
        .unwrap_or(false)
}

/// REFACTOR: function assetOwners(uint256 tokenId) external view returns (address[] memory)
/// Simplification: Direct list vs complex tree iteration
/// Optimization: Kept in sync automatically vs on-demand calculation
pub fn asset_owners(env: Env, asset_id: u64) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnersList(asset_id))
        .unwrap_or(Vec::new(&env))
}

/// REFACTOR: function addressAssets(address owner) external view returns (uint256[] memory)
/// Simplification similar to the function above
pub fn owner_assets(env: Env, owner: Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerAssetsList(owner))
        .unwrap_or(Vec::new(&env))
}
