# Koltena on Stellar: Fractional NFT Platform

🚨 **TESTNET ONLY** - This project is currently for testing purposes only. Do not use on mainnet.

Koltena is a comprehensive platform that democratizes ownership of real-world assets through sophisticated tokenization, trading, and revenue distribution mechanisms built on Stellar/Soroban.

## Platform Overview

Koltena enables **complete asset fractionalization ecosystem** with **real XLM operations**:

1. **Asset Tokenization**: Convert real-world assets into tradeable fractional NFTs (F-NFTs)
2. **P2P Trading**: Direct peer-to-peer trading of asset fractions with XLM settlements
3. **Revenue Distribution**: Collect and distribute XLM revenue proportionally to token holders
4. **Cross-Contract Integration**: Seamless interaction between fractionalization, trading, and funding
5. **Democratic Governance**: Token holder participation in asset management decisions

## Why Stellar/Soroban?

Built specifically for Stellar to leverage:
- **Native XLM Integration**: Direct XLM custody and transfers without wrapping
- **Lower Transaction Costs**: More efficient than Ethereum for micro-transactions
- **Fast Finality**: Quick settlement for trading and distributions
- **Asset Contract Standards**: Native integration with Stellar Asset Contract (SAC) pattern
```

Table of Contents
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Contract Overview](#contract-overview)
- [Setup & Deployment](#setup--deployment)
- [Basic Usage](#basic-usage)
- [Advanced Operations](#advanced-operations)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before setting up Koltena, ensure you have the following installed:

### 1. Rust & Cargo
```powershell
# Install Rust (if not already installed)
winget install Rustlang.Rust.MSVC

# Verify installation
rustc --version
cargo --version
```

### 2. Stellar CLI
```powershell
# Install Stellar CLI
cargo install --locked stellar-cli

# Add the WASM target for Soroban contracts
rustup target add wasm32-unknown-unknown

# Verify installation
stellar --version
rustup target list --installed
```

### 3. Node.js (for TypeScript utilities)
```powershell
# Install Node.js
winget install OpenJS.NodeJS

# Verify installation
node --version
npm --version
```

## Installation

### 1. Clone the Repository
```powershell
git clone https://github.com/miew-koltena-stellar/stellar-contracts.git
cd koltena-stellar
```

### 2. Configure Stellar CLI for Testnet
```powershell
# Set network to testnet
stellar network add --global testnet --rpc-url https://soroban-testnet.stellar.org:443 --network-passphrase "Test SDF Network ; September 2015"

# Set default network
stellar network ls
```

### 3. Create or Import Stellar Account
```powershell
# Option A: Generate new keypair for testing
stellar keys generate --global test-admin

# Option B: Import existing secret key
stellar keys add test-admin --secret-key SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Fund account with testnet XLM (required for contract deployment)
stellar keys fund test-admin --network testnet
```

## Contract Architecture

Koltena consists of three production-ready smart contracts working together:

### 1. **Fractional NFT (F-NFT) Contract** (`contracts/fractcore/`)
**Purpose**: Core asset fractionalization and ownership management
- **Asset Creation**: Mint fractional NFTs representing real-world asset ownership
- **Token Management**: ERC-1155 compatible multi-token standard
- **Ownership Tracking**: Efficient storage of token holders and balances
- **Cross-Contract Integration**: Provides ownership data to trading and funding contracts
- **Approval System**: Token allowances for trading and automated operations

**Key Features:**
- Create assets with custom token supplies (e.g., 1000 tokens = 100% ownership)
- Batch minting to multiple recipients
- Efficient ownership queries for large holder lists
- Automatic cleanup of zero-balance holders

### 2. **Trading Contract** (`contracts/trading/`)
**Purpose**: Peer-to-peer fractional asset trading
- **Direct Trading**: User-to-user token exchanges with XLM settlements
- **Sale Proposals**: Create and manage trading offers between specific parties
- **Atomic Settlements**: Secure token-for-XLM exchanges
- **Trading History**: Complete audit trail of all transactions
- **Expiration Management**: Time-limited offers with automatic cleanup

**Key Features:**
- Confirmed sale system (both parties must agree)
- Real XLM transfers for payments
- Trading analytics and history tracking
- Emergency functions for stuck transactions

### 3. **Funding Contract** (`contracts/funding/`)
**Purpose**: Revenue collection and proportional distribution
- **XLM Deposits**: Accept revenue deposits for specific assets
- **Proportional Distribution**: Automatically calculate and distribute funds to token holders
- **Multiple Distribution Methods**: Admin-controlled or democratic (owner-initiated)
- **Real XLM Custody**: Contract holds and transfers actual XLM funds
- **Emergency Controls**: Admin withdrawal capabilities for exceptional situations

**Key Features:**
- Real XLM transfers using Stellar Asset Contract (SAC) interface
- Proportional distribution based on token ownership percentages
- Dust handling for integer division remainders
- Complete distribution audit trail

## Contract Integration Flow

```
Real World Asset (e.g., rental property, gaming team)
    ↓
