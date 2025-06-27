use soroban_sdk::{Address, Env};
use crate::storage::DataKey;
use crate::events;

/// REFACTOR: function setApprovalForAll() from ERC1155
/// Full functionality maintained for compatibility
pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
    // Authorization verification
    owner.require_auth();

    // Store approval - direct storage vs nested mappings from Solidity
    env.storage().persistent().set(
        &DataKey::OperatorApproval(owner.clone(), operator.clone()),
        &approved,
    );

    // Emit event
    events::emit_approval_for_all(&env, owner, operator, approved);
}

/// REFACTOR: function isApprovedForAll() from ERC1155
/// Direct implementation
pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::OperatorApproval(owner, operator))
        .unwrap_or(false)
}

/// REFACTOR: function approval() from Solidity (AllowancesNestedMap)
/// Renamed to approve for ERC20/ERC1155 compatibility
pub fn approve(env: Env, owner: Address, operator: Address, asset_id: u64, amount: u64) {
    owner.require_auth();

    // Store specific allowance
    env.storage().persistent().set(
        &DataKey::TokenAllowance(owner.clone(), operator.clone(), asset_id),
        &amount,
    );

    // Emit event
    events::emit_approve(&env, owner, operator, asset_id, amount);
}

/// REFACTOR: function getAllowance() from AllowancesNestedMap
/// Renamed to allowance for standard compatibility
pub fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TokenAllowance(owner, operator, asset_id))
        .unwrap_or(0)
}
