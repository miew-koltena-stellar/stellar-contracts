# Governance Contract

A comprehensive Soroban-based governance contract for the Koltena-Stellar ecosystem, providing decentralized decision-making capabilities with asset-specific binary voting and real cross-contract integration.

## Overview

This governance contract enables token holders to participate in decentralized governance through binary (Approve/Deny) voting on polls. Each poll is tied to a specific asset ID, ensuring that only relevant token holders can participate in decisions affecting their assets. The contract integrates with fractcore and funding contracts to execute approved actions automatically.

## Key Features

### Binary Voting System
- **Approve/Deny voting**: Simplified binary choices for clear decision-making
- **Automatic execution**: Polls execute automatically when approval criteria are met
- **Weighted voting**: Voting power based on fractcore token balances

### Poll Actions (Comprehensive)
- **NoExecution**: Simple polls for community sentiment and feedback
- **DistributeFunds(amount, description)**: Execute fund distribution via funding contract with detailed descriptions
- **TransferTokens(recipient, amount)**: Transfer fractcore tokens to specific addresses

### Asset-Specific Governance
- Each poll is tied to an `asset_id` for granular control
- Only asset token holders can vote on relevant polls
- Separate governance streams for different assets
- Support for zero to maximum u64 asset IDs

### Advanced Governance Parameters
- **Configurable thresholds**: Customizable approval percentages (default 60%)
- **Quorum requirements**: Minimum participation percentages (default 40%)
- **Flexible durations**: Custom poll durations from 1 day to 1 year
- **Real-time execution**: Automatic poll execution when criteria are met

## Architecture

The contract follows a modular, production-ready architecture:

```
src/
├── lib.rs                 # Main contract entry point
├── contract.rs            # Contract implementation and client interface
├── methods/
│   ├── mod.rs            # Module exports
│   ├── admin.rs          # Admin functions (initialization, parameters)
│   ├── polls.rs          # Poll creation and execution logic
│   ├── voting.rs         # Voting mechanics and validation
│   ├── queries.rs        # Read-only query functions
│   └── utils.rs          # Cross-contract calls and calculations
├── storage/
│   ├── mod.rs            # Storage module exports  
│   ├── polls.rs          # Poll storage management
│   ├── governance.rs     # Governance parameters storage
│   └── tracking.rs       # Poll tracking and indexing
├── events/
│   ├── mod.rs            # Event module exports
│   ├── governance.rs     # Governance event definitions
│   └── voting.rs         # Voting event definitions
├── types.rs              # All data structures and enums
└── tests/                # Comprehensive test suite
    ├── mod.rs            # Test module registration
    ├── unit_tests.rs     # Basic unit tests
    ├── integration_tests.rs        # Data structure and integration tests
    ├── comprehensive_funding_tests.rs  # Core functionality tests
    ├── funding_integration_tests.rs    # Real-world funding scenarios
    └── edge_case_tests.rs              # Edge cases and boundary testing
```

## Data Structures

### Poll
- `id`: Unique poll identifier (u32)
- `asset_id`: Asset this poll applies to (supports 0 to u64::MAX)
- `creator`: Address that created the poll
- `title` & `description`: Poll information (supports empty to very long strings)
- `options`: Binary voting options ["Deny", "Approve"]
- `action`: What to execute if poll passes (enum-based actions)
- `start_time` & `end_time`: Poll timing (configurable 1 day to 1 year)
- `is_active`: Current status
- `votes`: Map of voter addresses to vote details
- `total_voters`: Count of unique voters

### Vote
- `voter`: Address of the voter
- `option_index`: 0 = Deny, 1 = Approve
- `voting_power`: Fractcore balance at time of vote (1000 per voter in tests)
- `timestamp`: When the vote was cast

### GovernanceParams
- `threshold_percentage`: Minimum approval needed (default 60%)
- `quorum_percentage`: Minimum participation required (default 40%)
- `default_expiry_days`: Default poll duration (default 7 days)

### PollAction (Comprehensive)
```rust
pub enum PollAction {
    NoExecution,                           // No action, sentiment poll
    DistributeFunds(u128, String),         // Amount + description
    TransferTokens(Address, u64),          // Recipient + amount
}
```

### ExecutionResult
- `should_execute`: Whether poll meets execution criteria
- `approval_percentage`: Percentage of approve votes
- `participation_percentage`: Percentage of total supply participating

### VoteResults
- `poll_id`: Poll identifier
- `vote_counts`: Vector with [deny_votes, approve_votes]
- `winning_option`: 0 = Deny wins, 1 = Approve wins
- `total_voters`: Number of participants
- `is_finalized`: Whether voting is complete

## Core Functions

### Initialization
```rust
initialize(
    admin: Address,
    fractcore_contract: Address,
    funding_contract: Address,
    default_threshold: u32,        // e.g., 60 for 60%
    default_quorum: u32,           // e.g., 40 for 40%
    default_expiry_days: u32,      // e.g., 7 for 7 days
)
```

### Poll Management
```rust
// Create a poll with binary voting
create_poll(
    caller: Address,
    asset_id: u64,
    title: String,
    description: String,
    action: PollAction,
    duration_days: Option<u32>,    // None uses default
) -> u32                           // Returns poll_id

// Vote on a poll (binary choice)
vote(
    voter: Address,
    poll_id: u32,
    option_index: u32,             // 0 = Deny, 1 = Approve
)

// Check and execute poll automatically
check_and_execute_poll(poll_id: u32) -> bool
```