F-NFT Contract: Issues 1000 tokens representing 100% ownership
    ↓ 
User A buys 300 tokens (30%) via Trading Contract
User B buys 700 tokens (70%) via Trading Contract
    ↓
Revenue arrives → Funding Contract receives XLM
    ↓
Distribution: Funding Contract sends XLM proportionally:
- User A gets 30% of revenue in XLM
- User B gets 70% of revenue in XLM
```


## Setup & Deployment

### 1. Build All Contracts
```powershell
# Build all contracts from root directory
cargo build --release

# Or build individually using Stellar CLI (recommended)
cd contracts/fractcore
stellar contract build

cd ../trading  
stellar contract build

cd ../funding
stellar contract build
```

**Note:** If you see "make command not found" errors, use `stellar contract build` instead of `make build` on Windows.

### 2. Deploy Contracts to Testnet

#### Deploy F-NFT Contract
```powershell
# Deploy from project root
stellar contract deploy --wasm target/wasm32v1-none/release/fractcore.wasm --source test-admin --network testnet

# Save the contract ID
# Example: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
set FRACTCORE_CONTRACT_ID=YOUR_ACTUAL_CONTRACT_ID_HERE
```

#### Deploy Trading Contract
```powershell
# Deploy trading contract
stellar contract deploy --wasm target/wasm32v1-none/release/trading.wasm --source test-admin --network testnet

# Save the contract ID
set TRADING_CONTRACT_ID=YOUR_ACTUAL_CONTRACT_ID_HERE
```

#### Deploy Funding Contract
```powershell
# Deploy funding contract
stellar contract deploy --wasm target/wasm32v1-none/release/funding.wasm --source test-admin --network testnet

# Save the contract ID
set FUNDING_CONTRACT_ID=YOUR_ACTUAL_CONTRACT_ID_HERE
```

### 3. Initialize Contracts

#### Initialize F-NFT Contract
```powershell
# Initialize with admin address
stellar contract invoke --id %FRACTCORE_CONTRACT_ID% --source test-admin --network testnet -- initialize --admin $(stellar keys address test-admin)
```

#### Initialize Trading Contract
```powershell
# Get XLM contract address (native asset on Stellar)
# Note: Use actual Stellar native asset contract address in production
set XLM_CONTRACT_ADDRESS=NATIVE_XLM_CONTRACT_ADDRESS

# Initialize trading contract
stellar contract invoke --id %TRADING_CONTRACT_ID% --source test-admin --network testnet -- initialize --admin $(stellar keys address test-admin) --fnft_contract %FRACTCORE_CONTRACT_ID% --xlm_contract %XLM_CONTRACT_ADDRESS%
```

#### Initialize Funding Contract
```powershell
# Initialize funding contract
stellar contract invoke --id %FUNDING_CONTRACT_ID% --source test-admin --network testnet -- initialize --admin $(stellar keys address test-admin) --fnft_contract %FRACTCORE_CONTRACT_ID% --xlm_token %XLM_CONTRACT_ADDRESS%
```
```

#### Deploy Trading Contract  
```powershell
cd ../trading

# Deploy trading contract
stellar contract deploy --wasm target/wasm32v1-none/release/trading.wasm --source test-admin --network testnet

# Save the returned contract ID as TRADING_CONTRACT_ID
```

#### Deploy Funding Contract
```powershell
cd ../funding

# Deploy funding contract  
stellar contract deploy --wasm target/wasm32v1-none/release/funding.wasm --source test-admin --network testnet

# Save the returned contract ID as FUNDING_CONTRACT_ID
```

### 3. Initialize Contracts

Replace the contract IDs below with your actual deployed contract addresses:

#### Initialize Custom Assets Contract
```powershell
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- initialize --admin GABC123...DEF456
```

#### Initialize Trading Contract
```powershell
stellar contract invoke --id TRADING_CONTRACT_ID --source test-admin --network testnet -- initialize --fnft_contract FRACTCORE_CONTRACT_ID --xlm_contract CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQAMAEV2P6T9
```

