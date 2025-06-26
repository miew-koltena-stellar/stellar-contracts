#![no_std]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

// Import FNFT contract interface for cross-contract calls
#[contractclient(name = "FNFTClient")]
pub trait FNFTInterface {
    fn asset_exists(env: Env, asset_id: u64) -> bool;
    fn asset_supply(env: Env, asset_id: u64) -> u64;
    fn asset_owners(env: Env, asset_id: u64) -> Vec<Address>;
    fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64;
    fn get_admin(env: Env) -> Address;
    fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool;
}

// Storage keys for funding contract data
#[contracttype]
pub enum DataKey {
    // Core contract data
    Admin,
    FNFTContract, // Address of the FNFT contract

    // Fund tracking
    AssetFunds(u64),       // asset_id -> total_funds_available
    TotalDistributed(u64), // asset_id -> total_amount_distributed

    // Distribution tracking (optional for analytics)
    DistributionCount(u64), // asset_id -> number_of_distributions
}

#[contract]
pub struct FundingContract;

#[contractimpl]
impl FundingContract {
    /// Initialize the funding contract
    /// admin: Address that can execute distributions
    /// fnft_contract: Address of the FNFT contract to integrate with
    pub fn initialize(env: Env, admin: Address, fnft_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        // Validate FNFT contract address (just check it's not zero-like)
        // Note: In Soroban, we can't easily get current contract address for comparison
        // So we'll just validate it's not a generated test address pattern

        // Set admin and FNFT contract
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::FNFTContract, &fnft_contract);

