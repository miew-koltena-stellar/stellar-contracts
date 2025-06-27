use soroban_sdk::{symbol_short, Address, Env};
use crate::storage::SaleProposal;

/// Emit contract initialization event
pub fn emit_init_event(env: &Env, admin: &Address, fnft_contract: &Address, xlm_contract: &Address) {
    env.events().publish(
        (symbol_short!("init"),),
        (admin.clone(), fnft_contract.clone(), xlm_contract.clone()),
    );
}

/// Emit sale proposal creation event
pub fn emit_sale_event(env: &Env, proposal: &SaleProposal) {
    env.events().publish(
        (symbol_short!("sale"),),
        (
            proposal.seller.clone(),
            proposal.buyer.clone(),
            proposal.asset_id,
            proposal.token_amount,
            proposal.price,
        ),
    );
}

/// Emit trade completion event
pub fn emit_trade_event(env: &Env, proposal: &SaleProposal, trade_id: u32) {
    env.events().publish(
        (symbol_short!("trade"),),
        (
            proposal.seller.clone(),
            proposal.buyer.clone(),
            proposal.asset_id,
            proposal.token_amount,
            proposal.price,
            trade_id,
        ),
    );
}

/// Emit sale withdrawal event
pub fn emit_withdraw_event(env: &Env, seller: &Address, buyer: &Address, asset_id: u64) {
    env.events().publish(
        (symbol_short!("withdraw"),),
        (seller.clone(), buyer.clone(), asset_id),
    );
}

/// Emit emergency allowance reset event
pub fn emit_emergency_reset_event(env: &Env, seller: &Address, asset_id: u64) {
    env.events().publish(
        (symbol_short!("reset"),),
        (seller.clone(), asset_id),
    );
}
