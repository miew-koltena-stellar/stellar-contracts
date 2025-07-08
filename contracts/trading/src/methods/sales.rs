use crate::events;
use crate::interfaces::FNFTClient;
use crate::methods::utils;
use crate::storage::{DataKey, SaleProposal, MAX_SALE_DURATION, MIN_SALE_DURATION};
#[allow(unused_imports)]
use soroban_sdk::IntoVal;
use soroban_sdk::{symbol_short, token::TokenClient, Address, Env};

/// Seller confirms sale: grants allowance to the trading contract and creates the proposal
pub fn confirm_sale(
    env: Env,
    seller: Address,
    buyer: Address,
    asset_id: u64,
    token_amount: u64,
    price: u128,
    duration_seconds: u64,
) {
    seller.require_auth();

    if token_amount == 0 {
        panic!("Token amount must be > 0");
    }
    if price == 0 {
        panic!("Price must be > 0");
    }
    if seller == buyer {
        panic!("Cannot trade with yourself");
    }
    if duration_seconds < MIN_SALE_DURATION || duration_seconds > MAX_SALE_DURATION {
        panic!("Duration must be between 1 hour and 1 week");
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);

    if !fnft_client.asset_exists(&asset_id) {
        panic!("Asset does not exist");
    }

    let seller_balance = fnft_client.balance_of(&seller, &asset_id);
    if seller_balance < token_amount {
        panic!("Insufficient balance");
    }

    // Check if sale proposal already exists
    if env.storage().persistent().has(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    )) {
        panic!("Sale proposal already exists - withdraw first");
    }

    // Grant allowance to trading contract for secure trade
    let trading_contract_id = env.current_contract_address();
    let current_allowance = fnft_client.allowance(&seller, &trading_contract_id, &asset_id);
    let new_total_allowance = current_allowance + token_amount;

    // Require authorization for allowance modification in production
    #[cfg(not(test))]
    seller.require_auth_for_args(
        (
            fnft_contract.clone(),
            symbol_short!("approve"),
            (
                &seller,
                &trading_contract_id,
                &asset_id,
                &new_total_allowance,
            ),
        )
            .into_val(&env),
    );

    fnft_client.approve(
        &seller,
        &trading_contract_id,
        &asset_id,
        &new_total_allowance,
    );

    let proposal = SaleProposal {
        seller: seller.clone(),
        buyer: buyer.clone(),
        asset_id,
        token_amount,
        price,
        timestamp: env.ledger().timestamp(),
        is_active: true,
        expires_at: env.ledger().timestamp() + duration_seconds,
    };

    env.storage().persistent().set(
        &DataKey::SaleProposal(seller.clone(), buyer.clone(), asset_id),
        &proposal,
    );

    utils::add_to_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);

    utils::add_to_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    events::emit_sale_event(&env, &proposal);
}

/// Buyer finishes transaction: completes the trade
pub fn finish_transaction(
    env: Env,
    buyer: Address,
    seller: Address,
    asset_id: u64,
    expected_token_amount: u64,
    expected_price: u128,
) {
    buyer.require_auth();

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

    // Validate buyer's expected terms to prevent bait-and-switch attacks
    if proposal.token_amount != expected_token_amount {
        panic!(
            "Token amount mismatch - expected {}, found {}",
            expected_token_amount, proposal.token_amount
        );
    }
    if proposal.price != expected_price {
        panic!(
            "Price mismatch - expected {}, found {}",
            expected_price, proposal.price
        );
    }

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    let trading_contract_id = env.current_contract_address();

    let seller_balance = fnft_client.balance_of(&proposal.seller, &proposal.asset_id);
    if seller_balance < proposal.token_amount {
        panic!("Seller has insufficient token balance");
    }

    let xlm_contract_address = utils::get_xlm_contract_address(env.clone());
    let xlm_client = TokenClient::new(&env, &xlm_contract_address);
    let buyer_xlm_balance = xlm_client.balance(&buyer);
    if buyer_xlm_balance < proposal.price as i128 {
        panic!("Buyer has insufficient XLM funds");
    }

    let allowance =
        fnft_client.allowance(&proposal.seller, &trading_contract_id, &proposal.asset_id);
    if allowance < proposal.token_amount {
        panic!("Insufficient allowance for token transfer");
    }

    // Atomic transaction: All or nothing
    fnft_client.transfer_from(
        &trading_contract_id,
        &proposal.seller,
        &proposal.buyer,
        &proposal.asset_id,
        &proposal.token_amount,
    );

    if proposal.price > i128::MAX as u128 {
        panic!("Proposal price exceeds maximum allowable value for i128");
    }
    xlm_client.transfer(&buyer, &seller, &(proposal.price as i128));

    // Reentrancy protection - Immediately clean up state
    env.storage().persistent().remove(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    ));
    utils::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
    utils::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    let trade_id = utils::record_trade_history(&env, &proposal);
    utils::add_to_asset_trades(&env, asset_id, trade_id);

    events::emit_trade_event(&env, &proposal, trade_id);
}

pub fn cleanup_expired_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
    let proposal = utils::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);

    if env.ledger().timestamp() <= proposal.expires_at {
        panic!("Sale has not expired yet");
    }

    env.storage().persistent().remove(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    ));
    utils::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
    utils::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    env.events().publish(
        (symbol_short!("expired"),),
        (seller, buyer, asset_id, proposal.token_amount),
    );
}

/// Seller withdraws sale proposal: cancels the trade
pub fn withdraw_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
    seller.require_auth();

    let proposal = utils::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);

    if proposal.seller != seller {
        panic!("Only the seller can withdraw this proposal");
    }

    if !proposal.is_active {
        panic!("Sale proposal is not active");
    }

    // Critical security: Reduce allowance by this proposal's amount
    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    let trading_contract_id = env.current_contract_address();

    let current_allowance = fnft_client.allowance(&seller, &trading_contract_id, &asset_id);
    let new_allowance = if current_allowance >= proposal.token_amount {
        current_allowance - proposal.token_amount
    } else {
        0 // Safety fallback
    };

    // Require authorization for allowance modification in production
    #[cfg(not(test))]
    seller.require_auth_for_args(
        (
            fnft_contract.clone(),
            symbol_short!("approve"),
            (&seller, &trading_contract_id, &asset_id, &new_allowance),
        )
            .into_val(&env),
    );
    fnft_client.approve(&seller, &trading_contract_id, &asset_id, &new_allowance);

    env.storage().persistent().remove(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    ));
    utils::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
    utils::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    events::emit_withdraw_event(&env, &seller, &buyer, asset_id);
}

/// Emergency function: Seller can reset all allowances to 0 for security
pub fn emergency_reset_allowance(env: Env, seller: Address, asset_id: u64) {
    seller.require_auth();

    let fnft_contract = utils::get_fnft_contract(&env);
    let fnft_client = FNFTClient::new(&env, &fnft_contract);
    let trading_contract_id = env.current_contract_address();

    // Reset allowance to 0
    #[cfg(not(test))]
    seller.require_auth_for_args(
        (
            fnft_contract.clone(),
            symbol_short!("approve"),
            (&seller, &trading_contract_id, &asset_id, &0u64),
        )
            .into_val(&env),
    );

    fnft_client.approve(&seller, &trading_contract_id, &asset_id, &0u64);

    events::emit_emergency_reset_event(&env, &seller, asset_id);
}
