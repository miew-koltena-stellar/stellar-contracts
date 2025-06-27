#![cfg(test)]

use crate::contract::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// Import the FNFT contract for testing
mod fnft {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/fractcore.wasm");
}

// Create a mock token contract for testing XLM transfers
mod token {
    use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

    #[contract]
    pub struct MockToken;

    #[contracttype]
    pub enum DataKey {
        Balance(Address),
    }

    #[contractimpl]
    impl MockToken {
        pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {
            // Mock implementation - just succeed for testing
        }

        pub fn balance(_env: Env, _id: Address) -> i128 {
            // Mock implementation
            1000000 // Return a large balance for testing
        }
    }
}

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    FundingContractClient<'static>,
    fnft::Client<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy FNFT contract
    let fnft_contract_id = env.register(fnft::WASM, ());
    let fnft_client = fnft::Client::new(&env, &fnft_contract_id);

    // Deploy mock XLM token contract
    let xlm_token_id = env.register(token::MockToken, ());

    // Deploy Funding contract
    let funding_contract_id = env.register(FundingContract, ());
    let funding_client = FundingContractClient::new(&env, &funding_contract_id);

    let admin = Address::generate(&env);

    // Initialize FNFT contract
    fnft_client.initialize(&admin);

    // Initialize Funding contract with XLM token address
    funding_client.initialize(&admin, &fnft_contract_id, &xlm_token_id);

    (
        env,
        admin,
        fnft_contract_id,
        xlm_token_id,
        funding_client,
        fnft_client,
    )
}

#[test]
fn test_initialize_funding_contract() {
    let (_env, admin, fnft_contract_id, xlm_token_id, funding_client, _fnft_client) = setup();

    assert_eq!(funding_client.get_admin(), admin);
    assert_eq!(funding_client.get_fnft_contract_address(), fnft_contract_id);
    assert_eq!(funding_client.get_xlm_token_address(), xlm_token_id);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_double_initialization() {
    let (env, _admin, fnft_contract_id, xlm_token_id, funding_client, _fnft_client) = setup();
    let new_admin = Address::generate(&env);

    // Second initialization should panic
    funding_client.initialize(&new_admin, &fnft_contract_id, &xlm_token_id);
}

#[test]
fn test_deposit_funds() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor = Address::generate(&env);

    // Create an asset first
    let asset_id = fnft_client.mint(&depositor, &100);

    // Deposit funds (now using i128 for XLM amounts)
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);

    // Check funds were deposited
    assert_eq!(funding_client.asset_funds(&asset_id), 1000u128);
}

#[test]
#[should_panic(expected = "Deposit amount must be > 0")]
fn test_deposit_zero_amount() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor = Address::generate(&env);

    let asset_id = fnft_client.mint(&depositor, &100);

    // Try to deposit 0 amount
    funding_client.deposit_funds(&depositor, &asset_id, &0i128);
}

#[test]
#[should_panic(expected = "Asset does not exist")]
fn test_deposit_nonexistent_asset() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, _fnft_client) = setup();
    let depositor = Address::generate(&env);

    // Try to deposit for non-existent asset
    funding_client.deposit_funds(&depositor, &999, &1000i128);
}

#[test]
fn test_multiple_deposits() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor1 = Address::generate(&env);
    let depositor2 = Address::generate(&env);

    let asset_id = fnft_client.mint(&depositor1, &100);

    // Multiple deposits should accumulate
    funding_client.deposit_funds(&depositor1, &asset_id, &500i128);
    funding_client.deposit_funds(&depositor2, &asset_id, &300i128);
    funding_client.deposit_funds(&depositor1, &asset_id, &200i128);

    assert_eq!(funding_client.asset_funds(&asset_id), 1000u128);
}

#[test]
fn test_admin_distribute_funds() {
    let (env, admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let depositor = Address::generate(&env);

    // Create asset and distribute tokens
    let asset_id = fnft_client.mint(&owner1, &600); // owner1 has 60%
    fnft_client.transfer(&owner1, &owner2, &asset_id, &400); // owner2 has 40%

    // Deposit funds
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);

    let description = String::from_str(&env, "Test distribution");

    // Distribution should work
    funding_client.distribute_funds(&admin, &asset_id, &1000u128, &description);

    // Check distribution completed (allowing for dust due to integer division)
    assert!(funding_client.asset_funds(&asset_id) <= 1u128); // Allow for 1 unit of dust
    assert!(funding_client.total_distributed(&asset_id) >= 999u128); // At least 999 distributed
    assert_eq!(funding_client.get_distribution_count(&asset_id), 1u32);
}

