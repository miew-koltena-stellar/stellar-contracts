use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::contract::{GovernanceParams, Poll};

// Storage keys
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    FractcoreContract,
    FundingContract,
    GovernanceParams,
    PollCounter,
    Poll(u32),
    AssetPolls(u64),
    ActivePolls,
}

// Initialization
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::Initialized, &true);
}

// Admin
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

// Contract addresses
pub fn get_fractcore_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FractcoreContract)
        .unwrap()
}

pub fn set_fractcore_contract(env: &Env, contract: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::FractcoreContract, contract);
}

pub fn get_funding_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::FundingContract)
        .unwrap()
}

pub fn set_funding_contract(env: &Env, contract: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::FundingContract, contract);
}

// Governance parameters
pub fn get_governance_params(env: &Env) -> GovernanceParams {
    env.storage()
        .instance()
        .get(&DataKey::GovernanceParams)
        .unwrap()
}

pub fn set_governance_params(env: &Env, params: &GovernanceParams) {
    env.storage()
        .instance()
        .set(&DataKey::GovernanceParams, params);
}

// Poll management
pub fn get_next_poll_id(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::PollCounter)
        .unwrap_or(1u32)
}

pub fn increment_poll_counter(env: &Env) {
    let current = get_next_poll_id(env);
    env.storage()
        .instance()
        .set(&DataKey::PollCounter, &(current + 1));
}

pub fn get_poll(env: &Env, poll_id: u32) -> Option<Poll> {
    env.storage().persistent().get(&DataKey::Poll(poll_id))
}

pub fn set_poll(env: &Env, poll_id: u32, poll: &Poll) {
    env.storage()
        .persistent()
        .set(&DataKey::Poll(poll_id), poll);
}

// Asset polls tracking
pub fn get_asset_polls(env: &Env, asset_id: u64) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::AssetPolls(asset_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_asset_poll(env: &Env, asset_id: u64, poll_id: u32) {
    let mut polls = get_asset_polls(env, asset_id);
    polls.push_back(poll_id);
    env.storage()
        .persistent()
        .set(&DataKey::AssetPolls(asset_id), &polls);
}

// Active polls tracking
pub fn get_active_polls(env: &Env) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::ActivePolls)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_active_poll(env: &Env, poll_id: u32) {
    let mut active_polls = get_active_polls(env);
    active_polls.push_back(poll_id);
    env.storage()
        .persistent()
        .set(&DataKey::ActivePolls, &active_polls);
}

pub fn remove_active_poll(env: &Env, poll_id: u32) {
    let active_polls = get_active_polls(env);
    let mut new_polls = Vec::new(env);

    for i in 0..active_polls.len() {
        if let Some(id) = active_polls.get(i) {
            if id != poll_id {
                new_polls.push_back(id);
            }
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::ActivePolls, &new_polls);
}
