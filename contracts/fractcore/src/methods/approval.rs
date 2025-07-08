use crate::events;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
    owner.require_auth();

    // Store approval - direct storage
    env.storage().persistent().set(
        &DataKey::OperatorApproval(owner.clone(), operator.clone()),
        &approved,
    );

    events::emit_approval_for_all(&env, owner, operator, approved);
}

pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OperatorApproval(owner, operator))
        .unwrap_or(false)
}

pub fn approve(env: Env, owner: Address, operator: Address, asset_id: u64, amount: u64) {
    owner.require_auth();

    // Store specific allowance
    env.storage().persistent().set(
        &DataKey::TokenAllowance(owner.clone(), operator.clone(), asset_id),
        &amount,
    );

    events::emit_approve(&env, owner, operator, asset_id, amount);
}

pub fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TokenAllowance(owner, operator, asset_id))
        .unwrap_or(0)
}
