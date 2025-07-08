use crate::events;
use crate::methods::{approval, balance, utils};
use crate::storage::DataKey;
use soroban_sdk::{Address, Env, Vec};

pub fn transfer(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
    from.require_auth();
    transfer_internal(env, from, to, asset_id, amount);
}

pub fn transfer_from(
    env: Env,
    operator: Address,
    from: Address,
    to: Address,
    asset_id: u64,
    amount: u64,
) {
    if operator != from {
        let approved_for_all =
            approval::is_approved_for_all(env.clone(), from.clone(), operator.clone());

        if !approved_for_all {
            let allowance: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TokenAllowance(
                    from.clone(),
                    operator.clone(),
                    asset_id,
                ))
                .unwrap_or(0);

            if allowance < amount {
                panic!("Insufficient allowance");
            }

            env.storage().persistent().set(
                &DataKey::TokenAllowance(from.clone(), operator.clone(), asset_id),
                &(allowance - amount),
            );
        }
    } else {
        from.require_auth();
    }

    transfer_internal(env, from, to, asset_id, amount);
}

pub fn transfer_internal(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
    if amount == 0 {
        panic!("Cannot transfer 0 tokens");
    }

    if from == to {
        panic!("Cannot transfer to self");
    }

    let from_balance = balance::balance_of(env.clone(), from.clone(), asset_id);
    let to_balance = balance::balance_of(env.clone(), to.clone(), asset_id);

    if from_balance < amount {
        panic!("Insufficient balance");
    }

    let new_from_balance = from_balance - amount;
    let new_to_balance = to_balance + amount;

    env.storage()
        .persistent()
        .set(&DataKey::Balance(from.clone(), asset_id), &new_from_balance);
    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone(), asset_id), &new_to_balance);

    if to_balance == 0 {
        utils::add_owner_to_asset(&env, asset_id, to.clone());
        utils::add_asset_to_owner(&env, to.clone(), asset_id);
    }

    if new_from_balance == 0 {
        utils::remove_owner_from_asset(&env, asset_id, from.clone());
        utils::remove_asset_from_owner(&env, from.clone(), asset_id);
    }

    events::emit_transfer(&env, from, to, asset_id, amount);
}
pub fn batch_transfer_from(
    env: Env,
    operator: Address,
    from: Address,
    to: Address,
    asset_ids: Vec<u64>,
    amounts: Vec<u64>,
) {
    // Array validation
    if asset_ids.len() != amounts.len() {
        panic!("Asset IDs and amounts length mismatch");
    }

    for i in 0..asset_ids.len() {
        let asset_id = asset_ids.get(i).unwrap();
        let amount = amounts.get(i).unwrap();
        transfer_from(
            env.clone(),
            operator.clone(),
            from.clone(),
            to.clone(),
            asset_id,
            amount,
        );
    }
}
