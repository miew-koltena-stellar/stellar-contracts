use soroban_sdk::{symbol_short, Address, Env, String};

/// Contract initialization
pub fn emit_init(env: &Env, admin: Address, fnft_contract: Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin, fnft_contract));
}

/// SAC registration
pub fn emit_sac_registered(env: &Env, asset_id: u64, sac_address: Address) {
    env.events()
        .publish((symbol_short!("sac_reg"), asset_id, sac_address), ());
}

/// Funds deposit event
pub fn emit_deposit(env: &Env, asset_id: u64, depositor: Address, amount: i128) {
    env.events()
        .publish((symbol_short!("deposit"),), (asset_id, depositor, amount));
}

/// Distribution execution (from SAC)
pub fn emit_distribution(
    env: &Env,
    asset_id: u64,
    amount: u128,
    description: String,
    recipients: u32,
) {
    env.events().publish(
        (
            symbol_short!("distrib"),
            asset_id,
            amount,
            description,
            recipients,
        ),
        (),
    );
}

/// Individual payment received
pub fn emit_received(env: &Env, asset_id: u64, recipient: Address, amount: u128) {
    env.events()
        .publish((symbol_short!("received"), asset_id, recipient, amount), ());
}

/// Admin role transfer
pub fn emit_admin_transfer(env: &Env, old_admin: Address, new_admin: Address) {
    env.events()
        .publish((symbol_short!("admin"), old_admin, new_admin), ());
}

/// Emergency withdrawal event (from SAC)
pub fn emit_emergency(env: &Env, asset_id: u64, admin: Address, amount: u128, reason: String) {
    env.events().publish(
        (symbol_short!("emergency"), asset_id, admin, amount, reason),
        (),
    );
}
