use crate::events;
use crate::interfaces::{FNFTClient, TokenClient};
use crate::methods::{admin, queries, utils};
use crate::storage::DataKey;
use soroban_sdk::{Address, Env, String};

/// Distribute funds from asset's SAC to asset owners (admin/governance only)
pub fn distribute_funds(
    env: Env,
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String,
) {
    let admin = admin::get_admin(env.clone());
    let governance_contract = utils::get_governance_contract(&env);

    let is_admin = caller == admin;
    let is_governance = if let Some(gov) = governance_contract {
        caller == gov
    } else {
        false
    };
    if !is_admin && !is_governance {
        panic!("Only admin or governance can distribute funds");
    }

    caller.require_auth();

    execute_sac_distribution(env, asset_id, amount, description);
}

/// Allow asset owners to distribute funds (democratic distribution)
pub fn owner_distribute_funds(
    env: Env,
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String,
) {
    caller.require_auth();

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.owns_asset(&caller, &asset_id) {
        panic!("Caller does not own tokens of this asset");
    }

    execute_sac_distribution(env, asset_id, amount, description);
}

/// Internal distribution logic - pulls from SAC and distributes to asset owners
fn execute_sac_distribution(env: Env, asset_id: u64, amount: u128, description: String) {
    let sac_address: Address = env
        .storage()
        .persistent()
        .get(&DataKey::AssetSAC(asset_id))
        .expect("Asset must have a registered SAC");

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    let total_supply = fnft_client.asset_supply(&asset_id);
    if total_supply == 0 {
        panic!("Asset has no supply");
    }

    let owners = fnft_client.asset_owners(&asset_id);
    if owners.len() == 0 {
        panic!("No asset owners found");
    }

    let sac_client = TokenClient::new(&env, &sac_address);
    let sac_balance = sac_client.balance(&sac_address);

    if (amount as i128) > sac_balance {
        panic!("Insufficient balance in asset SAC");
    }

    let mut total_distributed = 0u128;
    let mut recipients_count = 0u32;

    for owner in owners {
        let balance = fnft_client.balance_of(&owner, &asset_id);

        if balance > 0 {
            let owner_share = (amount * balance as u128) / total_supply as u128;

            if owner_share > 0 {
                sac_client.transfer(&sac_address, &owner, &(owner_share as i128));

                total_distributed += owner_share;
                recipients_count += 1;

                events::emit_received(&env, asset_id, owner, owner_share);
            }
        }
    }

    let current_distributed = queries::total_distributed(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::TotalDistributed(asset_id),
        &(current_distributed + total_distributed),
    );

    let distribution_count = queries::get_distribution_count(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::DistributionCount(asset_id),
        &(distribution_count + 1),
    );

    events::emit_distribution(
        &env,
        asset_id,
        total_distributed,
        description,
        recipients_count,
    );
}