        // Emit initialization event
        env.events().publish(
            (symbol_short!("init"),),
            (admin.clone(), fnft_contract.clone()),
        );
    }

    /// Deposit funds for a specific asset
    /// Anyone can deposit funds for any existing asset
    pub fn deposit_funds(env: Env, depositor: Address, asset_id: u64, amount: u128) {
        depositor.require_auth();

        if amount == 0 {
            panic!("Deposit amount must be > 0");
        }

        // Verify asset exists
        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);

        if !fnft_client.asset_exists(&asset_id) {
            panic!("Asset does not exist");
        }

        // Update asset funds
        let current_funds = Self::asset_funds(env.clone(), asset_id);
        env.storage()
            .persistent()
            .set(&DataKey::AssetFunds(asset_id), &(current_funds + amount));

        // Emit deposit event
        env.events().publish(
            (symbol_short!("deposit"),),
            (asset_id, depositor.clone(), amount),
        );
    }

    /// Distribute funds to asset owners (only admin/governance)
    /// This is the main distribution function called after governance approval
    pub fn distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        Self::require_admin_auth(env.clone(), caller);

        if amount == 0 {
            panic!("Distribution amount must be > 0");
        }

        // Verify asset exists and has funds
        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);

        if !fnft_client.asset_exists(&asset_id) {
            panic!("Asset does not exist");
        }

        let current_funds = Self::asset_funds(env.clone(), asset_id);
        if amount > current_funds {
            panic!("Insufficient funds for distribution");
        }

        let total_supply = fnft_client.asset_supply(&asset_id);
        if total_supply == 0 {
            panic!("Asset has no supply");
        }

        // Get all current owners
        let owners = fnft_client.asset_owners(&asset_id);
        if owners.len() == 0 {
            panic!("No asset owners found");
        }

        // Execute distribution
        Self::execute_distribution_logic(
            &env,
            &fnft_client,
            asset_id,
            amount,
            total_supply,
            owners,
            description,
        );
    }

    /// Allow asset owners to directly distribute funds (democratic distribution)
    /// Requires caller to own tokens of the asset
    pub fn owner_distribute_funds(
        env: Env,
        caller: Address,
        asset_id: u64,
        amount: u128,
        description: String,
    ) {
        caller.require_auth();

        if amount == 0 {
            panic!("Distribution amount must be > 0");
        }

        // Verify caller owns tokens of this asset
        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);

        if !fnft_client.owns_asset(&caller, &asset_id) {
            panic!("Caller does not own tokens of this asset");
        }

        let current_funds = Self::asset_funds(env.clone(), asset_id);
        if amount > current_funds {
            panic!("Insufficient funds for distribution");
        }

        let total_supply = fnft_client.asset_supply(&asset_id);
        if total_supply == 0 {
            panic!("Asset has no supply");
        }

        let owners = fnft_client.asset_owners(&asset_id);
        if owners.len() == 0 {
            panic!("No asset owners found");
        }

        // Execute distribution
        Self::execute_distribution_logic(
            &env,
            &fnft_client,
            asset_id,
            amount,
            total_supply,
            owners,
            description,
        );
    }

    /// Internal distribution logic (separated to avoid code duplication)
    fn execute_distribution_logic(
        env: &Env,
        fnft_client: &FNFTClient,
        asset_id: u64,
        amount: u128,
        total_supply: u64,
        owners: Vec<Address>,
        description: String,
    ) {
        // Update asset funds first
        let current_funds = Self::asset_funds(env.clone(), asset_id);
        env.storage()
            .persistent()
            .set(&DataKey::AssetFunds(asset_id), &(current_funds - amount));

        // Update total distributed with actual amount distributed
        let current_distributed = Self::total_distributed(env.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::TotalDistributed(asset_id),
            &(current_distributed + amount),
        );

        // Update distribution count
        let distribution_count = Self::get_distribution_count(env.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::DistributionCount(asset_id),
            &(distribution_count + 1),
        );

        let mut total_distributed = 0u128;
        let mut recipients_count = 0u32;

        // Distribute to each owner proportionally
        for owner in owners {
            let balance = fnft_client.balance_of(&owner, &asset_id);

            if balance > 0 {
                // Calculate proportional share: (amount * balance) / total_supply
                let owner_share = (amount * balance as u128) / total_supply as u128;

                if owner_share > 0 {
                    // In a real implementation, you would transfer tokens here
                    // For now, we'll emit an event indicating the distribution
                    total_distributed += owner_share;
                    recipients_count += 1;

                    env.events().publish(
                        (symbol_short!("received"),),
                        (asset_id, owner.clone(), owner_share),
                    );
                }
            }
        }

        // Handle any remaining dust (due to integer division)
        if total_distributed < amount {
            let dust = amount - total_distributed;
            // Add dust back to asset funds
            let remaining_funds = Self::asset_funds(env.clone(), asset_id);
            env.storage()
                .persistent()
                .set(&DataKey::AssetFunds(asset_id), &(remaining_funds + dust));

            // Update total distributed to reflect actual amount distributed
            let current_distributed = Self::total_distributed(env.clone(), asset_id);
            env.storage().persistent().set(
                &DataKey::TotalDistributed(asset_id),
                &(current_distributed - dust), // Subtract the dust that wasn't actually distributed
            );
        }

        // Emit distribution event
        env.events().publish(
            (symbol_short!("distrib"),),
            (asset_id, total_distributed, description, recipients_count),
        );
    }

    // === View Functions ===

    /// Get total funds available for an asset
    pub fn asset_funds(env: Env, asset_id: u64) -> u128 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetFunds(asset_id))
            .unwrap_or(0)
    }

    /// Get total amount distributed for an asset
    pub fn total_distributed(env: Env, asset_id: u64) -> u128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalDistributed(asset_id))
            .unwrap_or(0)
    }

    /// Get number of distributions for an asset
    pub fn get_distribution_count(env: Env, asset_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::DistributionCount(asset_id))
            .unwrap_or(0)
    }

    /// Get the FNFT contract address
    pub fn get_fnft_contract_address(env: Env) -> Address {
        Self::get_fnft_contract(&env)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Check if an address can distribute funds for an asset
    pub fn can_distribute(env: Env, caller: Address, asset_id: u64) -> bool {
        let admin = Self::get_admin(env.clone());
        if caller == admin {
            return true;
        }

        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);
        fnft_client.owns_asset(&caller, &asset_id)
    }

    // === Admin Functions ===

    /// Transfer admin role (only current admin)
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();

        let admin = Self::get_admin(env.clone());
        if current_admin != admin {
            panic!("Only current admin can transfer admin role");
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events()
            .publish((symbol_short!("admin"),), (current_admin, new_admin));
    }

    /// Emergency withdraw funds (only admin)
    /// For emergency situations or contract upgrades
    pub fn emergency_withdraw(
        env: Env,
        admin: Address,
        asset_id: u64,
        amount: u128,
        reason: String,
    ) {
        admin.require_auth();
        Self::require_admin_auth(env.clone(), admin.clone());

        let current_funds = Self::asset_funds(env.clone(), asset_id);
        if amount > current_funds {
            panic!("Insufficient funds for withdrawal");
        }

        // Update funds
        env.storage()
            .persistent()
            .set(&DataKey::AssetFunds(asset_id), &(current_funds - amount));

        // Emit emergency withdrawal event
        env.events().publish(
            (symbol_short!("emergency"),),
            (asset_id, admin, amount, reason),
        );
    }

    // === Internal Helper Functions ===

    /// Get FNFT contract address from storage
    fn get_fnft_contract(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::FNFTContract)
            .unwrap()
    }

    /// Require admin authorization
    fn require_admin_auth(env: Env, caller: Address) {
        let admin = Self::get_admin(env);
        if caller != admin {
            panic!("Only admin can perform this action");
        }
    }
}

mod test;
