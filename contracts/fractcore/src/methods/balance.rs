use soroban_sdk::{Address, Env, Vec};
use crate::storage::DataKey;

/// REFACTOR: function balanceOf(address account, uint256 id) external view returns (uint256)
/// Direct implementation - without the overhead of Solidity trees
pub fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64 {
    // Direct storage access - much simpler than RegistryNestedTree trees
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner, asset_id))
        .unwrap_or(0) // Return 0 if doesn't exist (like in Solidity)
}

/// REFACTOR: function balanceOfBatch() from ERC1155
/// Implementation maintained for compatibility
pub fn balance_of_batch(env: Env, owners: Vec<Address>, asset_ids: Vec<u64>) -> Vec<u64> {
    // Validation similar to Solidity
    if owners.len() != asset_ids.len() {
        panic!("Owners and asset_ids length mismatch");
    }

    let mut balances = Vec::new(&env);
    // Iterate and get individual balances
    for i in 0..owners.len() {
        let owner = owners.get(i).unwrap();
        let asset_id = asset_ids.get(i).unwrap();
        let balance = balance_of(env.clone(), owner, asset_id);
        balances.push_back(balance);
    }

    balances
}

/// REFACTOR: function assetSupply(uint256 assetId) external view returns (uint256)
/// Direct implementation vs complex calculations from registries
pub fn asset_supply(env: Env, asset_id: u64) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetSupply(asset_id))
        .unwrap_or(0)
}

/// NEW FUNCTION: Owner count per asset
/// Replaces expensive queries from Solidity tree structures
pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0)
}
