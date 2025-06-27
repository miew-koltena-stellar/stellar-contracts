use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::events;

/// Initialize the funding contract
/// admin: Address that can execute distributions
/// fnft_contract: Address of the FNFT contract to integrate with
/// xlm_token: Address of the XLM token contract (SAC)
pub fn initialize(env: Env, admin: Address, fnft_contract: Address, xlm_token: Address) {
    if env.storage().instance().has(&DataKey::Admin) {
        panic!("Contract already initialized");
    }

    admin.require_auth();

    // Set admin, FNFT contract, and XLM token
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage()
        .instance()
        .set(&DataKey::FNFTContract, &fnft_contract);
    env.storage().instance().set(&DataKey::XLMToken, &xlm_token);

    // Emit initialization event
    events::emit_init(&env, admin, fnft_contract, xlm_token);
}
