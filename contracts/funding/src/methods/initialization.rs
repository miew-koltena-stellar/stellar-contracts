use crate::events;
use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

/// Initialize the funding contract
pub fn initialize(env: Env, admin: Address, fnft_contract: Address) {
    admin.require_auth();

    if env.storage().instance().has(&DataKey::Admin) {
        panic!("Contract already initialized");
    }

    // Store core addresses
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage()
        .instance()
        .set(&DataKey::FNFTContract, &fnft_contract);

    events::emit_init(&env, admin, fnft_contract);
}
