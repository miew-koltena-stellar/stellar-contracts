use soroban_sdk::{token::TokenClient, Address, Env};
use crate::storage::DataKey;
use crate::interfaces::FNFTClient;
use crate::events;
use crate::methods::utils;

/// Step 2: Buyer finishes the transaction (completes the trade)
pub fn finish_transaction(env: Env, buyer: Address, seller: Address, asset_id: u64) {
    buyer.require_auth();

    // Verify sale proposal validity and existence
    let proposal = utils::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);
    if !proposal.is_active {
        panic!("Sale proposal is not active");
    }
    if proposal.buyer != buyer {
        panic!("Not authorized buyer for this sale");
    }
    if env.ledger().timestamp() > proposal.expires_at {
        panic!("Sale proposal has expired");
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    let trading_contract_id = env.current_contract_address();

    // Verify seller token balance sufficiency
    let seller_balance = fnft_client.balance_of(&proposal.seller, &proposal.asset_id);
    if seller_balance < proposal.token_amount {
        panic!("Seller has insufficient token balance");
    }

    // Verify buyer funds sufficiency
    let xlm_contract_address = utils::get_xlm_contract_address(env.clone());
    let xlm_client = TokenClient::new(&env, &xlm_contract_address);
    let buyer_xlm_balance = xlm_client.balance(&buyer);
    if buyer_xlm_balance < proposal.price as i128 {
        panic!("Buyer has insufficient XLM funds");
    }

    // Verify allowance (seller should have approved tokens to trading contract)
    let allowance =
        fnft_client.allowance(&proposal.seller, &trading_contract_id, &proposal.asset_id);
    if allowance < proposal.token_amount {
        panic!("Insufficient allowance for token transfer");
    }

    // Atomic transaction block - All or nothing
    // Transfer tokens seller -> buyer (trading contract executes as escrow)
    fnft_client.transfer_from(
        &trading_contract_id,
        &proposal.seller,
        &proposal.buyer,
        &proposal.asset_id,
        &proposal.token_amount,
    );

    // Transfer payment buyer -> seller
    if proposal.price > i128::MAX as u128 {
        panic!("Proposal price exceeds maximum allowable value for i128");
    }
    xlm_client.transfer(&buyer, &seller, &(proposal.price as i128));

    // Reentrancy protection - Immediately clean up state
    // Remove sale proposal (prevents reentrancy)
    env.storage().persistent().remove(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    ));
    utils::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
    utils::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    // Record successful trade completion
    let trade_id = utils::record_trade_history(&env, &proposal);
    utils::add_to_asset_trades(&env, asset_id, trade_id);

    // Emit trade completion event
    events::emit_trade_event(&env, &proposal, trade_id);
}