#### Initialize Funding Contract
```powershell
stellar contract invoke --id FUNDING_CONTRACT_ID --source test-admin --network testnet -- initialize --admin GABC123...DEF456 --fnft_contract FRACTCORE_CONTRACT_ID
```

## Basic Usage Examples

### 1. Create a Fractionalized Asset

```powershell
# Create an asset with 1000 tokens (representing 100% ownership)
stellar contract invoke --id %FRACTCORE_CONTRACT_ID% --source test-admin --network testnet -- mint --to $(stellar keys address test-admin) --num_tokens 1000

# The function returns the new asset_id (e.g., 1)
set ASSET_ID=1
```

### 2. Distribute Tokens to Investors

```powershell
# Create additional user accounts for testing
stellar keys generate --global investor1
stellar keys generate --global investor2
stellar keys fund investor1 --network testnet
stellar keys fund investor2 --network testnet

# Distribute tokens: investor1 gets 300 (30%), investor2 gets 700 (70%)
stellar contract invoke --id %FRACTCORE_CONTRACT_ID% --source test-admin --network testnet -- mint_to --asset_id %ASSET_ID% --recipients [$(stellar keys address investor1),$(stellar keys address investor2)] --amounts [300,700]
```

### 3. Set Up Trading Between Users

```powershell
# Investor1 wants to sell 100 tokens to investor2 for 1000 XLM

# First, investor1 approves trading contract to transfer their tokens
stellar contract invoke --id %FRACTCORE_CONTRACT_ID% --source investor1 --network testnet -- approve --owner $(stellar keys address investor1) --operator %TRADING_CONTRACT_ID% --asset_id %ASSET_ID% --amount 100

# Create a sale proposal (both parties agree on terms)
stellar contract invoke --id %TRADING_CONTRACT_ID% --source investor1 --network testnet -- confirm_sale --seller $(stellar keys address investor1) --buyer $(stellar keys address investor2) --asset_id %ASSET_ID% --token_amount 100 --price 100000000 --duration_hours 24

# Investor2 completes the purchase (transfers XLM and receives tokens)
stellar contract invoke --id %TRADING_CONTRACT_ID% --source investor2 --network testnet -- finish_transaction --buyer $(stellar keys address investor2) --seller $(stellar keys address investor1) --asset_id %ASSET_ID%
```

### 4. Revenue Distribution

```powershell
# Property generates 10000 XLM in rental income
# Admin deposits funds to funding contract

# First, admin needs XLM in their account for deposit
# Transfer 10000 XLM worth (1 billion stroops) to funding contract
stellar contract invoke --id %FUNDING_CONTRACT_ID% --source test-admin --network testnet -- deposit_funds --depositor $(stellar keys address test-admin) --asset_id %ASSET_ID% --amount 1000000000

# Distribute the funds proportionally to token holders
stellar contract invoke --id %FUNDING_CONTRACT_ID% --source test-admin --network testnet -- distribute_funds --caller $(stellar keys address test-admin) --asset_id %ASSET_ID% --amount 1000000000 --description "Monthly rental income"

# Results:
# - investor1 (300 tokens = 30%) receives 300,000,000 stroops (3000 XLM)
# - investor2 (700 tokens = 70%) receives 700,000,000 stroops (7000 XLM)
```

### 5. Query Contract State

```powershell
# Check token balances
stellar contract invoke --id %FRACTCORE_CONTRACT_ID% --source test-admin --network testnet -- balance_of --owner $(stellar keys address investor1) --asset_id %ASSET_ID%

# Check available funds for distribution
stellar contract invoke --id %FUNDING_CONTRACT_ID% --source test-admin --network testnet -- asset_funds --asset_id %ASSET_ID%

# View trading history
stellar contract invoke --id %TRADING_CONTRACT_ID% --source test-admin --network testnet -- get_trade_count
```

```powershell
# Check if asset exists
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- asset_exists --asset_id 1

# Check total supply of the asset
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- asset_supply --asset_id 1

# Get list of asset owners
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- asset_owners --asset_id 1
```

### 4. Trading Operations

#### Confirm Sale (Create Sale Proposal)
```powershell
# Seller creates a sale proposal
stellar contract invoke --id TRADING_CONTRACT_ID --source seller-key --network testnet -- confirm_sale --seller GSELLER123...ABC456 --buyer GBUYER789...XYZ012 --asset_id 1 --token_amount 100 --price 50000000 --duration 86400

# Parameters:
# - seller: Seller's address
# - buyer: Buyer's address  
# - asset_id: ID of the fractional custom asset
# - token_amount: Number of tokens to sell
# - price: Price in stroops (1 XLM = 10,000,000 stroops)
# - duration: Sale duration in seconds
```

