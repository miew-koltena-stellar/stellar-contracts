use soroban_sdk::{Address, Env};
use crate::storage::DataKey;

/// Get FNFT contract address from storage
pub fn get_fnft_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FNFTContract)
        .unwrap()
}

/// Get XLM token contract address from storage
pub fn get_xlm_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::XLMToken).unwrap()
}
