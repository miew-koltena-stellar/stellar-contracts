use soroban_sdk::{symbol_short, Address, Env, String};

/// Contract initialization event
pub fn emit_init(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("init"),), (admin,));
}

/// Token minting event
pub fn emit_mint(env: &Env, to: Address, asset_id: u64, num_tokens: u64) {
    env.events()
        .publish((symbol_short!("mint"),), (to, asset_id, num_tokens));
}

/// Token minting to specific recipients event
pub fn emit_mint_to(env: &Env, recipient: Address, asset_id: u64, amount: u64) {
    env.events()
        .publish((symbol_short!("mint_to"),), (recipient, asset_id, amount));
}

/// Transfer event
pub fn emit_transfer(env: &Env, from: Address, to: Address, asset_id: u64, amount: u64) {
    env.events()
        .publish((symbol_short!("transfer"),), (from, to, asset_id, amount));
}

/// Approval for all event
pub fn emit_approval_for_all(env: &Env, owner: Address, operator: Address, approved: bool) {
    env.events()
        .publish((symbol_short!("approval"),), (owner, operator, approved));
}

/// Token-specific approval event
pub fn emit_approve(env: &Env, owner: Address, operator: Address, asset_id: u64, amount: u64) {
    env.events().publish(
        (symbol_short!("approve"),),
        (owner, operator, asset_id, amount),
    );
}

/// Asset URI update event
pub fn emit_uri_update(env: &Env, asset_id: u64, uri: String) {
    env.events()
        .publish((symbol_short!("uri"),), (asset_id, uri));
}

/// Admin transfer event
pub fn emit_admin_transfer(env: &Env, current_admin: Address, new_admin: Address) {
    env.events()
        .publish((symbol_short!("admin"),), (current_admin, new_admin));
}
