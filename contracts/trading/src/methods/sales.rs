use crate::events;
use crate::interfaces::FNFTClient;
use crate::methods::utils;
use crate::storage::{DataKey, SaleProposal, MAX_SALE_DURATION, MIN_SALE_DURATION};
use soroban_sdk::{symbol_short, Address, Env, IntoVal};

/// Step 1: Seller confirms/proposes a sale
/// This grants allowance to the trading contract and creates the proposal
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

    // Verify asset exists and seller has sufficient balance
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

    // Grant allowance to trading contract for secure escrow
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

    // Create sale proposal
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

    // Store proposal
    env.storage().persistent().set(
        &DataKey::SaleProposal(seller.clone(), buyer.clone(), asset_id),
        &proposal,
    );

    // Update seller's active sales list
    utils::add_to_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);

    // Update buyer's offers list
    utils::add_to_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    // Emit sale confirmed event
    events::emit_sale_event(&env, &proposal);
}

/// Clean up expired sale proposals
pub fn cleanup_expired_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
    let proposal = utils::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);

    if env.ledger().timestamp() <= proposal.expires_at {
        panic!("Sale has not expired yet");
    }

    // Remove expired proposal
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

/// Seller withdraws sale proposal (cancels the sale)
pub fn withdraw_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
    seller.require_auth();

    // Get and verify the sale proposal exists
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

    // Get current allowance and subtract this proposal's amount
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

    // Remove proposal and update lists
    env.storage().persistent().remove(&DataKey::SaleProposal(
        seller.clone(),
        buyer.clone(),
        asset_id,
    ));
    utils::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
    utils::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

    // Emit withdrawal event
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

    // Emit emergency reset event
    events::emit_emergency_reset_event(&env, &seller, asset_id);
}