#### Finish Transaction (Execute Sale)
```powershell
# Buyer executes the sale
stellar contract invoke --id TRADING_CONTRACT_ID --source buyer-key --network testnet -- finish_transaction --buyer GBUYER789...XYZ012 --seller GSELLER123...ABC456 --asset_id 1

# This will:
# 1. Transfer tokens from seller to buyer
# 2. Transfer XLM from buyer to seller
# 3. Clear the sale proposal
```

### 5. Funding Operations

#### Deposit Funds to Asset
```powershell
# Admin deposits 10 XLM to asset_id 1
stellar contract invoke --id FUNDING_CONTRACT_ID --source test-admin --network testnet -- deposit_funds --asset_id 1 --amount 100000000

# Amount is in stroops (10 XLM = 100,000,000 stroops)
```

#### Distribute Funds to Token Holders
```powershell
# Distribute 5 XLM proportionally to all token holders of asset_id 1
stellar contract invoke --id FUNDING_CONTRACT_ID --source test-admin --network testnet -- distribute_funds --caller GABC123...DEF456 --asset_id 1 --amount 50000000 --description "Quarterly dividend distribution"
```

#### Check Asset Funding Status
```powershell
# Check available funds for an asset
stellar contract invoke --id FUNDING_CONTRACT_ID --source test-admin --network testnet -- get_asset_funds --asset_id 1

# Check total distributed amount
stellar contract invoke --id FUNDING_CONTRACT_ID --source test-admin --network testnet -- get_total_distributed --asset_id 1
```

## Advanced Operations

### Transfer Tokens Between Users
```powershell
# Transfer 50 tokens of asset_id 1 from one user to another
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source sender-key --network testnet -- transfer_from --operator GSENDER123...ABC456 --from GSENDER123...ABC456 --to GRECIPIENT789...XYZ012 --asset_id 1 --amount 50
```

### Set Allowances for Trading
```powershell
# Allow trading contract to transfer tokens on behalf of user
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source owner-key --network testnet -- approve --owner GOWNER123...ABC456 --operator TRADING_CONTRACT_ID --asset_id 1 --amount 100
```

### Emergency Functions
```powershell
# Emergency reset allowance (if needed)
stellar contract invoke --id TRADING_CONTRACT_ID --source seller-key --network testnet -- emergency_reset_allowance --seller GSELLER123...ABC456 --asset_id 1
```

## Environment Variables

For easier management, you can set these environment variables:

```powershell
# PowerShell
$env:FRACTCORE_CONTRACT = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
$env:TRADING_CONTRACT = "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"  
$env:FUNDING_CONTRACT = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"
$env:ADMIN_ADDRESS = "GABC123...DEF456"
```

Then use them in commands:
```powershell
stellar contract invoke --id $env:FRACTCORE_CONTRACT --source test-admin --network testnet -- balance_of --owner $env:ADMIN_ADDRESS --asset_id 1
```

## Troubleshooting

### Common Issues

#### 1. "Account not found" Error
```powershell
# Fund your account with testnet XLM
stellar keys fund test-admin --network testnet
```

#### 2. "Contract not found" Error
- Verify contract IDs are correct
- Ensure contracts are deployed to testnet
- Check network configuration

#### 3. "Insufficient Balance" Error
- Check XLM balance: `stellar account --address YOUR_ADDRESS --network testnet`
- Fund account if needed: `stellar keys fund YOUR_KEY --network testnet`

#### 4. Build Errors
```powershell
# Clean and rebuild
cargo clean
cargo build --release

# Update Rust if needed
rustup update

# If you see "WASM target not installed" error:
rustup target add wasm32-unknown-unknown
```

## Project Status

### ✅ **Production Ready Components**

**Fractional NFT Contract (`contracts/fractcore/`)**
- Complete ERC-1155 compatible implementation
- Efficient ownership tracking and management
- Cross-contract integration interfaces
- Comprehensive test coverage

**Trading Contract (`contracts/trading/`)**
- Peer-to-peer trading with confirmed sales
- Real XLM settlements via SAC interface
- Complete trading history and analytics
- Emergency functions and security controls

**Funding Contract (`contracts/funding/`)**
- Real XLM custody and distribution
- Proportional revenue sharing
- Democratic and admin-controlled distributions
- Integration with F-NFT ownership data

