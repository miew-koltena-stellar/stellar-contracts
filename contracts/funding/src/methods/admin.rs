use soroban_sdk::{Address, Env, String};
use crate::storage::DataKey;
use crate::events;
use crate::methods::queries;

/// Get the admin address
pub fn get_admin(env: Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

/// Transfer admin role (only current admin)
pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
    current_admin.require_auth();

    let admin = get_admin(env.clone());
    if current_admin != admin {
        panic!("Only current admin can transfer admin role");
    }

    env.storage().instance().set(&DataKey::Admin, &new_admin);

    events::emit_admin_transfer(&env, current_admin, new_admin);
}

/// Emergency withdraw funds (only admin)
/// For emergency situations or contract upgrades
pub fn emergency_withdraw(
    env: Env,
    admin: Address,
    asset_id: u64,
    amount: u128,
    reason: String,
) {
    admin.require_auth();
    require_admin_auth(env.clone(), admin.clone());

    let current_funds = queries::asset_funds(env.clone(), asset_id);
    if amount > current_funds {
        panic!("Insufficient funds for withdrawal");
    }

    // Update funds
    env.storage()
        .persistent()
        .set(&DataKey::AssetFunds(asset_id), &(current_funds - amount));

    // Emit emergency withdrawal event
    events::emit_emergency(&env, asset_id, admin, amount, reason);
}

/// Require admin authorization
pub fn require_admin_auth(env: Env, caller: Address) {
    let admin = get_admin(env);
    if caller != admin {
        panic!("Only admin can perform this action");
    }
}
