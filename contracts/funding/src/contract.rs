use soroban_sdk::{contract, contractimpl, Address, Env, String};
use crate::methods::{admin, distribution, funds, initialization, queries};

#[contract]
pub struct FundingContract;

#[contractimpl]
impl FundingContract {
    /// Initialize the funding contract
    pub fn initialize(env: Env, admin: Address, fnft_contract: Address, xlm_token: Address) {
        initialization::initialize(env, admin, fnft_contract, xlm_token);
    }

    /// Deposit XLM funds for a specific asset
    pub fn deposit_funds(env: Env, depositor: Address, asset_id: u64, amount: i128) {
        funds::deposit_funds(env, depositor, asset_id, amount);
    }

    /// Distribute funds to asset owners (only admin/governance)
    pub fn distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        distribution::distribute_funds(env, caller, asset_id, amount, description);
    }

    /// Allow asset owners to directly distribute funds (democratic distribution)
    pub fn owner_distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        distribution::owner_distribute_funds(env, caller, asset_id, amount, description);
    }

    /// Get total funds available for an asset
    pub fn asset_funds(env: Env, asset_id: u64) -> u128 {
        queries::asset_funds(env, asset_id)
    }

    /// Get total amount distributed for an asset
    pub fn total_distributed(env: Env, asset_id: u64) -> u128 {
        queries::total_distributed(env, asset_id)
    }

    /// Get number of distributions for an asset
    pub fn get_distribution_count(env: Env, asset_id: u64) -> u32 {
        queries::get_distribution_count(env, asset_id)
    }

    /// Get the FNFT contract address
    pub fn get_fnft_contract_address(env: Env) -> Address {
        queries::get_fnft_contract_address(env)
    }

    /// Get the XLM token contract address
    pub fn get_xlm_token_address(env: Env) -> Address {
        queries::get_xlm_token_address(env)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Address {
        admin::get_admin(env)
    }

    /// Check if an address can distribute funds for an asset
    pub fn can_distribute(env: Env, caller: Address, asset_id: u64) -> bool {
        queries::can_distribute(env, caller, asset_id)
    }

    /// Transfer admin role (only current admin)
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        admin::transfer_admin(env, current_admin, new_admin);
    }

    /// Emergency withdraw funds (only admin)
    pub fn emergency_withdraw(
        env: Env,
        admin: Address,
        asset_id: u64,
        amount: u128,
        reason: String,
    ) {
        admin::emergency_withdraw(env, admin, asset_id, amount, reason);
    }
}
