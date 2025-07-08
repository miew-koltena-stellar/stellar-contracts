use crate::storage::DataKey;
use soroban_sdk::{Address, Env};

pub fn get_fnft_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FNFTContract)
        .unwrap()
}

pub fn get_governance_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::GovernanceContract)
}
