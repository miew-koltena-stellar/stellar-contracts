use crate::events;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

pub fn require_admin_auth(env: Env) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
}

pub fn get_admin(env: Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
    require_admin_auth(env.clone());
    current_admin.require_auth();

    env.storage().instance().set(&DataKey::Admin, &new_admin);

    events::emit_admin_transfer(&env, current_admin, new_admin);
}