### 🚀 **Key Achievements**

1. **Complete Ecosystem**: All three contracts work seamlessly together
2. **Real XLM Operations**: Actual XLM transfers, not placeholder implementations
3. **Stellar Native**: Built specifically for Stellar's capabilities and advantages
4. **Production Grade**: Comprehensive error handling, security controls, and testing
5. **Cross-Contract Integration**: Efficient data sharing between contracts

### 📊 **Platform Capabilities**

| Feature | Status | Description |
|---------|---------|-------------|
| Asset Fractionalization | ✅ Production | Create any number of tokens per asset |
| Token Trading | ✅ Production | P2P trading with XLM settlements |
| Revenue Distribution | ✅ Production | Proportional XLM distribution to holders |
| Cross-Contract Integration | ✅ Production | Seamless data sharing between contracts |
| Batch Operations | ✅ Production | Efficient multi-recipient operations |
| Trading Analytics | ✅ Production | Complete trade history and statistics |
| Emergency Controls | ✅ Production | Admin functions for exceptional situations |
| Real XLM Support | ✅ Production | Native Stellar asset integration |

## Use Cases

### 1. **Real Estate Fractionalization**
```
Commercial Property → 10,000 F-NFT tokens → Trading on platform
Monthly rent revenue → Automatic XLM distribution to token holders
```

### 2. **Gaming Team Investment**
```
Esports Team → 1,000 F-NFT tokens → Fan ownership
Tournament winnings + sponsorships → Revenue sharing via XLM
```

### 3. **Art & Collectibles**
```
High-value artwork → 500 F-NFT tokens → Democratized ownership
Gallery revenue + appreciation → Proportional returns in XLM
```

### 4. **Revenue-Generating Assets**
```
Any income-producing asset → Fractional tokens → Liquid trading
Ongoing revenue streams → Automated distribution to stakeholders
```

## Technical Architecture

### Contract Interaction Pattern
```
User Actions → F-NFT Contract (ownership) ↔ Trading Contract (exchange)
                    ↓
Revenue Sources → Funding Contract (distribution) → Token Holders
```

### Data Flow
1. **Asset Creation**: F-NFT contract mints tokens representing ownership
2. **Trading**: Trading contract facilitates token-for-XLM exchanges
3. **Revenue Collection**: Funding contract receives XLM from asset operations
4. **Distribution**: Funding contract queries F-NFT for ownership data and distributes XLM

### Storage Efficiency
- **O(1) Balance Lookups**: Direct key-value access for balances
- **Automatic Cleanup**: Zero-balance holders removed automatically
- **Efficient Iteration**: Optimized owner lists for distribution calculations
- **Cross-Contract Queries**: Minimal gas cost for integration operations

## Future Enhancements

### Phase 1: Governance Integration
- Token holder voting on asset management decisions
- Proposal submission and execution system
- Democratic control over major asset decisions

### Phase 2: Advanced Features
- Batch distribution optimization for large holder lists
- Gaming team specific SAC implementation
- Enhanced analytics and reporting dashboard
- Mobile-friendly user interfaces

### Phase 3: Ecosystem Expansion
- Multi-asset portfolio management
- Sponsor integration for gaming teams
- NFT marketplace integration
- DeFi protocol connections

## Development Team

For questions, contributions, or support:

- **GitHub**: [Koltena Stellar Repository](https://github.com/your-username/koltena-stellar)
- **Documentation**: Each contract includes comprehensive README files
- **Testing**: Full test suites in each contract directory
- **Deployment**: Production-ready for Stellar testnet and mainnet

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## Acknowledgments

- Built on Stellar blockchain and Soroban smart contract platform
- Inspired by the need for democratized asset ownership
- Designed for real-world utility and adoption

---

**Koltena on Stellar**: Democratizing ownership through fractional NFTs, enabling everyone to participate in valuable asset ownership and revenue sharing.

#### 6. Missing WASM Target
```powershell
# Add the required WASM target
rustup target add wasm32-unknown-unknown
```

### Getting Help

- **Stellar Documentation**: https://developers.stellar.org/docs
- **Soroban Documentation**: https://soroban.stellar.org/docs
- **Discord**: Join the Stellar Developer Discord

### Testing

Run the test suite:
```powershell
# Test all contracts
cargo test

# Test specific contract
cd contracts/fractcore
cargo test
```

```
├── contracts
│   └── fractcore
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
│   └── funding
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
│   └── trading
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `fnft` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.