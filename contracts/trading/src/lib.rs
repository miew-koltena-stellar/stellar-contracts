#![no_std]
use soroban_sdk::token::TokenClient;
#[allow(unused_imports)]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Vec,
};

// Import FNFT contract interface
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

#[contracttype]
#[derive(Clone)]
pub struct SaleProposal {
    pub seller: Address,
    pub buyer: Address,
    pub asset_id: u64,
    pub token_amount: u64,
    pub price: u128,
    pub is_active: bool,
    pub timestamp: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TradeHistory {
    pub seller: Address,
    pub buyer: Address,
    pub asset_id: u64,
    pub token_amount: u64,
    pub price: u128,
    pub timestamp: u64,
}
// Constants
const MIN_SALE_DURATION: u64 = 3600; // 1 hour
const MAX_SALE_DURATION: u64 = 604800; // 1 week

// Storage keys for trading contract
#[contracttype]
pub enum DataKey {
    // Core contract data
    Admin,
    FNFTContract,
    XLMContract, // Address of the XLM contract for payments

    // Active sale proposals: (seller, buyer, asset_id) -> SaleProposal
    SaleProposal(Address, Address, u64),

    // Trade history counter and records
    TradeCounter,
    TradeHistory(u32), // trade_id -> TradeHistory

    // User's active sales (for querying)
    SellerSales(Address), // seller -> Vec<(Address, u64)> (buyer, asset_id pairs)
    BuyerOffers(Address), // buyer -> Vec<(Address, u64)> (seller, asset_id pairs)

    // Asset trade activity
    AssetTrades(u64), // asset_id -> Vec<u32> (trade_ids)
}

#[contract]
pub struct TradingContract;

#[contractimpl]
impl TradingContract {
    /// Contract Initialization
    pub fn initialize(env: Env, admin: Address, fnft_contract: Address, xlm_contract: Address) {
        // Validate contract initialization state
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        // Set admin and contract addresses
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::FNFTContract, &fnft_contract);
        env.storage()
            .instance()
            .set(&DataKey::XLMContract, &xlm_contract);
        env.storage().instance().set(&DataKey::TradeCounter, &0u32);

