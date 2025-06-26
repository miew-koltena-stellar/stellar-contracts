#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

// Storage key implementation for Soroban replacing Solidity's nested mappings
// Replaces Solidity's mapping(address => mapping(uint256 => uint256)) private _balance;
// Uses keys/variables that Soroban serializes automatically
#[contracttype]
pub enum DataKey {
    // Contract core data
    Admin,

    // Asset ID counter replacing id_counter from various registry implementations
    NextAssetId,

    // Balance and supply tracking
    // Replaces mapping(address => mapping(uint256 => uint256)) private _balance;
    Balance(Address, u64), // owner -> asset_id -> balance

    // Replaces mapping(uint256 => uint256) private _totalSupply;
    AssetSupply(u64), // asset_id -> total_supply

    // Ownership tracking
    // Replaces complex tree structures from RegistryNestedTree
    // Avoids unlimited Vec growth through simple boolean flags
    AssetOwnerExists(u64, Address), // asset_id -> owner -> bool
    OwnerAssetExists(Address, u64), // owner -> asset_id -> bool
    AssetOwnerCount(u64),           // asset_id -> number_of_owners

    // Funding distributions
    // New addition - replaces complex tree queries from Solidity
    // Enables efficient iteration for future funding/voting systems
    AssetOwnersList(u64), // asset_id -> Vec<Address> (owners with balance > 0)
    OwnerAssetsList(Address), // owner -> Vec<u64> (assets owned with balance > 0)

    // Authorization system
    // Simplification of AllowancesNestedMap from Solidity
    // Maintains ERC1155 compatibility but with simpler storage
    OperatorApproval(Address, Address), // owner -> operator -> approved_for_all
    TokenAllowance(Address, Address, u64), // owner -> operator -> asset_id -> allowance

    // Metadata support
    // Replaces mapping(uint256 => string) assetURIs; from Solidity
    AssetURI(u64), // asset_id -> metadata_uri
    ContractURI,   // global contract metadata

    // Asset management
    // New functionality - tracking who created each asset
    AssetCreator(u64), // asset_id -> creator_address
}

#[contract]
pub struct FractionalizationContract;

