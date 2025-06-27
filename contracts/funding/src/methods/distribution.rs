use soroban_sdk::{Address, Env, String, Vec};
use crate::storage::DataKey;
use crate::events;
use crate::interfaces::{FNFTClient, TokenClient};
use crate::methods::{admin, queries, utils};

/// Distribute funds to asset owners (only admin/governance)
/// This is the main distribution function called after governance approval
pub fn distribute_funds(
    env: Env,
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String,
) {
    admin::require_admin_auth(env.clone(), caller);

    if amount == 0 {
        panic!("Distribution amount must be > 0");
    }

    // Verify asset exists and has funds
    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    let current_funds = queries::asset_funds(env.clone(), asset_id);
    if amount > current_funds {
        panic!("Insufficient funds for distribution");
    }

    let total_supply = fnft_client.asset_supply(&asset_id);
    if total_supply == 0 {
        panic!("Asset has no supply");
    }

    // Get all current owners
    let owners = fnft_client.asset_owners(&asset_id);
    if owners.len() == 0 {
        panic!("No asset owners found");
    }

    // Execute distribution
    execute_distribution_logic(
        &env,
        &fnft_client,
        asset_id,
        amount,
        total_supply,
        owners,
        description,
    );
}

/// Allow asset owners to directly distribute funds (democratic distribution)
/// Requires caller to own tokens of the asset
pub fn owner_distribute_funds(
    env: Env,
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String,
) {
    caller.require_auth();

    if amount == 0 {
        panic!("Distribution amount must be > 0");
    }

    // Verify caller owns tokens of this asset
    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.owns_asset(&caller, &asset_id) {
        panic!("Caller does not own tokens of this asset");
    }

    let current_funds = queries::asset_funds(env.clone(), asset_id);
    if amount > current_funds {
        panic!("Insufficient funds for distribution");
    }

    let total_supply = fnft_client.asset_supply(&asset_id);
    if total_supply == 0 {
        panic!("Asset has no supply");
    }

    let owners = fnft_client.asset_owners(&asset_id);
    if owners.len() == 0 {
        panic!("No asset owners found");
    }

    // Execute distribution
    execute_distribution_logic(
        &env,
        &fnft_client,
        asset_id,
        amount,
        total_supply,
        owners,
        description,
    );
}

/// Internal distribution logic (separated to avoid code duplication)
pub fn execute_distribution_logic(
    env: &Env,
    fnft_client: &FNFTClient,
    asset_id: u64,
    amount: u128,
    total_supply: u64,
    owners: Vec<Address>,
    description: String,
) {
    // Update asset funds first
    let current_funds = queries::asset_funds(env.clone(), asset_id);
    env.storage()
        .persistent()
        .set(&DataKey::AssetFunds(asset_id), &(current_funds - amount));

    let mut total_distributed = 0u128;
    let mut recipients_count = 0u32;

    // Get XLM token contract for transfers
    let xlm_token_address = utils::get_xlm_token(env);
    let xlm_token = TokenClient::new(env, &xlm_token_address);

    // Calculate distributions for each owner proportionally
    for owner in owners {
        let balance = fnft_client.balance_of(&owner, &asset_id);

        if balance > 0 {
            // Calculate proportional share: (amount * balance) / total_supply
            let owner_share = (amount * balance as u128) / total_supply as u128;

            if owner_share > 0 {
                // Transfer XLM from contract to owner
                xlm_token.transfer(
                    &env.current_contract_address(),
                    &owner,
                    &(owner_share as i128),
                );

                total_distributed += owner_share;
                recipients_count += 1;

                events::emit_received(env, asset_id, owner, owner_share);
            }
        }
    }

    // Handle any remaining dust (due to integer division)
    if total_distributed < amount {
        let dust = amount - total_distributed;
        // Add dust back to asset funds
        let remaining_funds = queries::asset_funds(env.clone(), asset_id);
        env.storage()
            .persistent()
            .set(&DataKey::AssetFunds(asset_id), &(remaining_funds + dust));
    }

    // Update total distributed with actual amount distributed
    let current_distributed = queries::total_distributed(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::TotalDistributed(asset_id),
        &(current_distributed + total_distributed),
    );

    // Update distribution count
    let distribution_count = queries::get_distribution_count(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::DistributionCount(asset_id),
        &(distribution_count + 1),
    );

    // Emit distribution event
    events::emit_distribution(env, asset_id, total_distributed, description, recipients_count);
}
