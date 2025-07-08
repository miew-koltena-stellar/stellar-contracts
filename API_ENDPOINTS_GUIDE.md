# 🚀 Koltena Stellar Smart Contract API Guide

This guide documents all public functions (endpoints) across the three main contracts in the Koltena ecosystem. Each function can be called like an API endpoint from:
- Frontend applications (JavaScript/TypeScript)
- Other smart contracts
- CLI tools (`stellar contract invoke`)
- REST APIs

---

## 📋 Table of Contents
1. [Governance Contract](#governance-contract)
2. [Fractcore Contract](#fractcore-contract) 
3. [Funding Contract](#funding-contract)
4. [Trading Contract](#trading-contract)
5. [Usage Examples](#usage-examples)

---

## 🏛️ Governance Contract

The main governance system for decentralized decision making.

### 🔧 Admin Functions

#### `initialize`
**Purpose:** Initialize the governance contract
```rust
fn initialize(
    admin: Address,
    fractcore_contract: Address,
    funding_contract: Address,
    default_threshold: u32,     // % approval needed (e.g., 51)
    default_quorum: u32,        // % participation needed (e.g., 30)  
    default_expiry_days: u32    // Poll duration in days (e.g., 7)
) -> Result<(), GovernanceError>
```
**When to use:** Deploy and setup the governance system
**Access:** Admin only

#### `update_governance_params`
**Purpose:** Update voting thresholds and quorum requirements
```rust
fn update_governance_params(
    caller: Address,
    threshold_percentage: u32,
    quorum_percentage: u32,
    default_expiry_days: u32
) -> Result<(), GovernanceError>
```
**When to use:** Adjust governance parameters as the system evolves
**Access:** Admin only

### 📊 Poll Management

#### `create_poll` ⭐
**Purpose:** Create a new governance poll for voting
```rust
fn create_poll(
    caller: Address,
    asset_id: u64,                    // Which asset holders can vote
    title: String,                    // Poll title
    description: String,              // Poll description  
    action: PollAction,              // What happens if approved
    duration_days: Option<u32>       // Custom duration (optional)
) -> Result<u32, GovernanceError>   // Returns poll_id
```
**Poll Actions:**
- `NoExecution` - Opinion poll only
- `DistributeFunds(amount, description)` - Distribute funds to token holders
- `TransferTokens(to_address, amount)` - Transfer tokens from governance

**When to use:** Start any governance decision
**Access:** Asset holders or admin
**Returns:** `poll_id` for tracking

#### `vote` ⭐
**Purpose:** Cast a vote on an active poll
```rust
fn vote(
    voter: Address,
    poll_id: u32,
    option_index: u32    // 0 = Deny, 1 = Approve
) -> Result<(), GovernanceError>
```
**When to use:** Participate in governance decisions
**Access:** Asset holders with voting power
**Auto-execution:** Poll may execute automatically if all owners vote

#### `check_and_execute_poll` ⭐
**Purpose:** Manually execute a poll if conditions are met
```rust
fn check_and_execute_poll(
    poll_id: u32
) -> Result<bool, GovernanceError>  // Returns true if executed
```
**When to use:** Execute expired polls or check execution status
**Access:** Anyone
**Execution conditions:** Poll expired OR all asset owners voted

### 📖 Query Functions

#### `get_poll` ⭐
**Purpose:** Get complete poll information
```rust
fn get_poll(poll_id: u32) -> Poll
```
**Returns:**
```rust
struct Poll {
    id: u32,
    asset_id: u64,
    creator: Address,
    title: String,
    description: String,
    options: Vec<String>,      // ["Deny", "Approve"]
    action: PollAction,
    start_time: u64,
    end_time: u64,
    is_active: bool,
    votes: Map<Address, Vote>,
    total_voters: u32
}
```

#### `get_vote_results` ⭐
**Purpose:** Get voting results and statistics
```rust
fn get_vote_results(poll_id: u32) -> Result<VoteResults, GovernanceError>
```
**Returns:**
```rust
struct VoteResults {
    poll_id: u32,
    vote_counts: Vec<u64>,     // [deny_votes, approve_votes]
    winning_option: u32,       // 0 or 1
    total_voters: u32,
    is_finalized: bool
}
```

#### `can_vote`
**Purpose:** Check if an address can vote on a poll
```rust
fn can_vote(voter: Address, poll_id: u32) -> Result<bool, GovernanceError>
```

#### `get_governance_params`
**Purpose:** Get current governance settings
```rust
fn get_governance_params() -> GovernanceParams
```

#### `get_polls_by_asset`
**Purpose:** Get all poll IDs for an asset
```rust
fn get_polls_by_asset(asset_id: u64) -> Vec<u32>
```

#### `get_active_polls`
**Purpose:** Get all currently active poll IDs
```rust
fn get_active_polls() -> Vec<u32>
```

---

## 🔢 Fractcore Contract

Token fractionalization and ownership management.

### 🔧 Admin Functions

#### `initialize`
**Purpose:** Initialize the fractcore contract
```rust
fn initialize(admin: Address)
```

### 🪙 Token Management

#### `mint` ⭐
**Purpose:** Create a new fractionalized asset
```rust
fn mint(
    to: Address,
    num_tokens: u64
) -> u64  // Returns new asset_id
```
**When to use:** Fractionalizing a new real-world asset
**Access:** Admin only
**Returns:** Unique `asset_id` for the new asset

#### `mint_to` ⭐
**Purpose:** Distribute tokens of existing asset to multiple recipients
```rust
fn mint_to(
    asset_id: u64,
    recipients: Vec<Address>,
    amounts: Vec<u64>
)
```
**When to use:** Initial distribution or secondary sales
**Access:** Admin only

#### `transfer` ⭐
**Purpose:** Transfer tokens between addresses
```rust
fn transfer(
    from: Address,
    to: Address,
    asset_id: u64,
    amount: u64
)
```
**When to use:** P2P token transfers, marketplace transactions
**Access:** Token owner

#### `transfer_from`
**Purpose:** Transfer tokens using allowance system
```rust
fn transfer_from(
    operator: Address,
    from: Address,
    to: Address,
    asset_id: u64,
    amount: u64
)
```
**When to use:** Marketplace contracts, automated transfers
**Access:** Approved operators

### 📊 Query Functions

#### `balance_of` ⭐
**Purpose:** Get token balance for an owner
```rust
fn balance_of(owner: Address, asset_id: u64) -> u64
```

#### `asset_supply` ⭐
**Purpose:** Get total supply of an asset
```rust
fn asset_supply(asset_id: u64) -> u64
```

#### `asset_owners` ⭐
**Purpose:** Get list of all owners of an asset
```rust
fn asset_owners(asset_id: u64) -> Vec<Address>
```

#### `get_asset_owner_count` ⭐
**Purpose:** Get number of unique owners for an asset
```rust
fn get_asset_owner_count(asset_id: u64) -> u32
```

#### `balance_of_batch`
**Purpose:** Get multiple balances in one call
```rust
fn balance_of_batch(
    owners: Vec<Address>,
    asset_ids: Vec<u64>
) -> Vec<u64>
```

#### `asset_exists`
**Purpose:** Check if an asset exists
```rust
fn asset_exists(asset_id: u64) -> bool
```

#### `owns_asset`
**Purpose:** Check if address owns any tokens of an asset
```rust
fn owns_asset(owner: Address, asset_id: u64) -> bool
```

### 🔒 Approval Functions

#### `approve`
**Purpose:** Approve another address to transfer tokens
```rust
fn approve(
    owner: Address,
    operator: Address,
    asset_id: u64,
    amount: u64
)
```

#### `get_approved`
**Purpose:** Get approved amount for an operator
```rust
fn get_approved(
    owner: Address,
    operator: Address,
    asset_id: u64
) -> u64
```

---

## 💰 Funding Contract

Manages XLM funds for fractionalized assets and distributions.

### 🔧 Admin Functions

#### `initialize`
**Purpose:** Initialize the funding contract
```rust
fn initialize(
    admin: Address,
    fnft_contract: Address,  // Fractcore contract address
    xlm_token: Address       // XLM token contract address
)
```

#### `set_governance_contract`
**Purpose:** Authorize governance contract to trigger distributions
```rust
fn set_governance_contract(
    admin: Address,
    governance_contract: Address
)
```
**When to use:** Connect governance to funding system
**Access:** Admin only

### 💵 Fund Management

#### `deposit_funds` ⭐
**Purpose:** Deposit XLM funds for a specific asset
```rust
fn deposit_funds(
    depositor: Address,
    asset_id: u64,
    amount: i128
)
```
**When to use:** Property owners depositing rental income, sale proceeds
**Access:** Anyone (typically asset managers)

#### `distribute_funds` ⭐
**Purpose:** Distribute funds to all token holders
```rust
fn distribute_funds(
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String
)
```
**When to use:** Triggered by governance approval for distributions
**Access:** Admin or governance contract only
**Effect:** Proportionally distributes funds to all token holders

#### `owner_distribute_funds`
**Purpose:** Democratic distribution where token holders vote directly
```rust
fn owner_distribute_funds(
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String
)
```
**When to use:** Direct democracy without formal governance
**Access:** Token holders

#### `emergency_withdraw`
**Purpose:** Emergency fund withdrawal
```rust
fn emergency_withdraw(
    admin: Address,
    asset_id: u64,
    amount: u128,
    reason: String
)
```
**When to use:** Contract upgrades, emergencies
**Access:** Admin only

### 📊 Query Functions

#### `asset_funds` ⭐
**Purpose:** Get total available funds for an asset
```rust
fn asset_funds(asset_id: u64) -> u128
```

#### `total_distributed` ⭐
**Purpose:** Get total amount ever distributed for an asset
```rust
fn total_distributed(asset_id: u64) -> u128
```

#### `get_distribution_history`
**Purpose:** Get distribution history for an asset
```rust
fn get_distribution_history(asset_id: u64) -> Vec<DistributionRecord>
```

---

## 🛒 Trading Contract

The peer-to-peer trading system for fractional NFT tokens with XLM payments.

### 🔧 Admin Functions

#### `initialize`
**Purpose:** Initialize the trading contract
```rust
fn initialize(
    admin: Address,
    fnft_contract: Address,
    xlm_contract: Address
) -> Result<(), TradingError>
```
**When to use:** Deploy and setup the trading system
**Access:** Admin only

### 💰 Trading Functions

#### `confirm_sale` ⭐
**Purpose:** Seller creates a sale proposal and grants allowance
```rust
fn confirm_sale(
    seller: Address,
    buyer: Address,
    asset_id: u64,
    token_amount: u64,
    price: u128,                // XLM price in stroops
    duration_seconds: u64       // How long offer is valid
) -> Result<(), TradingError>
```
**What happens:**
- Grants allowance to trading contract
- Creates sale proposal 
- Adds to seller's active sales list

**When to use:** Seller wants to sell tokens to specific buyer
**Access:** Asset owners only
**Duration:** 1 hour to 1 week

#### `finish_transaction` ⭐ 
**Purpose:** Buyer completes the purchase with security validation
```rust
fn finish_transaction(
    buyer: Address,
    seller: Address,
    asset_id: u64,
    expected_token_amount: u64,  // Buyer protection: expected amount
    expected_price: u128         // Buyer protection: expected price
) -> Result<(), TradingError>
```
**What happens:**
- Validates proposal matches buyer's expectations (prevents bait-and-switch)
- Transfers tokens from seller to buyer
- Transfers XLM from buyer to seller
- Records trade history
- Cleans up proposal

**When to use:** Buyer accepts seller's proposal
**Access:** Authorized buyer only
**Security:** Validates proposal terms against buyer's expectations to prevent tampering

#### `withdraw_sale`
**Purpose:** Seller cancels an active sale
```rust
fn withdraw_sale(
    seller: Address,
    buyer: Address,
    asset_id: u64
) -> Result<(), TradingError>
```
**When to use:** Seller wants to cancel before buyer purchases
**Access:** Seller only

#### `cleanup_expired_sale`
**Purpose:** Clean up expired sale proposals
```rust
fn cleanup_expired_sale(
    seller: Address,
    buyer: Address,
    asset_id: u64
) -> Result<(), TradingError>
```
**When to use:** Clean up old expired proposals
**Access:** Anyone

#### `emergency_reset_allowance`
**Purpose:** Reset stuck allowances in emergency
```rust
fn emergency_reset_allowance(
    seller: Address,
    asset_id: u64
) -> Result<(), TradingError>
```
**When to use:** Fix stuck allowance issues
**Access:** Asset owners

### 📖 Query Functions

#### `get_sale_proposal`
**Purpose:** Get details of a specific sale proposal
```rust
fn get_sale_proposal(
    seller: Address,
    buyer: Address,
    asset_id: u64
) -> SaleProposal
```
**Returns:** Full proposal details (price, amount, expiry, etc.)

#### `sale_exists`
**Purpose:** Check if a sale proposal exists
```rust
fn sale_exists(
    seller: Address,
    buyer: Address,
    asset_id: u64
) -> bool
```

#### `get_seller_sales`
**Purpose:** Get all active sales for a seller
```rust
fn get_seller_sales(seller: Address) -> Vec<(Address, u64)>
```
**Returns:** List of (buyer_address, asset_id) pairs

#### `get_buyer_offers`
**Purpose:** Get all pending offers for a buyer
```rust
fn get_buyer_offers(buyer: Address) -> Vec<(Address, u64)>
```
**Returns:** List of (seller_address, asset_id) pairs

#### `get_trade_history`
**Purpose:** Get details of a completed trade
```rust
fn get_trade_history(trade_id: u32) -> TradeHistory
```

#### `get_trade_count`
**Purpose:** Get total number of completed trades
```rust
fn get_trade_count() -> u32
```

#### `get_asset_trades`
**Purpose:** Get all trade IDs for a specific asset
```rust
fn get_asset_trades(asset_id: u64) -> Vec<u32>
```

#### `time_until_expiry`
**Purpose:** Get remaining time on a sale proposal
```rust
fn time_until_expiry(
    seller: Address,
    buyer: Address,
    asset_id: u64
) -> u64
```
**Returns:** Seconds until expiry

#### `get_current_allowance`
**Purpose:** Check current allowance for seller
```rust
fn get_current_allowance(
    seller: Address,
    asset_id: u64
) -> u64
```

#### `get_fnft_contract_address` & `get_xlm_contract_address_public`
**Purpose:** Get connected contract addresses
```rust
fn get_fnft_contract_address() -> Address
fn get_xlm_contract_address_public() -> Address
```

---

## 💰 Funding Contract

Manages XLM funds for fractionalized assets and distributions.

### 🔧 Admin Functions

#### `initialize`
**Purpose:** Initialize the funding contract
```rust
fn initialize(
    admin: Address,
    fnft_contract: Address,  // Fractcore contract address
    xlm_token: Address       // XLM token contract address
)
```

#### `set_governance_contract`
**Purpose:** Authorize governance contract to trigger distributions
```rust
fn set_governance_contract(
    admin: Address,
    governance_contract: Address
)
```
**When to use:** Connect governance to funding system
**Access:** Admin only

### 💵 Fund Management

#### `deposit_funds` ⭐
**Purpose:** Deposit XLM funds for a specific asset
```rust
fn deposit_funds(
    depositor: Address,
    asset_id: u64,
    amount: i128
)
```
**When to use:** Property owners depositing rental income, sale proceeds
**Access:** Anyone (typically asset managers)

#### `distribute_funds` ⭐
**Purpose:** Distribute funds to all token holders
```rust
fn distribute_funds(
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String
)
```
**When to use:** Triggered by governance approval for distributions
**Access:** Admin or governance contract only
**Effect:** Proportionally distributes funds to all token holders

#### `owner_distribute_funds`
**Purpose:** Democratic distribution where token holders vote directly
```rust
fn owner_distribute_funds(
    caller: Address,
    asset_id: u64,
    amount: u128,
    description: String
)
```
**When to use:** Direct democracy without formal governance
**Access:** Token holders

#### `emergency_withdraw`
**Purpose:** Emergency fund withdrawal
```rust
fn emergency_withdraw(
    admin: Address,
    asset_id: u64,
    amount: u128,
    reason: String
)
```
**When to use:** Contract upgrades, emergencies
**Access:** Admin only

### 📊 Query Functions

#### `asset_funds` ⭐
**Purpose:** Get total available funds for an asset
```rust
fn asset_funds(asset_id: u64) -> u128
```

#### `total_distributed` ⭐
**Purpose:** Get total amount ever distributed for an asset
```rust
fn total_distributed(asset_id: u64) -> u128
```

#### `get_distribution_history`
**Purpose:** Get distribution history for an asset
```rust
fn get_distribution_history(asset_id: u64) -> Vec<DistributionRecord>
```

---

## 💻 Usage Examples

### JavaScript/TypeScript Frontend

```typescript
// Initialize contracts
const governance = new Contract({
  contractId: "GOVERNANCE_CONTRACT_ID",
  rpc: server,
  networkPassphrase: Networks.TESTNET
});

// Create a funding distribution poll
const pollId = await governance.create_poll({
  caller: userAddress,
  asset_id: 123,
  title: "Q4 Rental Distribution",
  description: "Distribute $50,000 in rental income to token holders",
  action: {
    tag: "DistributeFunds",
    values: [50000, "Q4 rental income"]
  },
  duration_days: 7
});

// Vote on the poll
await governance.vote({
  voter: userAddress,
  poll_id: pollId,
  option_index: 1  // Approve
});

// Check poll results
const results = await governance.get_vote_results({
  poll_id: pollId
});

// Get token balance
const balance = await fractcore.balance_of({
  owner: userAddress,
  asset_id: 123
});

// Check available funds
const funds = await funding.asset_funds({
  asset_id: 123
});
```

### CLI Examples

```bash
# Create a poll
stellar contract invoke \
  --id $GOVERNANCE_CONTRACT \
  --source $USER_SECRET \
  -- \
  create_poll \
  --caller $USER_ADDRESS \
  --asset-id 123 \
  --title "Equipment Purchase" \
  --description "Buy new gaming equipment for $25,000" \
  --action '{"DistributeFunds": [25000, "Equipment purchase"]}' \
  --duration-days 5

# Vote on poll
stellar contract invoke \
  --id $GOVERNANCE_CONTRACT \
  --source $VOTER_SECRET \
  -- \
  vote \
  --voter $VOTER_ADDRESS \
  --poll-id 1 \
  --option-index 1

# Check token balance
stellar contract invoke \
  --id $FRACTCORE_CONTRACT \
  -- \
  balance_of \
  --owner $USER_ADDRESS \
  --asset-id 123

# Deposit funds
stellar contract invoke \
  --id $FUNDING_CONTRACT \
  --source $DEPOSITOR_SECRET \
  -- \
  deposit_funds \
  --depositor $DEPOSITOR_ADDRESS \
  --asset-id 123 \
  --amount 100000
```

---

## 🔗 Common Workflows

### 1. **Asset Creation & Initial Distribution**
1. `fractcore.mint()` - Create new asset
2. `fractcore.mint_to()` - Distribute initial tokens
3. `funding.deposit_funds()` - Add initial funds

### 2. **Governance-Driven Fund Distribution**
1. `governance.create_poll()` - Create distribution proposal
2. `governance.vote()` - Token holders vote
3. Auto-execution triggers `funding.distribute_funds()`

### 3. **Token Trading**
1. `fractcore.approve()` - Approve marketplace
2. `fractcore.transfer_from()` - Execute trade
3. Update ownership automatically

### 4. **Asset Management**
1. `funding.deposit_funds()` - Property manager deposits income
2. `governance.create_poll()` - Propose distribution
3. Token holders vote and funds distribute automatically

---

## ⚡ Key Features

- **🔄 Auto-execution:** Polls execute automatically when all owners vote
- **🔗 Cross-contract:** Seamless integration between all three contracts  
- **⚖️ Weighted voting:** Voting power proportional to token ownership
- **💰 Proportional distribution:** Funds distributed based on token holdings
- **🛡️ Access control:** Role-based permissions for all functions
- **📊 Rich queries:** Comprehensive data access for frontends

---

This API guide covers all the essential endpoints you'll need to build a complete governance and asset management system on Stellar! 🚀
