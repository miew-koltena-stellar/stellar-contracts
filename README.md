# Koltena on Stellar: Fractional NFT Platform

🚨 **TESTNET ONLY** - This project is currently for testing purposes only. Do not use on mainnet.

Koltena is a comprehensive platform that democratizes ownership of assets through sophisticated tokenization, trading, and governance mechanisms built on Stellar/Soroban.
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
git clone https://github.com/your-username/koltena-stellar.git
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

## Contract Overview

Koltena consists of three main smart contracts:

1. **Custom Assets Contract** (`contracts/fractcore/`): Core fractional NFT functionality
2. **Trading Contract** (`contracts/trading/`): Peer-to-peer trading system  
3. **Funding Contract** (`contracts/funding/`): Fund distribution to token holders

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

#### Deploy Custom Assets Contract
```powershell
# Option 1: Deploy from contract directory (use relative path to root target)
cd contracts/fractcore
stellar contract deploy --wasm ../../target/wasm32v1-none/release/fractcore.wasm --source test-admin --network testnet

# Option 2: Deploy from project root (recommended)
cd ../..  # Go back to project root
stellar contract deploy --wasm target/wasm32v1-none/release/fractcore.wasm --source test-admin --network testnet

# Save the returned contract ID as FRACTCORE_CONTRACT_ID
# Example: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
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

## Basic Usage

### 1. Mint Fractional Custom Assets

Create a new fractional custom asset with 1000 tokens:

```powershell
# Mint 1000 tokens of a new asset to an address
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- mint --to GABC123...DEF456 --num_tokens 1000

# This returns an asset_id (e.g., 1) which represents your new fractional custom asset
```

### 2. Check Token Balance

```powershell
# Check balance of asset_id 1 for a specific address
stellar contract invoke --id FRACTCORE_CONTRACT_ID --source test-admin --network testnet -- balance_of --owner GABC123...DEF456 --asset_id 1
```

### 3. Check Asset Information

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

#### 5. Whitespace in Path Warning
If you see "Cargo home directory contains whitespace" warning, this is usually safe to ignore for development. The build will still work correctly.

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

.
├── contracts
│   └── fractcore
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│   └── funding
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│   └── trading
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `fnft` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.