use soroban_sdk::{Address, Env, Vec};
use crate::storage::DataKey;
use crate::events;
use crate::methods::{approval, balance, ownership, utils};

/// NEW FUNCTION: Simple transfer (owner transfers their own tokens)
/// Simplification vs safeTransferFrom from Solidity
pub fn transfer(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
    // Mandatory authorization verification
    from.require_auth();
    // Delegate to internal logic
    transfer_internal(env, from, to, asset_id, amount);
}

/// REFACTOR: function safeTransferFrom() from ERC1155
/// Simplification: Removes security callback (will be implemented in upper layer)
/// Maintains: Allowance system and authorization
pub fn transfer_from(
    env: Env,
    operator: Address,
    from: Address,
    to: Address,
    asset_id: u64,
    amount: u64,
) {
    // === AUTHORIZATION VERIFICATION ===
    // Simplification vs _verifyTransaction from AllowancesNestedMap
    if operator != from {
        // Check if has approval for all
        let approved_for_all =
            approval::is_approved_for_all(env.clone(), from.clone(), operator.clone());

        if !approved_for_all {
            // Check specific allowance for this token
            let allowance: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::TokenAllowance(
                    from.clone(),
                    operator.clone(),
                    asset_id,
                ))
                .unwrap_or(0);

            if allowance < amount {
                panic!("Insufficient allowance");
            }

            // Decrement allowance - similar to updateAllowanceRecords from Solidity
            env.storage().persistent().set(
                &DataKey::TokenAllowance(from.clone(), operator.clone(), asset_id),
                &(allowance - amount),
            );
        }
    } else {
        // If operator == from, check direct authorization
        from.require_auth();
    }

    // Execute transfer
    transfer_internal(env, from, to, asset_id, amount);
}

/// REFACTOR: function _transferFrom() from various Solidity registries
/// Simplification: Unified logic without overhead of tree structures
/// Addition: Automatic maintenance of owner lists
pub fn transfer_internal(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
    // Basic validations
    if amount == 0 {
        panic!("Cannot transfer 0 tokens");
    }

    if from == to {
        panic!("Cannot transfer to self");
    }

    // Get current balances - direct access vs complex queries from Solidity
    let from_balance = balance::balance_of(env.clone(), from.clone(), asset_id);
    let to_balance = balance::balance_of(env.clone(), to.clone(), asset_id);

    if from_balance < amount {
        panic!("Insufficient balance");
    }

    // Calculate new balances
    let new_from_balance = from_balance - amount;
    let new_to_balance = to_balance + amount;

    // Update balances in storage
    env.storage()
        .persistent()
        .set(&DataKey::Balance(from.clone(), asset_id), &new_from_balance);
    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone(), asset_id), &new_to_balance);

    // === NEW OWNERSHIP TRACKING ===
    // New functionality vs Solidity - automatic list maintenance
    if to_balance == 0 {
        // Recipient is new owner of this asset
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerExists(asset_id, to.clone()), &true);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerAssetExists(to.clone(), asset_id), &true);

        // Increment owner count
        let owner_count = ownership::get_asset_owner_count(env.clone(), asset_id);
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count + 1));

        // Add to lists for efficient queries
        utils::add_owner_to_asset(&env, asset_id, to.clone());
        utils::add_asset_to_owner(&env, to.clone(), asset_id);
    }

    // === OWNERSHIP CLEANUP ===
    // Remove sender from lists if balance became 0
    if new_from_balance == 0 {
        utils::remove_owner_from_asset(&env, asset_id, from.clone());
        utils::remove_asset_from_owner(&env, from.clone(), asset_id);

        // Decrement owner count
        let owner_count = ownership::get_asset_owner_count(env.clone(), asset_id);
        if owner_count > 0 {
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count - 1));
        }
    }

    // Emit transfer event
    events::emit_transfer(&env, from, to, asset_id, amount);
}

/// REFACTOR: function safeBatchTransferFrom() from ERC1155
/// Simplification: Removes security callback, maintains authorization logic
pub fn batch_transfer_from(
    env: Env,
    operator: Address,
    from: Address,
    to: Address,
    asset_ids: Vec<u64>,
    amounts: Vec<u64>,
) {
    // Array validation
    if asset_ids.len() != amounts.len() {
        panic!("Asset IDs and amounts length mismatch");
    }

    // Execute individual transfers - each with their own validations
    for i in 0..asset_ids.len() {
        let asset_id = asset_ids.get(i).unwrap();
        let amount = amounts.get(i).unwrap();
        transfer_from(
            env.clone(),
            operator.clone(),
            from.clone(),
            to.clone(),
            asset_id,
            amount,
        );
    }
}
