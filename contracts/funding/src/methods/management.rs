use crate::events;
use crate::interfaces::FNFTClient;
use crate::methods::utils;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

/// Register a SAC address for an asset (any asset owner can register)
pub fn register_asset_sac(env: Env, caller: Address, asset_id: u64, sac_address: Address) {
    caller.require_auth();

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    if !fnft_client.owns_asset(&caller, &asset_id) {
        panic!("Only asset owners can register SAC");
    }

    if env.storage().persistent().has(&DataKey::AssetSAC(asset_id)) {
        panic!("Asset already has a registered SAC");
    }

    if sac_address == env.current_contract_address() {
        panic!("SAC address cannot be the funding contract itself");
    }

    env.storage()
        .persistent()
        .set(&DataKey::AssetSAC(asset_id), &sac_address);
    env.storage()
        .persistent()
        .set(&DataKey::SACToAsset(sac_address.clone()), &asset_id);

    events::emit_sac_registered(&env, asset_id, sac_address);
}
