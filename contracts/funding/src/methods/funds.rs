use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::events;
use crate::interfaces::{FNFTClient, TokenClient};
use crate::methods::{queries, utils};

/// Deposit XLM funds for a specific asset
/// The contract will hold the XLM and track it for the asset
pub fn deposit_funds(env: Env, depositor: Address, asset_id: u64, amount: i128) {
    depositor.require_auth();

    if amount <= 0 {
        panic!("Deposit amount must be > 0");
    }

    // Verify asset exists
    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    // Get XLM token contract and transfer from depositor to this contract
    let xlm_token_address = utils::get_xlm_token(&env);
    let xlm_token = TokenClient::new(&env, &xlm_token_address);

    // Transfer XLM from depositor to this contract
    xlm_token.transfer(&depositor, &env.current_contract_address(), &amount);

    // Update asset funds tracking
    let current_funds = queries::asset_funds(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::AssetFunds(asset_id),
        &(current_funds + amount as u128),
    );

    // Emit deposit event
    events::emit_deposit(&env, asset_id, depositor, amount);
}
