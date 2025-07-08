use crate::events;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

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

pub fn require_admin_auth(env: Env, caller: Address) {
    let admin = get_admin(env);
    if caller != admin {
        panic!("Only admin can perform this action");
    }
}

pub fn set_governance_contract(env: Env, admin: Address, governance_contract: Address) {
    admin.require_auth();

    let current_admin = get_admin(env.clone());
    if admin != current_admin {
        panic!("Only admin can set governance contract");
    }

    env.storage()
        .instance()
        .set(&DataKey::GovernanceContract, &governance_contract);
}

pub fn get_governance_contract(env: Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::GovernanceContract)
}

/// Check if caller is authorized (admin or governance contract)
pub fn require_authorized_auth(env: Env, caller: Address) {
    let admin = get_admin(env.clone());
    let governance_contract = get_governance_contract(env);

    if caller != admin {
        if let Some(gov_contract) = governance_contract {
            if caller != gov_contract {
                panic!("Only admin or governance contract can perform this action");
            }
        } else {
            panic!("Only admin can perform this action");
        }
    }
}