#[contractimpl]
impl FractionalizationContract {
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
        env.events().publish((symbol_short!("init"),), (admin,));
    }

    /// Token minting replacing function mint(uint256 numTokens) public virtual noReentrancy delegateOnly returns (uint256)
    /// Simplification: Removes reentrancy guard (Soroban has built-in protections)
    /// Change: Mint to specific address instead of msg.sender
    /// Returns: asset_id instead of returning via event
    pub fn mint(env: Env, to: Address, num_tokens: u64) -> u64 {
        // Admin verification - replaces delegateOnly modifier from Solidity
        Self::require_admin_auth(env.clone());

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
        Self::add_owner_to_asset(&env, asset_id, to.clone());
        Self::add_asset_to_owner(&env, to.clone(), asset_id);

        // Emit event - similar to TransferSingle from ERC1155 but simplified
        env.events()
            .publish((symbol_short!("mint"),), (to.clone(), asset_id, num_tokens));

        asset_id
    }

    /// Multiple recipient minting from Solidity's dynamicMint() - simplified version
    /// Allows minting to multiple recipients of an existing asset
    /// Validation: asset_id must exist (cannot create new assets)
    pub fn mint_to(env: Env, asset_id: u64, recipients: Vec<Address>, amounts: Vec<u64>) {
        // Admin verification
        Self::require_admin_auth(env.clone());

        // Asset ID validation - prevents creation of new assets via mint_to
        if asset_id == 0 {
            panic!("Asset ID cannot be 0 - use mint() to create new assets");
        }

        // Verify asset exists - new functionality vs Solidity
        if !Self::asset_exists(env.clone(), asset_id) {
            panic!("Asset does not exist");
        }

        // Input validations - similar to Solidity
        if recipients.len() != amounts.len() {
            panic!("Recipients and amounts length mismatch");
        }

        if recipients.len() == 0 {
            panic!("No recipients specified");
        }

        let mut total_minted = 0u64;
        let mut owner_count = Self::get_asset_owner_count(env.clone(), asset_id);

        // Process each recipient
        for i in 0..recipients.len() {
            let recipient = recipients.get(i).unwrap();
            let amount = amounts.get(i).unwrap();

            if amount == 0 {
                panic!("Cannot mint 0 tokens");
            }

            // Update balance (may add to existing balance)
            let current_balance = Self::balance_of(env.clone(), recipient.clone(), asset_id);
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
                Self::add_owner_to_asset(&env, asset_id, recipient.clone());
                Self::add_asset_to_owner(&env, recipient.clone(), asset_id);
            }

            total_minted += amount;

            // Emit event per recipient
            env.events().publish(
                (symbol_short!("mint_to"),),
                (recipient.clone(), asset_id, amount),
            );
        }

        // Update total supply
        let current_supply = Self::asset_supply(env.clone(), asset_id);
        env.storage().persistent().set(
            &DataKey::AssetSupply(asset_id),
            &(current_supply + total_minted),
        );

        // Update owner count
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnerCount(asset_id), &owner_count);
    }

    /// REFACTOR: function balanceOf(address account, uint256 id) external view returns (uint256)
    /// Direct implementation - without the overhead of Solidity trees
    pub fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64 {
        // Direct storage access - much simpler than RegistryNestedTree trees
        env.storage()
            .persistent()
            .get(&DataKey::Balance(owner, asset_id))
            .unwrap_or(0) // Return 0 if doesn't exist (like in Solidity)
    }

    /// REFACTOR: function balanceOfBatch() from ERC1155
    /// Implementation maintained for compatibility
    pub fn balance_of_batch(env: Env, owners: Vec<Address>, asset_ids: Vec<u64>) -> Vec<u64> {
        // Validation similar to Solidity
        if owners.len() != asset_ids.len() {
            panic!("Owners and asset_ids length mismatch");
        }

        let mut balances = Vec::new(&env);
        // Iterate and get individual balances
        for i in 0..owners.len() {
            let owner = owners.get(i).unwrap();
            let asset_id = asset_ids.get(i).unwrap();
            let balance = Self::balance_of(env.clone(), owner, asset_id);
            balances.push_back(balance);
        }

        balances
    }

    /// NEW FUNCTION: Simple transfer (owner transfers their own tokens)
    /// Simplification vs safeTransferFrom from Solidity
    pub fn transfer(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
        // Mandatory authorization verification
        from.require_auth();
        // Delegate to internal logic
        Self::transfer_internal(env, from, to, asset_id, amount);
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
                Self::is_approved_for_all(env.clone(), from.clone(), operator.clone());

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
        Self::transfer_internal(env, from, to, asset_id, amount);
    }

    /// REFACTOR: function _transferFrom() from various Solidity registries
    /// Simplification: Unified logic without overhead of tree structures
    /// Addition: Automatic maintenance of owner lists
    fn transfer_internal(env: Env, from: Address, to: Address, asset_id: u64, amount: u64) {
        // Basic validations
        if amount == 0 {
            panic!("Cannot transfer 0 tokens");
        }

        if from == to {
            panic!("Cannot transfer to self");
        }

        // Get current balances - direct access vs complex queries from Solidity
        let from_balance = Self::balance_of(env.clone(), from.clone(), asset_id);
        let to_balance = Self::balance_of(env.clone(), to.clone(), asset_id);

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
            let owner_count = Self::get_asset_owner_count(env.clone(), asset_id);
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count + 1));

            // Add to lists for efficient queries
            Self::add_owner_to_asset(&env, asset_id, to.clone());
            Self::add_asset_to_owner(&env, to.clone(), asset_id);
        }

        // === OWNERSHIP CLEANUP ===
        // Remove sender from lists if balance became 0
        if new_from_balance == 0 {
            Self::remove_owner_from_asset(&env, asset_id, from.clone());
            Self::remove_asset_from_owner(&env, from.clone(), asset_id);

            // Decrement owner count
            let owner_count = Self::get_asset_owner_count(env.clone(), asset_id);
            if owner_count > 0 {
                env.storage()
                    .persistent()
                    .set(&DataKey::AssetOwnerCount(asset_id), &(owner_count - 1));
            }
        }

        // Emit transfer event
        env.events().publish(
            (symbol_short!("transfer"),),
            (from.clone(), to.clone(), asset_id, amount),
        );
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
            Self::transfer_from(
                env.clone(),
                operator.clone(),
                from.clone(),
                to.clone(),
                asset_id,
                amount,
            );
        }
    }

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
        env.events()
            .publish((symbol_short!("approval"),), (owner, operator, approved));
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
        env.events().publish(
            (symbol_short!("approve"),),
            (owner, operator, asset_id, amount),
        );
    }

    /// REFACTOR: function getAllowance() from AllowancesNestedMap
    /// Renamed to allowance for standard compatibility
    pub fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::TokenAllowance(owner, operator, asset_id))
            .unwrap_or(0)
    }

    /// REFACTOR: function assetSupply(uint256 assetId) external view returns (uint256)
    /// Direct implementation vs complex calculations from registries
    pub fn asset_supply(env: Env, asset_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetSupply(asset_id))
            .unwrap_or(0)
    }

    /// NEW FUNCTION: Owner count per asset
    /// Replaces expensive queries from Solidity tree structures
    pub fn get_asset_owner_count(env: Env, asset_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnerCount(asset_id))
            .unwrap_or(0)
    }

    /// NEW FUNCTION: Fast ownership verification
    /// Replaces complex iterations from RegistryNestedTree
    pub fn owns_asset(env: Env, owner: Address, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnerExists(asset_id, owner))
            .unwrap_or(false)
    }

    /// NEW FUNCTION: Check if owner has any asset
    /// Helper functionality for queries
    pub fn has_assets(env: Env, owner: Address, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerAssetExists(owner, asset_id))
            .unwrap_or(false)
    }

    /// REFACTOR: function assetOwners(uint256 tokenId) external view returns (address[] memory)
    /// Simplification: Direct list vs complex tree iteration
    /// Optimization: Kept in sync automatically vs on-demand calculation
    pub fn asset_owners(env: Env, asset_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(&env))
    }

    /// REFACTOR: function addressAssets(address owner) external view returns (uint256[] memory)
    /// Simplification similar to the function above
    pub fn owner_assets(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// NEW FUNCTION: Next asset ID to be assigned
    /// Helper functionality for front-ends
    pub fn next_asset_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextAssetId)
            .unwrap_or(1)
    }

    /// NEW FUNCTION: Asset existence verification
    /// Replaces complex verifications from Solidity
    pub fn asset_exists(env: Env, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::AssetSupply(asset_id))
    }

    /// REFACTOR: function setUri(uint256 _tokenId, string calldata uri_) public
    /// Addition: Authorization verification (admin or creator)
    /// Change: Explicit caller vs implicit msg.sender
    pub fn set_asset_uri(env: Env, caller: Address, asset_id: u64, uri: String) {
        caller.require_auth();

        // Check if asset exists
        if !Self::asset_exists(env.clone(), asset_id) {
            panic!("Asset does not exist");
        }

        // Authorization verification - only admin or asset creator
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        let creator: Address = env
            .storage()
            .persistent()
            .get(&DataKey::AssetCreator(asset_id))
            .unwrap();

        if caller != admin && caller != creator {
            panic!("Not authorized to set URI");
        }

        // Store URI
        env.storage()
            .persistent()
            .set(&DataKey::AssetURI(asset_id), &uri);

        // Emit event
        env.events()
            .publish((symbol_short!("uri"),), (asset_id, uri));
    }

    /// REFACTOR: function uri(uint256 _tokenId) public view returns (string memory)
    /// Direct implementation
    pub fn asset_uri(env: Env, asset_id: u64) -> Option<String> {
        env.storage().persistent().get(&DataKey::AssetURI(asset_id))
    }

    /// NEW FUNCTION: Contract-level URI
    /// Additional functionality for global metadata
    pub fn set_contract_uri(env: Env, caller: Address, uri: String) {
        Self::require_admin_auth(env.clone());
        caller.require_auth();

        env.storage().persistent().set(&DataKey::ContractURI, &uri);
    }

    /// NEW FUNCTION: Get contract URI
    pub fn contract_uri(env: Env) -> Option<String> {
        env.storage().persistent().get(&DataKey::ContractURI)
    }

    /// REFACTOR: function getAdmin() public view returns (address)
    /// Direct implementation
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// NEW FUNCTION: Get asset creator
    /// Additional tracking vs original Solidity
    pub fn get_asset_creator(env: Env, asset_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetCreator(asset_id))
    }

    /// NEW FUNCTION: Admin role transfer
    /// Basic governance functionality
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        Self::require_admin_auth(env.clone());
        current_admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events()
            .publish((symbol_short!("admin"),), (current_admin, new_admin));
    }

    /// REFACTOR: modifier onlyAdmin from Solidity
    /// Converted to helper function
    fn require_admin_auth(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
    }

    // === INTERNAL LIST MANAGEMENT FUNCTIONS ===
    // NEW ADDITION: Replaces complex tree structures from RegistryNestedTree
    // Enables efficient queries for future funding and voting systems

    /// Add owner to asset's owner list
    /// Keeps list updated for fast queries
    fn add_owner_to_asset(env: &Env, asset_id: u64, owner: Address) {
        let mut owners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(env));

        // Check if owner is already in list (avoid duplicates)
        let mut found = false;
        for i in 0..owners.len() {
            if owners.get(i).unwrap() == owner {
                found = true;
                break;
            }
        }

        // Add only if doesn't exist
        if !found {
            owners.push_back(owner.clone());
            env.storage()
                .persistent()
                .set(&DataKey::AssetOwnersList(asset_id), &owners);
        }
    }

    /// Remove owner from list when balance = 0
    /// Automatic maintenance vs manual cleanup from Solidity
    fn remove_owner_from_asset(env: &Env, asset_id: u64, owner: Address) {
        let owners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetOwnersList(asset_id))
            .unwrap_or(Vec::new(env));

        // Filter out owner to remove
        let mut new_owners = Vec::new(env);
        for i in 0..owners.len() {
            let current_owner = owners.get(i).unwrap();
            if current_owner != owner {
                new_owners.push_back(current_owner);
            }
        }

        // Update list
        env.storage()
            .persistent()
            .set(&DataKey::AssetOwnersList(asset_id), &new_owners);
    }

    /// Add asset to owner's asset list
    /// Maintains list for fast queries- similar to bidirectional mapping in Solidity
    fn add_asset_to_owner(env: &Env, owner: Address, asset_id: u64) {
        let mut assets: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner.clone()))
            .unwrap_or(Vec::new(env));

        // verify duplicates
        let mut found = false;
        for i in 0..assets.len() {
            if assets.get(i).unwrap() == asset_id {
                found = true;
                break;
            }
        }

        // add new
        if !found {
            assets.push_back(asset_id);
            env.storage()
                .persistent()
                .set(&DataKey::OwnerAssetsList(owner), &assets);
        }
    }

    /// Auto Cleanup: Remove asset from owner list when balance = 0
    fn remove_asset_from_owner(env: &Env, owner: Address, asset_id: u64) {
        let assets: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerAssetsList(owner.clone()))
            .unwrap_or(Vec::new(env));

        // filter and remove
        let mut new_assets = Vec::new(env);
        for i in 0..assets.len() {
            let current_asset = assets.get(i).unwrap();
            if current_asset != asset_id {
                new_assets.push_back(current_asset);
            }
        }

        // update list
        env.storage()
            .persistent()
            .set(&DataKey::OwnerAssetsList(owner), &new_assets);
    }
}

mod test;
