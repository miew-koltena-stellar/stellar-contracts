use crate::interfaces::{FNFTClient, TokenClient};
use crate::methods::utils;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

/// Get the SAC address for an asset
pub fn get_asset_sac(env: Env, asset_id: u64) -> Option<Address> {
    env.storage().persistent().get(&DataKey::AssetSAC(asset_id))
}

/// Get asset ID from SAC address (reverse lookup)
pub fn get_asset_by_sac(env: Env, sac_address: Address) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::SACToAsset(sac_address))
}

/// Get SAC balance for an asset
pub fn asset_funds(env: Env, asset_id: u64) -> u128 {
    let sac_address = env
        .storage()
        .persistent()
        .get(&DataKey::AssetSAC(asset_id))
        .expect("Asset must have a registered SAC");

    let sac_client = TokenClient::new(&env, &sac_address);
    sac_client.balance(&sac_address) as u128
}

/// Get total amount distributed for an asset (analytics)
pub fn total_distributed(env: Env, asset_id: u64) -> u128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalDistributed(asset_id))
        .unwrap_or(0)
}

/// Get number of distributions for an asset (analytics)
pub fn get_distribution_count(env: Env, asset_id: u64) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::DistributionCount(asset_id))
        .unwrap_or(0)
}

pub fn get_fnft_contract_address(env: Env) -> Address {
    utils::get_fnft_contract(&env)
}

/// Check if an address can distribute funds for an asset
pub fn can_distribute(env: Env, caller: Address, asset_id: u64) -> bool {
    let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
    if let Some(admin_addr) = admin {
        if caller == admin_addr {
            return true;
        }
    }

    let governance_contract: Option<Address> =
        env.storage().instance().get(&DataKey::GovernanceContract);
    if let Some(gov_addr) = governance_contract {
        if caller == gov_addr {
            return true;
        }
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    fnft_client.owns_asset(&caller, &asset_id)
}