#[test]
fn test_admin_transfer() {
    let (env, admin, _fnft_contract_id, _xlm_token_id, funding_client, _fnft_client) = setup();
    let new_admin = Address::generate(&env);

    // Transfer admin role
    funding_client.transfer_admin(&admin, &new_admin);

    // Verify new admin
    assert_eq!(funding_client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Only current admin can transfer admin role")]
fn test_unauthorized_admin_transfer() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, _fnft_client) = setup();
    let unauthorized = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Unauthorized user tries to transfer admin
    funding_client.transfer_admin(&unauthorized, &new_admin);
}

#[test]
fn test_emergency_withdraw() {
    let (env, admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor = Address::generate(&env);

    let asset_id = fnft_client.mint(&depositor, &100);
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);

    let reason = String::from_str(&env, "Contract upgrade");

    // Admin can emergency withdraw
    funding_client.emergency_withdraw(&admin, &asset_id, &300u128, &reason);

    // Check funds were reduced
    assert_eq!(funding_client.asset_funds(&asset_id), 700u128);
}

#[test]
#[should_panic(expected = "Only admin can perform this action")]
fn test_unauthorized_emergency_withdraw() {
    let (env, _admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let asset_id = fnft_client.mint(&depositor, &100);
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);

    let reason = String::from_str(&env, "Unauthorized withdrawal");

    // Unauthorized user tries emergency withdraw
    funding_client.emergency_withdraw(&unauthorized, &asset_id, &300u128, &reason);
}

#[test]
#[should_panic(expected = "Insufficient funds for withdrawal")]
fn test_emergency_withdraw_excessive_amount() {
    let (env, admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let depositor = Address::generate(&env);

    let asset_id = fnft_client.mint(&depositor, &100);
    funding_client.deposit_funds(&depositor, &asset_id, &500i128);

    let reason = String::from_str(&env, "Excessive withdrawal");

    // Try to withdraw more than available
    funding_client.emergency_withdraw(&admin, &asset_id, &1000u128, &reason);
}

#[test]
fn test_can_distribute_permissions() {
    let (env, admin, _fnft_contract_id, _xlm_token_id, funding_client, fnft_client) = setup();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);

    let asset_id = fnft_client.mint(&owner, &100);

    // Admin can always distribute
    assert!(funding_client.can_distribute(&admin, &asset_id));

    // Owner can distribute
    assert!(funding_client.can_distribute(&owner, &asset_id));

    // Non-owner cannot distribute
    assert!(!funding_client.can_distribute(&non_owner, &asset_id));
}

#[test]
fn test_view_functions() {
    let (env, admin, fnft_contract_id, xlm_token_id, funding_client, fnft_client) = setup();
    let owner = Address::generate(&env);
    let depositor = Address::generate(&env);

    let asset_id = fnft_client.mint(&owner, &100);

    // Initially no funds or distributions
    assert_eq!(funding_client.asset_funds(&asset_id), 0u128);
    assert_eq!(funding_client.total_distributed(&asset_id), 0u128);
    assert_eq!(funding_client.get_distribution_count(&asset_id), 0u32);

    // Deposit and check
    funding_client.deposit_funds(&depositor, &asset_id, &1500i128);
    assert_eq!(funding_client.asset_funds(&asset_id), 1500u128);

    // Distribute and check
    let description = String::from_str(&env, "Test Distribution");
    funding_client.distribute_funds(&admin, &asset_id, &800u128, &description);

    assert_eq!(funding_client.asset_funds(&asset_id), 700u128);
    assert_eq!(funding_client.total_distributed(&asset_id), 800u128);
    assert_eq!(funding_client.get_distribution_count(&asset_id), 1u32);

    // Check contract addresses
    assert_eq!(funding_client.get_fnft_contract_address(), fnft_contract_id);
    assert_eq!(funding_client.get_xlm_token_address(), xlm_token_id);
    assert_eq!(funding_client.get_admin(), admin);
}
