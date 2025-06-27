use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use crate::methods::{admin, approval, balance, metadata, mint, ownership, transfer};

#[contract]
pub struct FractionalizationContract;

#[contractimpl]
impl FractionalizationContract {
    /// Contract initialization
    pub fn initialize(env: Env, admin: Address) {
        mint::initialize(env, admin);
    }

    /// Token minting - creates new asset
    pub fn mint(env: Env, to: Address, num_tokens: u64) -> u64 {
        mint::mint(env, to, num_tokens)
    }

    /// Multiple recipient minting for existing asset
    pub fn mint_to(env: Env, asset_id: u64, recipients: Vec<Address>, amounts: Vec<u64>) {
        mint::mint_to(env, asset_id, recipients, amounts);
    }

    /// Get balance of owner for specific asset
    pub fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64 {
        balance::balance_of(env, owner, asset_id)
    }

    /// Batch balance query
    pub fn balance_of_batch(env: Env, owners: Vec<Address>, asset_ids: Vec<u64>) -> Vec<u64> {
        balance::balance_of_batch(env, owners, asset_ids)
    }

    /// Get total supply of asset
    pub fn asset_supply(env: Env, asset_id: u64) -> u64 {
        balance::asset_supply(env, asset_id)
    }

    /// Simple transfer (owner transfers their own tokens)
    pub fn transfer(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
        transfer::transfer(env, from, to, asset_id, amount);
    }

    /// Transfer from (with allowance system)
    pub fn transfer_from(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        asset_id: u64,
        amount: u64,
    ) {
        transfer::transfer_from(env, operator, from, to, asset_id, amount);
    }

    /// Batch transfer from
    pub fn batch_transfer_from(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        asset_ids: Vec<u64>,
        amounts: Vec<u64>,
    ) {
        transfer::batch_transfer_from(env, operator, from, to, asset_ids, amounts);
    }

    /// Set approval for all
    pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
        approval::set_approval_for_all(env, owner, operator, approved);
    }

    /// Check if approved for all
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        approval::is_approved_for_all(env, owner, operator)
    }

    /// Approve specific amount for specific asset
    pub fn approve(env: Env, owner: Address, operator: Address, asset_id: u64, amount: u64) {
        approval::approve(env, owner, operator, asset_id, amount);
    }

    /// Get allowance for specific asset
    pub fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64 {
        approval::allowance(env, owner, operator, asset_id)
    }

    /// Get asset owner count
    pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
        ownership::get_asset_owner_count(env, asset_id)
    }

    /// Check if address owns asset
    pub fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool {
        ownership::owns_asset(env, owner, asset_id)
    }

    /// Check if owner has any assets
    pub fn has_assets(env: Env, owner: Address, asset_id: u64) -> bool {
        ownership::has_assets(env, owner, asset_id)
    }

    /// Get all owners of an asset
    pub fn asset_owners(env: Env, asset_id: u64) -> Vec<Address> {
        ownership::asset_owners(env, asset_id)
    }

    /// Get all assets owned by an address
    pub fn owner_assets(env: Env, owner: Address) -> Vec<u64> {
        ownership::owner_assets(env, owner)
    }

    /// Get next asset ID
    pub fn next_asset_id(env: Env) -> u64 {
        crate::methods::utils::next_asset_id(env)
    }

    /// Check if asset exists
    pub fn asset_exists(env: Env, asset_id: u64) -> bool {
        crate::methods::utils::asset_exists(env, asset_id)
    }

    /// Set asset URI
    pub fn set_asset_uri(env: Env, caller: Address, asset_id: u64, uri: String) {
        metadata::set_asset_uri(env, caller, asset_id, uri);
    }

    /// Get asset URI
    pub fn asset_uri(env: Env, asset_id: u64) -> Option<String> {
        metadata::asset_uri(env, asset_id)
    }

    /// Set contract URI
    pub fn set_contract_uri(env: Env, caller: Address, uri: String) {
        metadata::set_contract_uri(env, caller, uri);
    }

    /// Get contract URI
    pub fn contract_uri(env: Env) -> Option<String> {
        metadata::contract_uri(env)
    }

    /// Get contract admin
    pub fn get_admin(env: Env) -> Address {
        admin::get_admin(env)
    }

    /// Get asset creator
    pub fn get_asset_creator(env: Env, asset_id: u64) -> Option<Address> {
        metadata::get_asset_creator(env, asset_id)
    }

    /// Transfer admin role
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        admin::transfer_admin(env, current_admin, new_admin);
    }
}
