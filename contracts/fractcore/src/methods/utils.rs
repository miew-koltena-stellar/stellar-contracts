use crate::storage::DataKey;
use soroban_sdk::{Address, Env, Vec};

static MAX_OWNERS_PER_PAGE: u32 = 50; // Maximum owners per page

/// Next asset ID to be assigned
pub fn next_asset_id(env: Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::NextAssetId)
        .unwrap_or(1)
}

pub fn asset_exists(env: Env, asset_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::AssetSupply(asset_id))
}

/// Add asset to owner's asset list
pub fn add_owner_to_asset(env: &Env, asset_id: u64, owner: Address) {
    // Check if owner already exists - only add if new
    if env
        .storage()
        .persistent()
        .has(&DataKey::AssetOwnerExists(asset_id, owner.clone()))
    {
        return; // Owner already exists, nothing to do
    }

    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerExists(asset_id, owner.clone()), &true);
    env.storage()
        .persistent()
        .set(&DataKey::OwnerAssetExists(owner.clone(), asset_id), &true);

    let current_count: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerCount(asset_id), &(current_count + 1));

    if let Some(hint_page) = env
        .storage()
        .persistent()
        .get::<DataKey, u32>(&DataKey::AssetLastActivePage(asset_id))
    {
        if let Some(mut page) = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::AssetOwnersPage(asset_id, hint_page))
        {
            if page.len() < MAX_OWNERS_PER_PAGE {
                // Space found in hinted page
                page.push_back(owner.clone());
                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnersPage(asset_id, hint_page), &page);

                // Store location for fast removal
                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnerLocation(asset_id, owner), &hint_page);
                return;
            }
        }
    }

    // Fallback: scan for first page with space
    let page_count: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnerPageCount(asset_id))
        .unwrap_or(0);

    for page_idx in 0..page_count {
        if let Some(mut page) = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::AssetOwnersPage(asset_id, page_idx))
        {
            if page.len() < MAX_OWNERS_PER_PAGE {
                // Found space in existing page
                page.push_back(owner.clone());
                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnersPage(asset_id, page_idx), &page);

                env.storage()
                    .persistent()
                    .set(&DataKey::AssetLastActivePage(asset_id), &page_idx);

                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnerLocation(asset_id, owner), &page_idx);
                return;
            }
        }
    }

    // Create new page if all existing pages are full
    let mut new_page = Vec::new(&env);
    new_page.push_back(owner.clone());
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnersPage(asset_id, page_count), &new_page);
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerPageCount(asset_id), &(page_count + 1));

    // SET HINT to new page
    env.storage()
        .persistent()
        .set(&DataKey::AssetLastActivePage(asset_id), &page_count);

    // Store location
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerLocation(asset_id, owner), &page_count);
}

/// Remove owner from asset's paginated lists using location tracking
pub fn remove_owner_from_asset(env: &Env, asset_id: u64, owner: Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::AssetOwnerExists(asset_id, owner.clone()));

    let current_count: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0);
    if current_count > 0 {
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerCount(asset_id), &(current_count - 1));
    }

    if let Some(page_num) = env
        .storage()
        .persistent()
        .get::<DataKey, u32>(&DataKey::AssetOwnerLocation(asset_id, owner.clone()))
    {
        let page: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetOwnersPage(asset_id, page_num))
            .unwrap_or(Vec::new(&env));

        // Remove owner from page
        let mut new_page = Vec::new(&env);
        for i in 0..page.len() {
            let current_owner = page.get(i).unwrap();
            if current_owner != owner {
                new_page.push_back(current_owner);
            }
        }

        if new_page.len() == 0 {
            // Page is now empty - remove it
            env.storage()
                .persistent()
                .remove(&DataKey::AssetOwnersPage(asset_id, page_num));
        } else {
            // Update page with filtered content
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnersPage(asset_id, page_num), &new_page);

            env.storage()
                .persistent()
                .set(&DataKey::AssetLastActivePage(asset_id), &page_num);
        }

        // Remove location tracking
        env.storage()
            .persistent()
            .remove(&DataKey::AssetOwnerLocation(asset_id, owner));
    }
}

/// Auto Cleanup: Remove asset from owner when balance = 0
pub fn remove_asset_from_owner(env: &Env, owner: Address, asset_id: u64) {
    env.storage()
        .persistent()
        .remove(&DataKey::OwnerAssetExists(owner.clone(), asset_id));
}

/// Add asset to owner when they get their first tokens
pub fn add_asset_to_owner(env: &Env, owner: Address, asset_id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::OwnerAssetExists(owner.clone(), asset_id), &true);
}
