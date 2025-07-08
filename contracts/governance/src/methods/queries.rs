use soroban_sdk::{Env, Vec};

use crate::contract::{GovernanceError, GovernanceParams, Poll, VoteResults};
use crate::methods::utils;
use crate::storage;

pub fn get_poll(env: &Env, poll_id: u32) -> Result<Poll, GovernanceError> {
    storage::get_poll(env, poll_id).ok_or(GovernanceError::PollNotFound)
}

pub fn get_asset_polls(env: &Env, asset_id: u64) -> Vec<u32> {
    storage::get_asset_polls(env, asset_id)
}

pub fn get_active_polls(env: &Env) -> Vec<u32> {
    storage::get_active_polls(env)
}

pub fn get_vote_results(env: &Env, poll_id: u32) -> Result<VoteResults, GovernanceError> {
    let poll = storage::get_poll(env, poll_id).ok_or(GovernanceError::PollNotFound)?;
    let (winning_option, vote_counts) = utils::calculate_vote_results(env, &poll)?;

    Ok(VoteResults {
        poll_id,
        vote_counts,
        winning_option,
        total_voters: poll.total_voters,
        is_finalized: !poll.is_active,
    })
}

pub fn get_governance_params(env: &Env) -> GovernanceParams {
    storage::get_governance_params(env)
}
