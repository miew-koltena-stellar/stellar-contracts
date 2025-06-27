use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::events;

/// REFACTOR: modifier onlyAdmin from Solidity
/// Converted to helper function
pub fn require_admin_auth(env: Env) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
}

/// REFACTOR: function getAdmin() public view returns (address)
/// Direct implementation
pub fn get_admin(env: Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

/// NEW FUNCTION: Admin role transfer
/// Basic governance functionality
pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
    require_admin_auth(env.clone());
    current_admin.require_auth();

    env.storage().instance().set(&DataKey::Admin, &new_admin);

    events::emit_admin_transfer(&env, current_admin, new_admin);
}
