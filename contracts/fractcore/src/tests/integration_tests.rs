#![cfg(test)]

use crate::contract::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, FractionalizationContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FractionalizationContract, ());
    let client = FractionalizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    (env, admin, client)
}

// === Integration Tests for Complete Workflows ===

#[test]
fn test_mint_to_existing_asset() {
    let (env, _admin, client) = setup();
    let original_owner = Address::generate(&env);
    let new_owner1 = Address::generate(&env);
    let new_owner2 = Address::generate(&env);

    // Create initial asset
    let asset_id = client.mint(&original_owner, &100);

    // Mint to multiple recipients including existing owner
    let mut recipients = soroban_sdk::Vec::new(&env);
    recipients.push_back(original_owner.clone()); // existing owner
    recipients.push_back(new_owner1.clone()); // new owner
    recipients.push_back(new_owner2.clone()); // new owner

    let mut amounts = soroban_sdk::Vec::new(&env);
    amounts.push_back(50); // add to existing owner
    amounts.push_back(75); // new owner
    amounts.push_back(25); // new owner

    client.mint_to(&asset_id, &recipients, &amounts);

    // Check balances
    assert_eq!(client.balance_of(&original_owner, &asset_id), 150);
    assert_eq!(client.balance_of(&new_owner1, &asset_id), 75);
    assert_eq!(client.balance_of(&new_owner2, &asset_id), 25);

    // Check total supply and owner count
    assert_eq!(client.asset_supply(&asset_id), 250);
    assert_eq!(client.get_asset_owner_count(&asset_id), 3);

    // Test owner list is updated correctly
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 3);

    // Check all owners are present (order may vary)
    let mut found_original = false;
    let mut found_new1 = false;
    let mut found_new2 = false;

    for i in 0..owners.len() {
        let owner = owners.get(i).unwrap();
        if owner == original_owner {
            found_original = true;
        }
        if owner == new_owner1 {
            found_new1 = true;
        }
        if owner == new_owner2 {
            found_new2 = true;
        }
    }

    assert!(found_original);
    assert!(found_new1);
    assert!(found_new2);
}

#[test]
fn test_transfer_with_list_updates() {
    let (env, _admin, client) = setup();
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let asset_id = client.mint(&from, &100);

    // Initially only 'from' is owner
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 1);
    assert_eq!(owners.get(0).unwrap(), from);

    // Transfer some tokens
    client.transfer(&from, &to, &asset_id, &30);

    // Now both should be owners
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 2);
    assert_eq!(client.get_asset_owner_count(&asset_id), 2);

    // Check both are in the list
    let mut found_from = false;
    let mut found_to = false;
    for i in 0..owners.len() {
        let owner = owners.get(i).unwrap();
        if owner == from {
            found_from = true;
        }
        if owner == to {
            found_to = true;
        }
    }
    assert!(found_from);
    assert!(found_to);

    // Check 'to' has asset in their list
    let to_assets = client.owner_assets(&to);
    assert_eq!(to_assets.len(), 1);
    assert_eq!(to_assets.get(0).unwrap(), asset_id);
}

#[test]
fn test_transfer_all_tokens_removes_from_lists() {
    let (env, _admin, client) = setup();
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let asset_id = client.mint(&from, &100);

    // Transfer ALL tokens away from 'from'
    client.transfer(&from, &to, &asset_id, &100);

    // 'from' should be removed from owners list
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 1);
    assert_eq!(owners.get(0).unwrap(), to);
    assert_eq!(client.get_asset_owner_count(&asset_id), 1);

    // 'from' should not have this asset in their list anymore
    let from_assets = client.owner_assets(&from);
    assert_eq!(from_assets.len(), 0);

    // But 'to' should have it
    let to_assets = client.owner_assets(&to);
    assert_eq!(to_assets.len(), 1);
    assert_eq!(to_assets.get(0).unwrap(), asset_id);
}

