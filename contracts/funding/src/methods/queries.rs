use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::interfaces::FNFTClient;
use crate::methods::{admin, utils};

/// Get total funds available for an asset
pub fn asset_funds(env: Env, asset_id: u64) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::AssetFunds(asset_id))
        .unwrap_or(0)
}

/// Get total amount distributed for an asset
pub fn total_distributed(env: Env, asset_id: u64) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalDistributed(asset_id))
        .unwrap_or(0)
}

/// Get number of distributions for an asset
pub fn get_distribution_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::DistributionCount(asset_id))
        .unwrap_or(0)
}

/// Get the FNFT contract address
pub fn get_fnft_contract_address(env: Env) -> Address {
    utils::get_fnft_contract(&env)
}

/// Get the XLM token contract address
pub fn get_xlm_token_address(env: Env) -> Address {
    utils::get_xlm_token(&env)
}

/// Check if an address can distribute funds for an asset
pub fn can_distribute(env: Env, caller: Address, asset_id: u64) -> bool {
    let admin = admin::get_admin(env.clone());
    if caller == admin {
        return true;
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    fnft_client.owns_asset(&caller, &asset_id)
}
