use crate::methods::{admin, distribution, funds, initialization, management, queries};
use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct FundingContract;

#[contractimpl]
impl FundingContract {
    /// TODO: Emergency withdraw from asset's SAC by Poll

    pub fn initialize(env: Env, admin: Address, fnft_contract: Address) {
        initialization::initialize(env, admin, fnft_contract);
    }

    pub fn set_governance_contract(env: Env, admin: Address, governance_contract: Address) {
        admin::set_governance_contract(env, admin, governance_contract);
    }
    /// Register SAC address for an asset
    pub fn register_asset_sac(env: Env, caller: Address, asset_id: u64, sac_address: Address) {
        management::register_asset_sac(env, caller, asset_id, sac_address);
    }

    /// Get the SAC address for an asset
    pub fn get_asset_sac(env: Env, asset_id: u64) -> Option<Address> {
        queries::get_asset_sac(env, asset_id)
    }

    /// Get asset ID from SAC address
    pub fn get_asset_by_sac(env: Env, sac_address: Address) -> Option<u64> {
        queries::get_asset_by_sac(env, sac_address)
    }

    /// Deposit funds to asset's SAC (with tracking)
    pub fn deposit_funds(env: Env, depositor: Address, asset_id: u64, amount: i128) {
        funds::deposit_funds(env, depositor, asset_id, amount);
    }

    /// Distribute funds from asset's SAC to Asset Owners
    pub fn distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        distribution::distribute_funds(env, caller, asset_id, amount, description);
    }

    /// Allow asset owners to distribute funds
    pub fn owner_distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        distribution::owner_distribute_funds(env, caller, asset_id, amount, description);
    }

    /// Get SAC balance for an asset
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

    pub fn get_fnft_contract_address(env: Env) -> Address {
        queries::get_fnft_contract_address(env)
    }

    pub fn get_admin(env: Env) -> Address {
        admin::get_admin(env)
    }

    /// Check if an address can distribute funds for an asset
    pub fn can_distribute(env: Env, caller: Address, asset_id: u64) -> bool {
        queries::can_distribute(env, caller, asset_id)
    }

    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        admin::transfer_admin(env, current_admin, new_admin);
    }
}
