use crate::events;
use crate::methods::admin;
use crate::methods::utils;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env, String};

pub fn set_asset_uri(env: Env, caller: Address, asset_id: u64, uri: String) {
    caller.require_auth();

    if !utils::asset_exists(env.clone(), asset_id) {
        panic!("Asset does not exist");
    }

    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    let creator: Address = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCreator(asset_id))
        .unwrap();

    if caller != admin && caller != creator {
        panic!("Not authorized to set URI");
    }

    env.storage()
        .persistent()
        .set(&DataKey::AssetURI(asset_id), &uri);

    events::emit_uri_update(&env, asset_id, uri);
}

pub fn asset_uri(env: Env, asset_id: u64) -> Option<String> {
    env.storage().persistent().get(&DataKey::AssetURI(asset_id))
}

pub fn set_contract_uri(env: Env, caller: Address, uri: String) {
    admin::require_admin_auth(env.clone());
    caller.require_auth();

    env.storage().persistent().set(&DataKey::ContractURI, &uri);
}

pub fn contract_uri(env: Env) -> Option<String> {
    env.storage().persistent().get(&DataKey::ContractURI)
}

pub fn get_asset_creator(env: Env, asset_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::AssetCreator(asset_id))
}
