use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, Map, String, Vec,
};

use crate::methods::{admin, polls, queries, utils, voting};
use crate::storage;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidParameters = 4,
    PollNotFound = 5,
    PollNotActive = 6,
    PollExpired = 7,
    AlreadyVoted = 8,
    InsufficientVotingPower = 9,
    InvalidOption = 10,
    InvalidOptions = 11,
    InvalidDuration = 12,
    CannotExecuteYet = 13,
    CrossContractCallFailed = 14,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PollAction {
    NoExecution,
    DistributeFunds(u128, String),
    TransferTokens(Address, u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Poll {
    pub id: u32,
    pub asset_id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub action: PollAction,
    pub start_time: u64,
    pub end_time: u64,
    pub is_active: bool,
    pub votes: Map<Address, Vote>,
    pub total_voters: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Vote {
    pub voter: Address,
    pub option_index: u32,
    pub voting_power: u64,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GovernanceParams {
    pub threshold_percentage: u32,
    pub quorum_percentage: u32,
    pub default_expiry_days: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct VoteResults {
    pub poll_id: u32,
    pub vote_counts: Vec<u64>,
    pub winning_option: u32,
    pub total_voters: u32,
    pub is_finalized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ExecutionResult {
    pub should_execute: bool,
    pub approval_percentage: u32,
    pub participation_percentage: u32,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        fractcore_contract: Address,
        funding_contract: Address,
        default_threshold: u32,
        default_quorum: u32,
        default_expiry_days: u32,
    ) -> Result<(), GovernanceError> {
        admin::initialize(
            &env,
            &admin,
            &fractcore_contract,
            &funding_contract,
            default_threshold,
            default_quorum,
            default_expiry_days,
        )
    }

    pub fn create_poll(
        env: Env,
        caller: Address,
        asset_id: u64,
        title: String,
        description: String,
        action: PollAction,
        duration_days: Option<u32>,
    ) -> Result<u32, GovernanceError> {
        polls::create_poll(
            &env,
            &caller,
            asset_id,
            &title,
            &description,
            &action,
            duration_days,
        )
    }

    pub fn vote(
        env: Env,
        voter: Address,
        poll_id: u32,
        option_index: u32,
    ) -> Result<(), GovernanceError> {
        voting::vote(&env, &voter, poll_id, option_index)
    }

    /// Update governance parameters (admin only)
    pub fn update_governance_params(
        env: Env,
        caller: Address,
        threshold_percentage: u32,
        quorum_percentage: u32,
        default_expiry_days: u32,
    ) -> Result<(), GovernanceError> {
        admin::update_governance_params(
            &env,
            &caller,
            threshold_percentage,
            quorum_percentage,
            default_expiry_days,
        )
    }

    /// Check and execute poll if conditions are met
    pub fn check_and_execute_poll(env: Env, poll_id: u32) -> Result<bool, GovernanceError> {
        polls::check_and_execute_poll(&env, poll_id)
    }

    /// Admin function to update governance parameters
    pub fn set_governance_params(
        env: Env,
        admin: Address,
        new_params: GovernanceParams,
    ) -> Result<(), GovernanceError> {
        admin::set_governance_params(&env, &admin, &new_params)
    }

    pub fn get_poll(env: Env, poll_id: u32) -> Result<Poll, GovernanceError> {
        queries::get_poll(&env, poll_id)
    }

    pub fn get_asset_polls(env: Env, asset_id: u64) -> Vec<u32> {
        queries::get_asset_polls(&env, asset_id)
    }

    pub fn get_active_polls(env: Env) -> Vec<u32> {
        queries::get_active_polls(&env)
    }

    pub fn get_vote_results(env: Env, poll_id: u32) -> Result<VoteResults, GovernanceError> {
        queries::get_vote_results(&env, poll_id)
    }

    pub fn get_governance_params(env: Env) -> GovernanceParams {
        queries::get_governance_params(&env)
    }

    pub fn can_vote(env: Env, voter: Address, poll_id: u32) -> Result<bool, GovernanceError> {
        voting::can_vote(&env, &voter, poll_id)
    }

    /// Check poll execution criteria without executing
    pub fn check_poll_execution(
        env: Env,
        poll_id: u32,
    ) -> Result<ExecutionResult, GovernanceError> {
        let poll = storage::get_poll(&env, poll_id).ok_or(GovernanceError::PollNotFound)?;
        let params = storage::get_governance_params(&env);
        let (_, vote_counts) = utils::calculate_vote_results(&env, &poll)?;
        utils::check_execution_criteria(&env, &poll, &vote_counts, &params)
    }
}
