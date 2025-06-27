use soroban_sdk::{Address, Env, Vec};
use crate::storage::DataKey;
use crate::events;
use crate::methods::{admin, balance, utils};

/// Contract initialization replacing function initialize(string memory uri_) public virtual initializer
/// Simplification: Removes initial URI, focuses only on admin setup
/// Admin setup is mandatory (Solidity allowed deployment without defined admin)
pub fn initialize(env: Env, admin: Address) {
    // Reentrancy protection (similar to initializer modifier from Solidity)
    if env.storage().instance().has(&DataKey::Admin) {
        panic!("Contract already initialized");
    }

    // Mandatory authorization verification in Soroban
    admin.require_auth();

    // Set admin - replaces _admin = msg.sender; from Solidity
    env.storage().instance().set(&DataKey::Admin, &admin);

    // Initialize asset ID counter - replaces id_counter = 1; from registries
    env.storage().instance().set(&DataKey::NextAssetId, &1u64);

    // Emit initialization event - similar to Solidity events but simpler
    events::emit_init(&env, admin);
}

/// Token minting replacing function mint(uint256 numTokens) public virtual noReentrancy delegateOnly returns (uint256)
/// Simplification: Removes reentrancy guard (Soroban has built-in protections)
/// Change: Mint to specific address instead of msg.sender
/// Returns: asset_id instead of returning via event
pub fn mint(env: Env, to: Address, num_tokens: u64) -> u64 {
    // Admin verification - replaces delegateOnly modifier from Solidity
    admin::require_admin_auth(env.clone());

    // Basic validation - similar to Solidity but more explicit
    if num_tokens == 0 {
        panic!("Cannot mint 0 tokens");
    }

    // Get next asset ID - replaces id_counter++ from Solidity
    let asset_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextAssetId)
        .unwrap_or(1);

    // Increment for next mint
    env.storage()
        .instance()
        .set(&DataKey::NextAssetId, &(asset_id + 1));

    // Set owner balance - replaces registry system from Solidity
    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone(), asset_id), &num_tokens);

    // Set total supply - similar to _totalSupply[tokenId] = n_tokens;
    env.storage()
        .persistent()
        .set(&DataKey::AssetSupply(asset_id), &num_tokens);

    // Creator tracking - new functionality vs Solidity
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    env.storage()
        .persistent()
        .set(&DataKey::AssetCreator(asset_id), &admin);

    // Efficient ownership tracking
    // Replaces complex tree structures from RegistryNestedTree
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerExists(asset_id, to.clone()), &true);
    env.storage()
        .persistent()
        .set(&DataKey::OwnerAssetExists(to.clone(), asset_id), &true);
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerCount(asset_id), &1u32);

    // List maintenance for future systems
    // New functionality - enables efficient queries for funding/voting
    utils::add_owner_to_asset(&env, asset_id, to.clone());
    utils::add_asset_to_owner(&env, to.clone(), asset_id);

    // Emit event - similar to TransferSingle from ERC1155 but simplified
    events::emit_mint(&env, to, asset_id, num_tokens);

    asset_id
}

/// Multiple recipient minting from Solidity's dynamicMint() - simplified version
/// Allows minting to multiple recipients of an existing asset
/// Validation: asset_id must exist (cannot create new assets)
pub fn mint_to(env: Env, asset_id: u64, recipients: Vec<Address>, amounts: Vec<u64>) {
    // Admin verification
    admin::require_admin_auth(env.clone());

    // Asset ID validation - prevents creation of new assets via mint_to
    if asset_id == 0 {
        panic!("Asset ID cannot be 0 - use mint() to create new assets");
    }

    // Verify asset exists - new functionality vs Solidity
    if !utils::asset_exists(env.clone(), asset_id) {
        panic!("Asset does not exist");
    }

    // Input validations - similar to Solidity
    if recipients.len() != amounts.len() {
        panic!("Recipients and amounts length mismatch");
    }

    if recipients.len() == 0 {
        panic!("No recipients specified");
    }

    let mut total_minted = 0u64;        let mut owner_count = balance::get_asset_owner_count(env.clone(), asset_id);

    // Process each recipient
    for i in 0..recipients.len() {
        let recipient = recipients.get(i).unwrap();
        let amount = amounts.get(i).unwrap();

        if amount == 0 {
            panic!("Cannot mint 0 tokens");
        }

        // Update balance (may add to existing balance)
        let current_balance = balance::balance_of(env.clone(), recipient.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::Balance(recipient.clone(), asset_id),
            &(current_balance + amount),
        );

        // Track new ownership only if didn't have tokens before
        if current_balance == 0 {
            env.storage().persistent().set(
                &DataKey::AssetOwnerExists(asset_id, recipient.clone()),
                &true,
            );
            env.storage().persistent().set(
                &DataKey::OwnerAssetExists(recipient.clone(), asset_id),
                &true,
            );
            owner_count += 1;

            // Add to lists for future queries
            utils::add_owner_to_asset(&env, asset_id, recipient.clone());
            utils::add_asset_to_owner(&env, recipient.clone(), asset_id);
        }

        total_minted += amount;

        // Emit event per recipient
        events::emit_mint_to(&env, recipient, asset_id, amount);
    }

    // Update total supply
    let current_supply = balance::asset_supply(env.clone(), asset_id);
    env.storage().persistent().set(
        &DataKey::AssetSupply(asset_id),
        &(current_supply + total_minted),
    );

    // Update owner count
    env.storage()
        .persistent()
        .set(&DataKey::AssetOwnerCount(asset_id), &owner_count);
}