#[test]
fn test_multiple_assets_per_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);

    // Create multiple assets for same owner
    let asset1 = client.mint(&owner, &100);
    let asset2 = client.mint(&owner, &200);
    let asset3 = client.mint(&owner, &300);

    // Check owner has all assets
    let assets = client.owner_assets(&owner);
    assert_eq!(assets.len(), 3);

    // Check all assets are present (order may vary)
    let mut found_asset1 = false;
    let mut found_asset2 = false;
    let mut found_asset3 = false;

    for i in 0..assets.len() {
        let asset = assets.get(i).unwrap();
        if asset == asset1 {
            found_asset1 = true;
        }
        if asset == asset2 {
            found_asset2 = true;
        }
        if asset == asset3 {
            found_asset3 = true;
        }
    }

    assert!(found_asset1);
    assert!(found_asset2);
    assert!(found_asset3);

    // Each asset should have only this owner
    for asset_id in [asset1, asset2, asset3] {
        let owners = client.asset_owners(&asset_id);
        assert_eq!(owners.len(), 1);
        assert_eq!(owners.get(0).unwrap(), owner);
    }
}

#[test]
fn test_complex_transfer_scenario_with_lists() {
    let (env, _admin, client) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    // Create asset with initial owner
    let asset_id = client.mint(&user1, &1000);

    // Add more tokens to existing and new owners via mint_to
    let mut recipients = soroban_sdk::Vec::new(&env);
    recipients.push_back(user1.clone()); // existing owner gets more
    recipients.push_back(user2.clone()); // new owner
    recipients.push_back(user3.clone()); // new owner

    let mut amounts = soroban_sdk::Vec::new(&env);
    amounts.push_back(500); // user1 gets 500 more (total 1500)
    amounts.push_back(300); // user2 gets 300
    amounts.push_back(200); // user3 gets 200

    client.mint_to(&asset_id, &recipients, &amounts);

    // Verify all are in owners list
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 3);
    assert_eq!(client.get_asset_owner_count(&asset_id), 3);

    // Do some transfers
    client.transfer(&user1, &user2, &asset_id, &100); // user1 -> user2
    client.transfer(&user3, &user1, &asset_id, &50); // user3 -> user1

    // All should still be owners (none have 0 balance)
    let owners_after = client.asset_owners(&asset_id);
    assert_eq!(owners_after.len(), 3);

    // Transfer all of user3's remaining tokens away
    client.transfer(&user3, &user2, &asset_id, &150); // user3 -> user2 (user3 now has 0)

    // user3 should be removed from owners list
    let final_owners = client.asset_owners(&asset_id);
    assert_eq!(final_owners.len(), 2);
    assert_eq!(client.get_asset_owner_count(&asset_id), 2);

    // user3 should not have this asset in their list
    let user3_assets = client.owner_assets(&user3);
    assert_eq!(user3_assets.len(), 0);

    // But user1 and user2 should still have it
    let user1_assets = client.owner_assets(&user1);
    assert_eq!(user1_assets.len(), 1);
    assert_eq!(user1_assets.get(0).unwrap(), asset_id);

    let user2_assets = client.owner_assets(&user2);
    assert_eq!(user2_assets.len(), 1);
    assert_eq!(user2_assets.get(0).unwrap(), asset_id);
}

#[test]
fn test_empty_lists_for_new_users() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);

    // User who has never owned any assets
    let assets = client.owner_assets(&user);
    assert_eq!(assets.len(), 0);

    // Owners of non-existent asset
    let owners = client.asset_owners(&999);
    assert_eq!(owners.len(), 0);
}

#[test]
fn test_list_consistency_with_approvals() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let recipient = Address::generate(&env);

    let asset_id = client.mint(&owner, &100);

    // Set approval for all
    client.set_approval_for_all(&owner, &operator, &true);

    // Operator transfers tokens
    client.transfer_from(&operator, &owner, &recipient, &asset_id, &30);

    // Both owner and recipient should be in owners list
    let owners = client.asset_owners(&asset_id);
    assert_eq!(owners.len(), 2);

    // Both should have the asset in their asset lists
    let owner_assets = client.owner_assets(&owner);
    assert_eq!(owner_assets.len(), 1);

    let recipient_assets = client.owner_assets(&recipient);
    assert_eq!(recipient_assets.len(), 1);
}
