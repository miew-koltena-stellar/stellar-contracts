use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::events;

/// Initialize the trading contract
pub fn initialize(env: Env, admin: Address, fnft_contract: Address, xlm_contract: Address) {
    // Validate contract initialization state
    if env.storage().instance().has(&DataKey::Admin) {
        panic!("Contract already initialized");
    }

    admin.require_auth();

    // Set admin and contract addresses
    env.storage().instance().set(&DataKey::Admin, &admin);
    env.storage()
        .instance()
        .set(&DataKey::FNFTContract, &fnft_contract);
    env.storage()
        .instance()
        .set(&DataKey::XLMContract, &xlm_contract);
    env.storage().instance().set(&DataKey::TradeCounter, &0u32);

    // Emit initialization event
    events::emit_init_event(&env, &admin, &fnft_contract, &xlm_contract);
}
