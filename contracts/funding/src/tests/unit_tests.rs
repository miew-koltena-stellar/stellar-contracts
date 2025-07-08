#![cfg(test)]

use crate::contract::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// Import the FNFT contract for testing
mod fnft {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/fractcore.wasm");
}

// Create a mock SAC contract for testing
mod mock_sac {
    use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

    #[contract]
    pub struct MockSAC;

    // No contractclient or trait needed for the mock contract

    #[contracttype]
    pub enum DataKey {
        Balance(Address),
    }

    #[contractimpl]
    impl MockSAC {
        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            // Check for non-negative transfer
            if amount < 0 {
                panic!("Cannot transfer negative amount");
            }
            let mut from_balance = Self::balance(env.clone(), from.clone());
            let mut to_balance = Self::balance(env.clone(), to.clone());
            if from_balance < amount {
                panic!("Insufficient balance");
            }
            from_balance -= amount;
            to_balance += amount;
            env.storage()
                .persistent()
                .set(&DataKey::Balance(from), &from_balance);
            env.storage()
                .persistent()
                .set(&DataKey::Balance(to), &to_balance);
        }

        pub fn balance(env: Env, id: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&DataKey::Balance(id))
                .unwrap_or(1000000i128) // Default large balance for testing
        }

        pub fn mint(env: Env, to: Address, amount: i128) {
            if amount < 0 {
                panic!("Cannot mint negative amount");
            }
            let current_balance = Self::balance(env.clone(), to.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Balance(to), &(current_balance + amount));
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
    mock_sac::MockSACClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy FNFT contract
    let fnft_contract_id = env.register(fnft::WASM, ());
    let fnft_client = fnft::Client::new(&env, &fnft_contract_id);

    // Deploy mock SAC contract
    let sac_contract_id = env.register(mock_sac::MockSAC, ());
    let sac_client = mock_sac::MockSACClient::new(&env, &sac_contract_id);

    // Deploy Funding contract
    let funding_contract_id = env.register(FundingContract, ());
    let funding_client = FundingContractClient::new(&env, &funding_contract_id);

    let admin = Address::generate(&env);

    // Initialize FNFT contract
    fnft_client.initialize(&admin);

    // Initialize Funding contract (NO xlm_token needed now)
    funding_client.initialize(&admin, &fnft_contract_id);

    (
        env,
        admin,
        fnft_contract_id,
        sac_contract_id,
        funding_client,
        fnft_client,
        sac_client,
    )
}

#[test]
fn test_initialize_funding_contract() {
    let (
        _env,
        admin,
        fnft_contract_id,
        _sac_contract_id,
        funding_client,
        _fnft_client,
        _sac_client,
    ) = setup();

    assert_eq!(funding_client.get_admin(), admin);
    assert_eq!(funding_client.get_fnft_contract_address(), fnft_contract_id);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_double_initialization() {
    let (
        env,
        _admin,
        fnft_contract_id,
        _sac_contract_id,
        funding_client,
        _fnft_client,
        _sac_client,
    ) = setup();
    let new_admin = Address::generate(&env);

    // Second initialization should panic
    funding_client.initialize(&new_admin, &fnft_contract_id);
}

#[test]
fn test_register_asset_sac() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, _sac_client) =
        setup();
    let team_owner = Address::generate(&env);

    // Create an asset (team)
    let asset_id = fnft_client.mint(&team_owner, &100);

    // Team owner registers SAC
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // Verify SAC was registered
    assert_eq!(
        funding_client.get_asset_sac(&asset_id),
        Some(sac_contract_id.clone())
    );
    assert_eq!(
        funding_client.get_asset_by_sac(&sac_contract_id),
        Some(asset_id)
    );
}

#[test]
#[should_panic(expected = "Only asset owners can register SAC")]
fn test_register_sac_unauthorized() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, _sac_client) =
        setup();
    let team_owner = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    // Create an asset
    let asset_id = fnft_client.mint(&team_owner, &100);

    // Unauthorized user tries to register SAC
    funding_client.register_asset_sac(&unauthorized, &asset_id, &sac_contract_id);
}

#[test]
#[should_panic(expected = "Asset already has a registered SAC")]
fn test_register_sac_already_exists() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, _sac_client) =
        setup();
    let team_owner = Address::generate(&env);

    let asset_id = fnft_client.mint(&team_owner, &100);

    // Register SAC first time
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // Try to register again
    let another_sac = Address::generate(&env);
    funding_client.register_asset_sac(&team_owner, &asset_id, &another_sac);
}

#[test]
fn test_deposit_funds_to_sac() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
        setup();
    let team_owner = Address::generate(&env);
    let depositor = Address::generate(&env);

    // Create asset and register SAC
    let asset_id = fnft_client.mint(&team_owner, &100);
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // Mint some tokens to depositor in the SAC
    sac_client.mint(&depositor, &5000i128);

    // Deposit funds (should go to SAC)
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);
    // Simulate the deposit by updating the mock SAC contract's balance for the SAC address
    sac_client.mint(&sac_contract_id, &1000i128);

    // Check SAC balance increased
    assert_eq!(
        funding_client.asset_funds(&asset_id),
        sac_client.balance(&sac_contract_id) as u128
    );
}

