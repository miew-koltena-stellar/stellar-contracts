use soroban_sdk::{Address, Env, Vec};
use crate::storage::DataKey;

/// NEW FUNCTION: Next asset ID to be assigned
/// Helper functionality for front-ends
pub fn next_asset_id(env: Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextAssetId)
        .unwrap_or(1)
}

/// NEW FUNCTION: Asset existence verification
/// Replaces complex verifications from Solidity
pub fn asset_exists(env: Env, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::AssetSupply(asset_id))
}

/// Add owner to asset's owner list
/// Keeps list updated for fast queries
pub fn add_owner_to_asset(env: &Env, asset_id: u64, owner: Address) {
    let mut owners: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnersList(asset_id))
        .unwrap_or(Vec::new(env));

    // Check if owner is already in list (avoid duplicates)
    let mut found = false;
    for i in 0..owners.len() {
        if owners.get(i).unwrap() == owner {
            found = true;
            break;
        }
    }

    // Add only if doesn't exist
    if !found {
        owners.push_back(owner.clone());
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnersList(asset_id), &owners);
    }
}

/// Remove owner from list when balance = 0
/// Automatic maintenance vs manual cleanup from Solidity
pub fn remove_owner_from_asset(env: &Env, asset_id: u64, owner: Address) {
    let owners: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnersList(asset_id))
        .unwrap_or(Vec::new(env));

    // Filter out owner to remove
    let mut new_owners = Vec::new(env);
    for i in 0..owners.len() {
        let current_owner = owners.get(i).unwrap();
        if current_owner != owner {
            new_owners.push_back(current_owner);
        }
    }

    // Update list
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnersList(asset_id), &new_owners);
}

/// Add asset to owner's asset list
/// Maintains list for fast queries- similar to bidirectional mapping in Solidity
pub fn add_asset_to_owner(env: &Env, owner: Address, asset_id: u64) {
    let mut assets: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerAssetsList(owner.clone()))
        .unwrap_or(Vec::new(env));

    // verify duplicates
    let mut found = false;
    for i in 0..assets.len() {
        if assets.get(i).unwrap() == asset_id {
            found = true;
            break;
        }
    }

    // add new
    if !found {
        assets.push_back(asset_id);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerAssetsList(owner), &assets);
    }
}

/// Auto Cleanup: Remove asset from owner list when balance = 0
pub fn remove_asset_from_owner(env: &Env, owner: Address, asset_id: u64) {
    let assets: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerAssetsList(owner.clone()))
        .unwrap_or(Vec::new(env));

    // filter and remove
    let mut new_assets = Vec::new(env);
    for i in 0..assets.len() {
        let current_asset = assets.get(i).unwrap();
        if current_asset != asset_id {
            new_assets.push_back(current_asset);
        }
    }

    // update list
    env.storage()
        .persistent()
        .set(&DataKey::OwnerAssetsList(owner), &new_assets);
}
