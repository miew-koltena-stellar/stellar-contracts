use crate::events;
use crate::methods::{admin, balance, utils};
use crate::storage::DataKey;
use soroban_sdk::{Address, Env, Vec};

pub fn initialize(env: Env, admin: Address) {
    // Reentrancy protection
    if env.storage().instance().has(&DataKey::Admin) {
        panic!("Contract already initialized");
    }

    admin.require_auth();

    env.storage().instance().set(&DataKey::Admin, &admin);

    env.storage().instance().set(&DataKey::NextAssetId, &1u64);

    events::emit_init(&env, admin);
}

pub fn mint(env: Env, to: Address, num_tokens: u64) -> u64 {
    admin::require_admin_auth(env.clone());

    if num_tokens == 0 {
        panic!("Cannot mint 0 tokens");
    }

    let asset_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextAssetId)
        .unwrap_or(1);

    env.storage()
        .instance()
        .set(&DataKey::NextAssetId, &(asset_id + 1));

    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone(), asset_id), &num_tokens);

    env.storage()
        .persistent()
        .set(&DataKey::AssetSupply(asset_id), &num_tokens);

    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::AssetCreator(asset_id), &admin);

    utils::add_owner_to_asset(&env, asset_id, to.clone());
    utils::add_asset_to_owner(&env, to.clone(), asset_id);

    events::emit_mint(&env, to, asset_id, num_tokens);

    asset_id
}

/// Allows minting to multiple recipients of an existing asset
pub fn mint_to(env: Env, asset_id: u64, recipients: Vec<Address>, amounts: Vec<u64>) {
    admin::require_admin_auth(env.clone());

    if asset_id == 0 {
        panic!("Asset ID cannot be 0 - use mint() to create new assets");
    }

    if !utils::asset_exists(env.clone(), asset_id) {
        panic!("Asset does not exist");
    }

    if recipients.len() != amounts.len() {
        panic!("Recipients and amounts length mismatch");
    }

    if recipients.len() == 0 {
        panic!("No recipients specified");
    }

    let mut total_minted = 0u64;

    for i in 0..recipients.len() {
        let recipient = recipients.get(i).unwrap();
        let amount = amounts.get(i).unwrap();

        if amount == 0 {
            panic!("Cannot mint 0 tokens");
        }

        let current_balance = balance::balance_of(env.clone(), recipient.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::Balance(recipient.clone(), asset_id),
            &(current_balance + amount),
        );

        if current_balance == 0 {
            utils::add_owner_to_asset(&env, asset_id, recipient.clone());
            utils::add_asset_to_owner(&env, recipient.clone(), asset_id);
        }

        total_minted += amount;

        events::emit_mint_to(&env, recipient, asset_id, amount);
    }

    // Update total supply
    let current_supply = balance::asset_supply(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::AssetSupply(asset_id),
        &(current_supply + total_minted),
    );
}