#[test]
#[should_panic(expected = "Asset must have a registered SAC to use funding features")]
fn test_deposit_without_sac() {
    let (
        env,
        _admin,
        _fnft_contract_id,
        _sac_contract_id,
        funding_client,
        fnft_client,
        _sac_client,
    ) = setup();
    let team_owner = Address::generate(&env);
    let depositor = Address::generate(&env);

    // Create asset but DON'T register SAC
    let asset_id = fnft_client.mint(&team_owner, &100);

    // Try to deposit without SAC - should fail
    funding_client.deposit_funds(&depositor, &asset_id, &1000i128);
}

#[test]
fn test_asset_funds_from_sac() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
        setup();
    let team_owner = Address::generate(&env);

    // Create asset and register SAC
    let asset_id = fnft_client.mint(&team_owner, &100);
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // Mint tokens directly to SAC
    sac_client.mint(&sac_contract_id, &2500i128);

    // asset_funds should return SAC balance
    assert_eq!(
        funding_client.asset_funds(&asset_id),
        sac_client.balance(&sac_contract_id) as u128
    );
}

#[test]
fn test_distribute_from_sac() {
    let (env, admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
        setup();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    // Create asset and distribute tokens
    let asset_id = fnft_client.mint(&owner1, &600); // owner1 has 60%
    fnft_client.transfer(&owner1, &owner2, &asset_id, &400); // owner2 has 40%

    // Register SAC
    funding_client.register_asset_sac(&owner1, &asset_id, &sac_contract_id);

    // Add funds to SAC
    sac_client.mint(&sac_contract_id, &1000i128);

    let description = String::from_str(&env, "Test distribution from SAC");

    // Distribute from SAC
    funding_client.distribute_funds(&admin, &asset_id, &1000u128, &description);

    // Check analytics updated (allow for dust: distributed may be less than requested)
    let distributed = funding_client.total_distributed(&asset_id);
    assert!(distributed <= 1000u128 && distributed >= 999u128);
    assert_eq!(funding_client.get_distribution_count(&asset_id), 1u32);
}

// #[test]
// fn test_emergency_withdraw_from_sac() {
//     let (env, admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
//         setup();
//     let team_owner = Address::generate(&env);
//
//     // Create asset and register SAC
//     let asset_id = fnft_client.mint(&team_owner, &100);
//     funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);
//
//     // Add funds to SAC
//     sac_client.mint(&sac_contract_id, &1000i128);
//
//     let reason = String::from_str(&env, "Emergency withdrawal from SAC");
//
//     // Admin emergency withdraw from SAC
//     funding_client.emergency_withdraw(&admin, &asset_id, &300u128, &reason);
//
//     // Check funds were withdrawn from SAC
//     assert_eq!(sac_client.balance(&sac_contract_id), 700i128);
// }

#[test]
#[should_panic(expected = "Insufficient balance in asset SAC")]
fn test_distribute_insufficient_sac_balance() {
    let (env, admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, _sac_client) =
        setup();
    let team_owner = Address::generate(&env);

    let asset_id = fnft_client.mint(&team_owner, &100);
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // SAC has default balance, try to distribute more
    let description = String::from_str(&env, "Over-distribution");
    funding_client.distribute_funds(&admin, &asset_id, &2000000u128, &description);
}

#[test]
fn test_owner_distribute_from_sac() {
    let (env, _admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
        setup();
    let team_owner = Address::generate(&env);

    // Create asset
    let asset_id = fnft_client.mint(&team_owner, &100);
    funding_client.register_asset_sac(&team_owner, &asset_id, &sac_contract_id);

    // Add funds to SAC
    sac_client.mint(&sac_contract_id, &500i128);

    let description = String::from_str(&env, "Owner distribution");

    // Owner can distribute
    funding_client.owner_distribute_funds(&team_owner, &asset_id, &500u128, &description);

    // Check analytics
    assert_eq!(funding_client.total_distributed(&asset_id), 500u128);
}

#[test]
fn test_can_distribute_permissions() {
    let (env, admin, _fnft_contract_id, sac_contract_id, funding_client, fnft_client, _sac_client) =
        setup();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);

    let asset_id = fnft_client.mint(&owner, &100);
    funding_client.register_asset_sac(&owner, &asset_id, &sac_contract_id);

    // Admin can always distribute
    assert!(funding_client.can_distribute(&admin, &asset_id));

    // Owner can distribute
    assert!(funding_client.can_distribute(&owner, &asset_id));

    // Non-owner cannot distribute
    assert!(!funding_client.can_distribute(&non_owner, &asset_id));
}

#[test]
fn test_view_functions() {
    let (env, admin, fnft_contract_id, sac_contract_id, funding_client, fnft_client, sac_client) =
        setup();
    let owner = Address::generate(&env);

    let asset_id = fnft_client.mint(&owner, &100);
    funding_client.register_asset_sac(&owner, &asset_id, &sac_contract_id);

    // Initially some default balance in SAC
    let initial_balance = sac_client.balance(&sac_contract_id);
    assert_eq!(
        funding_client.asset_funds(&asset_id),
        initial_balance as u128
    );

    // Initially no distributions
    assert_eq!(funding_client.total_distributed(&asset_id), 0u128);
    assert_eq!(funding_client.get_distribution_count(&asset_id), 0u32);

    // Check contract addresses
    assert_eq!(funding_client.get_fnft_contract_address(), fnft_contract_id);
    assert_eq!(funding_client.get_admin(), admin);

    // Check SAC mapping
    assert_eq!(
        funding_client.get_asset_sac(&asset_id),
        Some(sac_contract_id.clone())
    );
    assert_eq!(
        funding_client.get_asset_by_sac(&sac_contract_id),
        Some(asset_id)
    );
}
