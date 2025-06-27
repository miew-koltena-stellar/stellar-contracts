use soroban_sdk::{contractclient, Address, Env};

// FNFT contract interface for cross-contract calls
#[contractclient(name = "FNFTClient")]
pub trait FNFTInterface {
    fn asset_exists(env: Env, asset_id: u64) -> bool;
    fn balance_of(env: Env, owner: Address, asset_id: u64) -> u64;
    fn transfer_from(
        env: Env,
        operator: Address,
        from: Address,
        to: Address,
        asset_id: u64,
        amount: u64,
    );
    fn approve(env: Env, owner: Address, operator: Address, asset_id: u64, amount: u64);
    fn allowance(env: Env, owner: Address, operator: Address, asset_id: u64) -> u64;
}
