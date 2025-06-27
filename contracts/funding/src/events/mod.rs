use soroban_sdk::{symbol_short, Address, Env, String};

/// Contract initialization event
pub fn emit_init(env: &Env, admin: Address, fnft_contract: Address, xlm_token: Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin, fnft_contract, xlm_token));
}

/// Funds deposit event
pub fn emit_deposit(env: &Env, asset_id: u64, depositor: Address, amount: i128) {
    env.events()
        .publish((symbol_short!("deposit"),), (asset_id, depositor, amount));
}

/// Individual recipient distribution event
pub fn emit_received(env: &Env, asset_id: u64, owner: Address, owner_share: u128) {
    env.events()
        .publish((symbol_short!("received"),), (asset_id, owner, owner_share));
}

/// Distribution completion event
pub fn emit_distribution(
    env: &Env,
    asset_id: u64,
    total_distributed: u128,
    description: String,
    recipients_count: u32,
) {
    env.events().publish(
        (symbol_short!("distrib"),),
        (asset_id, total_distributed, description, recipients_count),
    );
}

/// Admin transfer event
pub fn emit_admin_transfer(env: &Env, current_admin: Address, new_admin: Address) {
    env.events()
        .publish((symbol_short!("admin"),), (current_admin, new_admin));
}

/// Emergency withdrawal event
pub fn emit_emergency(env: &Env, asset_id: u64, admin: Address, amount: u128, reason: String) {
    env.events().publish(
        (symbol_short!("emergency"),),
        (asset_id, admin, amount, reason),
    );
}