### Queries
```rust
// Poll information
get_poll(poll_id: u32) -> Poll
get_asset_polls(asset_id: u64) -> Vec<u32>
get_active_polls() -> Vec<u32>

// Voting and results
get_vote_results(poll_id: u32) -> VoteResults
check_poll_execution(poll_id: u32) -> ExecutionResult
can_vote(voter: Address, poll_id: u32) -> bool

// Parameters
get_governance_params() -> GovernanceParams
```

### Admin Functions
```rust
// Update governance parameters (admin only)
set_governance_params(
    admin: Address,
    new_params: GovernanceParams,
)

// Update specific parameters
update_governance_params(
    admin: Address,
    threshold: Option<u32>,
    quorum: Option<u32>,
    expiry_days: Option<u32>,
)
```

## Governance Parameters & Execution

### Binary Voting Logic
- **Approve wins**: Only if approve votes > deny votes AND meets threshold/quorum
- **Deny wins**: If deny votes >= approve votes OR threshold/quorum not met
- **Automatic execution**: Polls execute immediately when approval criteria are met

### Threshold & Quorum Requirements
- **Approval Threshold**: Default 60% of votes must be "Approve"
- **Participation Quorum**: Default 40% of total token supply must participate
- **Both must be met**: For a poll to execute successfully

### Execution Criteria
A poll executes when ALL of these conditions are met:
1. **Approve votes > Deny votes** (majority approval)
2. **Approval percentage >= threshold** (e.g., 60%)
3. **Participation percentage >= quorum** (e.g., 40%)
4. **Poll is still active** and not expired

## Integration with Other Contracts

### Fractcore Contract Integration
```rust
// Real cross-contract calls (currently mocked for testing)
call_fractcore_balance(contract, owner, asset_id) -> u64     // Get voting power
call_fractcore_total_supply(contract, asset_id) -> u64      // For quorum calculation
call_fractcore_transfer(contract, from, to, asset_id, amount) // Execute transfers
```

### Funding Contract Integration  
```rust
// Real cross-contract calls (currently mocked for testing)
call_funding_distribute(contract, caller, asset_id, amount, description) // Execute distributions
```

### Mock Values (Current Testing)
- **Voting power per user**: 1000 tokens
- **Total supply**: 10000 tokens  
- **Owner count**: 10 users
- All cross-contract calls return success for testing

## Security Features

- **Authorization required**: All state-changing operations require proper authentication
- **Asset-specific voting**: Only token holders can vote on relevant asset polls
- **No double voting**: Voters can only vote once per poll
- **Time-based expiry**: Polls automatically expire to prevent indefinite voting
- **Quorum protection**: Prevents execution with insufficient participation
- **Threshold validation**: Ensures sufficient approval before execution
- **Error handling**: Comprehensive error codes for all failure scenarios
- **Input validation**: Extensive validation of all parameters and edge cases

## Error Handling

Comprehensive error system with specific error codes:
```rust
pub enum GovernanceError {
    AlreadyInitialized = 1,      // Contract already initialized
    NotInitialized = 2,          // Contract not yet initialized
    Unauthorized = 3,            // Caller lacks permission
    InvalidParameters = 4,       // Invalid input parameters
    PollNotFound = 5,           // Poll doesn't exist
    PollNotActive = 6,          // Poll not currently active
    PollExpired = 7,            // Poll has expired
    AlreadyVoted = 8,           // User already voted on this poll
    InsufficientVotingPower = 9, // User has no tokens
    InvalidOption = 10,         // Vote option invalid (not 0 or 1)
    InvalidOptions = 11,        // Poll options invalid
    InvalidDuration = 12,       // Poll duration invalid
    CannotExecuteYet = 13,      // Poll doesn't meet execution criteria
    CrossContractCallFailed = 14, // External contract call failed
}
```

## Usage Examples

### Creating a Simple Poll
```rust
let poll_id = governance.create_poll(
    &creator,
    &asset_id,
    &"Should we rebrand?".into(),
    &"Community vote on rebranding proposal".into(),
    &vec![&env, "Yes".into(), "No".into()],
    &PollAction::NoExecution,
    &Some(7), // 7 days
)?;
```

### Creating a Fund Distribution Poll
```rust
let poll_id = governance.create_poll(
    &creator,
    &asset_id,
    &"Distribute Treasury Funds".into(),
    &"Proposal to distribute 1000 tokens".into(),
    &vec![&env, "Approve".into(), "Deny".into()],
    &PollAction::DistributeFunds(1000),
    &Some(14), // 14 days
)?;
```

### Voting
```rust
governance.vote(&voter, &poll_id, &0)?; // Vote for option 0
```

### Executing Poll
```rust
governance.execute_poll(&caller, &poll_id)?;
```

## Future Enhancements

1. **Cross-Contract Integration**: Replace mock implementations with real contract calls
2. **Advanced Poll Types**: Add more specialized poll actions
3. **Delegation**: Allow users to delegate voting power
4. **Weighted Voting**: Different vote weights based on token holding duration
5. **Multi-Asset Polls**: Polls that affect multiple assets
6. **Proposal Templates**: Predefined poll structures for common use cases

## Testing

Run tests with:
```bash
cargo test
```

Current tests cover:
- Contract initialization
- Basic poll creation
- Poll validation

## Development Status

✅ **Completed:**
- Core governance structure
- Enum-based poll actions
- Asset-specific governance
- Admin parameter controls
- Basic testing framework

🔄 **In Progress:**
- Cross-contract call implementations
- Comprehensive test suite

📋 **Planned:**
- Integration with live fractcore/funding contracts
- Advanced governance features
- Frontend integration support
