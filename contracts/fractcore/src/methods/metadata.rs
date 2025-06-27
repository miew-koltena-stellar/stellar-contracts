use soroban_sdk::{Address, Env, String};
use crate::storage::DataKey;
use crate::events;
use crate::methods::admin;
use crate::methods::utils;

/// REFACTOR: function setUri(uint256 _tokenId, string calldata uri_) public
/// Addition: Authorization verification (admin or creator)
/// Change: Explicit caller vs implicit msg.sender
pub fn set_asset_uri(env: Env, caller: Address, asset_id: u64, uri: String) {
    caller.require_auth();

    // Check if asset exists
    if !utils::asset_exists(env.clone(), asset_id) {
        panic!("Asset does not exist");
    }

    // Authorization verification - only admin or asset creator
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    let creator: Address = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCreator(asset_id))
        .unwrap();

    if caller != admin && caller != creator {
        panic!("Not authorized to set URI");
    }

    // Store URI
    env.storage()
        .persistent()
        .set(&DataKey::AssetURI(asset_id), &uri);

    // Emit event
    events::emit_uri_update(&env, asset_id, uri);
}

/// REFACTOR: function uri(uint256 _tokenId) public view returns (string memory)
/// Direct implementation
pub fn asset_uri(env: Env, asset_id: u64) -> Option<String> {
    env.storage().persistent().get(&DataKey::AssetURI(asset_id))
}

/// NEW FUNCTION: Contract-level URI
/// Additional functionality for global metadata
pub fn set_contract_uri(env: Env, caller: Address, uri: String) {
    admin::require_admin_auth(env.clone());
    caller.require_auth();

    env.storage().persistent().set(&DataKey::ContractURI, &uri);
}

/// NEW FUNCTION: Get contract URI
pub fn contract_uri(env: Env) -> Option<String> {
    env.storage().persistent().get(&DataKey::ContractURI)
}

/// NEW FUNCTION: Get asset creator
/// Additional tracking vs original Solidity
pub fn get_asset_creator(env: Env, asset_id: u64) -> Option<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::AssetCreator(asset_id))
}