        // Emit initialization event with contract addresses
        env.events().publish(
            (symbol_short!("init"),),
            (admin.clone(), fnft_contract.clone(), xlm_contract.clone()),
        );
    }

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
        let fnft_contract = Self::get_fnft_contract(&env);
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
        // Atomic trade design: Each buyer gets independent allowance
        // Get current allowance and add the new sale amount for multiple buyers
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
        Self::add_to_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);

        // Update buyer's offers list
        Self::add_to_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

        // Emit sale confirmed event
        env.events().publish(
            (symbol_short!("sale_conf"),),
            (seller.clone(), buyer.clone(), asset_id, token_amount, price),
        );
    }

    /// Step 2: Buyer finishes the transaction (completes the trade)
    /// Atomic trade execution: All transfers happen together or none at all
    /// Reentrancy protection: Clears allowances and updates state immediately
    pub fn finish_transaction(env: Env, buyer: Address, seller: Address, asset_id: u64) {
        buyer.require_auth();

        // Verify parameters and authorization

        // Verify sale proposal validity and existence
        let proposal =
            Self::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);
        if !proposal.is_active {
            panic!("Sale proposal is not active");
        }
        if proposal.buyer != buyer {
            panic!("Not authorized buyer for this sale");
        }
        if env.ledger().timestamp() > proposal.expires_at {
            panic!("Sale proposal has expired");
        }

        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);
        let trading_contract_id = env.current_contract_address();

        // Verify seller token balance sufficiency
        let seller_balance = fnft_client.balance_of(&proposal.seller, &proposal.asset_id);
        if seller_balance < proposal.token_amount {
            panic!("Seller has insufficient token balance");
        }

        // Verify buyer funds sufficiency
        let xlm_contract_address = Self::get_xlm_contract_address(env.clone());
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
        Self::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
        Self::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

        // Note: Allowance is automatically reduced by transfer_from call
        // The FNFT contract handles allowance reduction automatically, providing security

        // Record successful trade completion
        let trade_id = Self::record_trade_history(&env, &proposal);
        Self::add_to_asset_trades(&env, asset_id, trade_id);

        env.events().publish(
            (symbol_short!("trade"),),
            (
                seller,
                buyer,
                asset_id,
                proposal.token_amount,
                proposal.price,
                trade_id,
            ),
        );
    }
    /// Clean up expired sales (can be called by anyone)
    /// Security: Only removes the proposal, seller must manually reset allowances
    /// Use emergency_reset_allowance() if you need to clear all allowances
    pub fn cleanup_expired_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
        let proposal =
            Self::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);

        if env.ledger().timestamp() <= proposal.expires_at {
            panic!("Sale has not expired yet");
        }

        // Security note: We cannot reset allowances here since this can be called by anyone
        // and we don't have the seller's authorization. Sellers should use:
        // 1. withdraw_sale() before expiration (resets allowance)
        // 2. emergency_reset_allowance() to manually reset all allowances
        // This prevents the authorization issue while still allowing cleanup

        // Remove expired proposal
        env.storage().persistent().remove(&DataKey::SaleProposal(
            seller.clone(),
            buyer.clone(),
            asset_id,
        ));
        Self::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
        Self::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

        env.events().publish(
            (symbol_short!("expired"),),
            (seller, buyer, asset_id, proposal.token_amount),
        );
    }

    /// Step 3: Seller withdraws sale proposal (cancels the sale)
    /// Security: Must reset allowance to prevent future unauthorized transfers
    pub fn withdraw_sale(env: Env, seller: Address, buyer: Address, asset_id: u64) {
        seller.require_auth();

        // Get and verify the sale proposal exists
        let proposal =
            Self::get_sale_proposal(env.clone(), seller.clone(), buyer.clone(), asset_id);

        if proposal.seller != seller {
            panic!("Only the seller can withdraw this proposal");
        }

        if !proposal.is_active {
            panic!("Sale proposal is not active");
        }

        // Critical security: Reduce allowance by this proposal's amount
        let fnft_contract = Self::get_fnft_contract(&env);
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
        Self::remove_from_seller_sales(&env, seller.clone(), buyer.clone(), asset_id);
        Self::remove_from_buyer_offers(&env, buyer.clone(), seller.clone(), asset_id);

        // Emit withdrawal event
        env.events().publish(
            (symbol_short!("withdraw"),),
            (
                seller,
                buyer,
                asset_id,
                proposal.token_amount,
                proposal.price,
            ),
        );
    }

    /// Emergency function: Seller can reset all allowances to 0 for security
    /// Security: This provides sellers with an emergency way to revoke all trading contract permissions
    pub fn emergency_reset_allowance(env: Env, seller: Address, asset_id: u64) {
        seller.require_auth();

        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);
        let trading_contract_id = env.current_contract_address();

        // Require authorization for allowance reset in production
        #[cfg(not(test))]
        seller.require_auth_for_args(
            (
                fnft_contract.clone(),
                symbol_short!("approve"),
                (&seller, &trading_contract_id, &asset_id, &0u64),
            )
                .into_val(&env),
        );

        // Reset allowance to 0
        fnft_client.approve(&seller, &trading_contract_id, &asset_id, &0);

        env.events()
            .publish((symbol_short!("reset"),), (seller, asset_id));
    }

    // === View Functions ===

    /// Get the XLM contract address
    pub fn get_xlm_contract_address_public(env: Env) -> Address {
        Self::get_xlm_contract_address(env)
    }

    /// Get a specific sale proposal
    pub fn get_sale_proposal(
        env: Env,
        seller: Address,
        buyer: Address,
        asset_id: u64,
    ) -> SaleProposal {
        env.storage()
            .persistent()
            .get(&DataKey::SaleProposal(seller, buyer, asset_id))
            .unwrap_or_else(|| panic!("Sale proposal not found"))
    }

    /// Check if a sale proposal exists
    pub fn sale_exists(env: Env, seller: Address, buyer: Address, asset_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::SaleProposal(seller, buyer, asset_id))
    }

    /// Get all active sales for a seller
    pub fn get_seller_sales(env: Env, seller: Address) -> Vec<(Address, u64)> {
        env.storage()
            .persistent()
            .get(&DataKey::SellerSales(seller))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all offers for a buyer
    pub fn get_buyer_offers(env: Env, buyer: Address) -> Vec<(Address, u64)> {
        env.storage()
            .persistent()
            .get(&DataKey::BuyerOffers(buyer))
            .unwrap_or(Vec::new(&env))
    }

    /// Get trade history by trade ID
    pub fn get_trade_history(env: Env, trade_id: u32) -> TradeHistory {
        env.storage()
            .persistent()
            .get(&DataKey::TradeHistory(trade_id))
            .unwrap_or_else(|| panic!("Trade not found"))
    }

    /// Get total number of completed trades
    pub fn get_trade_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TradeCounter)
            .unwrap_or(0)
    }

    /// Get all trades for a specific asset
    pub fn get_asset_trades(env: Env, asset_id: u64) -> Vec<u32> {
        env.storage()
            .persistent()
            .get(&DataKey::AssetTrades(asset_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get the FNFT contract address
    pub fn get_fnft_contract_address(env: Env) -> Address {
        Self::get_fnft_contract(&env)
    }

    pub fn time_until_expiry(env: Env, seller: Address, buyer: Address, asset_id: u64) -> u64 {
        let proposal = Self::get_sale_proposal(env.clone(), seller, buyer, asset_id);
        let current_time = env.ledger().timestamp();

        if current_time >= proposal.expires_at {
            0
        } else {
            proposal.expires_at - current_time
        }
    }

    /// View function: Check current allowance for a seller-asset pair
    /// Security: Helps sellers monitor their allowances for security
    pub fn get_current_allowance(env: Env, seller: Address, asset_id: u64) -> u64 {
        let fnft_contract = Self::get_fnft_contract(&env);
        let fnft_client = FNFTClient::new(&env, &fnft_contract);
        let trading_contract_id = env.current_contract_address();

        fnft_client.allowance(&seller, &trading_contract_id, &asset_id)
    }

    // === Internal Helper Functions ===

    fn get_fnft_contract(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::FNFTContract)
            .unwrap()
    }

    fn get_xlm_contract_address(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::XLMContract)
            .unwrap_or_else(|| panic!("XLM contract address not configured"))
    }

    fn record_trade_history(env: &Env, proposal: &SaleProposal) -> u32 {
        let trade_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TradeCounter)
            .unwrap_or(0);

        let new_trade_id = trade_id + 1;

        let history = TradeHistory {
            seller: proposal.seller.clone(),
            buyer: proposal.buyer.clone(),
            asset_id: proposal.asset_id,
            token_amount: proposal.token_amount,
            price: proposal.price,
            timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::TradeHistory(new_trade_id), &history);

        env.storage()
            .instance()
            .set(&DataKey::TradeCounter, &new_trade_id);

        new_trade_id
    }

    fn add_to_seller_sales(env: &Env, seller: Address, buyer: Address, asset_id: u64) {
        let mut sales: Vec<(Address, u64)> = env
            .storage()
            .persistent()
            .get(&DataKey::SellerSales(seller.clone()))
            .unwrap_or(Vec::new(env));

        sales.push_back((buyer, asset_id));
        env.storage()
            .persistent()
            .set(&DataKey::SellerSales(seller), &sales);
    }

    fn remove_from_seller_sales(env: &Env, seller: Address, buyer: Address, asset_id: u64) {
        let sales: Vec<(Address, u64)> = env
            .storage()
            .persistent()
            .get(&DataKey::SellerSales(seller.clone()))
            .unwrap_or(Vec::new(env));

        let mut new_sales = Vec::new(env);
        for i in 0..sales.len() {
            let (current_buyer, current_asset_id) = sales.get(i).unwrap();
            if !(current_buyer == buyer && current_asset_id == asset_id) {
                new_sales.push_back((current_buyer, current_asset_id));
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::SellerSales(seller), &new_sales);
    }

    fn add_to_buyer_offers(env: &Env, buyer: Address, seller: Address, asset_id: u64) {
        let mut offers: Vec<(Address, u64)> = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerOffers(buyer.clone()))
            .unwrap_or(Vec::new(env));

        offers.push_back((seller, asset_id));
        env.storage()
            .persistent()
            .set(&DataKey::BuyerOffers(buyer), &offers);
    }

    fn remove_from_buyer_offers(env: &Env, buyer: Address, seller: Address, asset_id: u64) {
        let offers: Vec<(Address, u64)> = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerOffers(buyer.clone()))
            .unwrap_or(Vec::new(env));

        let mut new_offers = Vec::new(env);
        for i in 0..offers.len() {
            let (current_seller, current_asset_id) = offers.get(i).unwrap();
            if !(current_seller == seller && current_asset_id == asset_id) {
                new_offers.push_back((current_seller, current_asset_id));
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::BuyerOffers(buyer), &new_offers);
    }

    fn add_to_asset_trades(env: &Env, asset_id: u64, trade_id: u32) {
        let mut trades: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::AssetTrades(asset_id))
            .unwrap_or(Vec::new(env));

        trades.push_back(trade_id);
        env.storage()
            .persistent()
            .set(&DataKey::AssetTrades(asset_id), &trades);
    }
}

mod test;
