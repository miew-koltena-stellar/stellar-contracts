use soroban_sdk::{contracttype, Address};

/// Storage key implementation for Soroban replacing Solidity's nested mappings
/// Replaces Solidity's mapping(address => mapping(uint256 => uint256)) private _balance;
/// Uses keys/variables that Soroban serializes automatically
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
    AssetOwnerExists(u64, Address),   // asset_id -> owner -> bool
    OwnerAssetExists(Address, u64),   // owner -> asset_id -> bool
    AssetOwnerCount(u64),             // asset_id -> number_of_owners
    AssetOwnersPage(u64, u32),        // asset_id -> page_num -> Vec<Address>
    AssetOwnerPageCount(u64),         // asset_id -> number_of_pages
    AssetLastActivePage(u64),         // Hint: last page with space
    AssetOwnerLocation(u64, Address), // Fast removal: owner -> page_num

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
