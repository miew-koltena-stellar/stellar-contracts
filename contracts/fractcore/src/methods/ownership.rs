use crate::storage::DataKey;
use soroban_sdk::{Address, Env, Vec};

pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0)
}

pub fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerExists(asset_id, owner))
        .unwrap_or(false)
}

pub fn has_assets(env: Env, owner: Address, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerAssetExists(owner, asset_id))
        .unwrap_or(false)
}

pub fn asset_owners(env: Env, asset_id: u64) -> Vec<Address> {
    let page_count: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnerPageCount(asset_id))
        .unwrap_or(0);

    let mut all_owners = Vec::new(&env);

    // Collect owners from all pages
    for page_idx in 0..page_count {
        if let Some(page) = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::AssetOwnersPage(asset_id, page_idx))
        {
            for i in 0..page.len() {
                all_owners.push_back(page.get(i).unwrap());
            }
        }
    }

    all_owners
}

pub fn owner_assets(env: Env, owner: Address) -> Vec<u64> {
    let next_asset_id = env
        .storage()
        .instance()
        .get(&DataKey::NextAssetId)
        .unwrap_or(1);

    let mut owned_assets = Vec::new(&env);

    for asset_id in 1..next_asset_id {
        if env
            .storage()
            .persistent()
            .get(&DataKey::OwnerAssetExists(owner.clone(), asset_id))
            .unwrap_or(false)
        {
            owned_assets.push_back(asset_id);
        }
    }

    owned_assets
}
