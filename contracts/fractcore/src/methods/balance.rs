use crate::storage::DataKey;
use soroban_sdk::{Address, Env, Vec};

pub fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner, asset_id))
        .unwrap_or(0) // Return 0 if doesn't exist
}

pub fn balance_of_batch(env: Env, owners: Vec<Address>, asset_ids: Vec<u64>) -> Vec<u64> {
    if owners.len() != asset_ids.len() {
        panic!("Owners and asset_ids length mismatch");
    }

    let mut balances = Vec::new(&env);
    for i in 0..owners.len() {
        let owner = owners.get(i).unwrap();
        let asset_id = asset_ids.get(i).unwrap();
        let balance = balance_of(env.clone(), owner, asset_id);
        balances.push_back(balance);
    }

    balances
}

pub fn asset_supply(env: Env, asset_id: u64) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetSupply(asset_id))
        .unwrap_or(0)
}

pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetOwnerCount(asset_id))
        .unwrap_or(0)
}
