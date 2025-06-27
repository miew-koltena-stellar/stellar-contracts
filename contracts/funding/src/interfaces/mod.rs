use soroban_sdk::{contractclient, Address, Env, Vec};

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

// Stellar Asset Contract interface for XLM transfers
#[contractclient(name = "TokenClient")]
pub trait TokenInterface {
    fn transfer(env: Env, from: Address, to: Address, amount: i128);
    fn balance(env: Env, id: Address) -> i128;
}
