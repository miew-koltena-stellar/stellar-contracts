use crate::events;
use crate::interfaces::{FNFTClient, TokenClient};
use crate::methods::utils;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

/// Deposit XLM funds to asset's SAC (with tracking)
pub fn deposit_funds(env: Env, depositor: Address, asset_id: u64, amount: i128) {
    depositor.require_auth();

    if amount <= 0 {
        panic!("Deposit amount must be > 0");
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    let sac_address = env
        .storage()
        .persistent()
        .get(&DataKey::AssetSAC(asset_id))
        .expect("Asset must have a registered SAC to use funding features");

    let sac_client = TokenClient::new(&env, &sac_address);
    sac_client.transfer(&depositor, &sac_address, &amount);

    events::emit_deposit(&env, asset_id, depositor, amount);
}
