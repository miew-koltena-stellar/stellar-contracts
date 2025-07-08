use soroban_sdk::{panic_with_error, Address, Env};

use crate::contract::{GovernanceError, Vote};
use crate::events;
use crate::methods::{polls, utils};
use crate::storage;

pub fn vote(
    env: &Env,
    voter: &Address,
    poll_id: u32,
    option_index: u32,
) -> Result<(), GovernanceError> {
    voter.require_auth();

    let mut poll = storage::get_poll(env, poll_id).ok_or(GovernanceError::PollNotFound)?;

    if !poll.is_active {
        panic_with_error!(env, GovernanceError::PollNotActive);
    }

    if env.ledger().timestamp() >= poll.end_time {
        panic_with_error!(env, GovernanceError::PollExpired);
    }

    if option_index >= poll.options.len() || option_index > 1 {
        panic_with_error!(env, GovernanceError::InvalidOption);
    }

    if poll.votes.contains_key(voter.clone()) {
        panic_with_error!(env, GovernanceError::AlreadyVoted);
    }

    let fractcore_contract = storage::get_fractcore_contract(env);
    let voting_power =
        utils::call_fractcore_balance(env, &fractcore_contract, voter, poll.asset_id)
            .map_err(|_| GovernanceError::CrossContractCallFailed)?;

    if voting_power == 0 {
        panic_with_error!(env, GovernanceError::InsufficientVotingPower);
    }

    let vote = Vote {
        voter: voter.clone(),
        option_index,
        voting_power,
        timestamp: env.ledger().timestamp(),
    };

    poll.votes.set(voter.clone(), vote);
    poll.total_voters += 1;

    storage::set_poll(env, poll_id, &poll);

    events::emit_vote_cast(env, poll_id, voter, option_index, voting_power);

    polls::check_and_execute_poll(env, poll_id)?;

    Ok(())
}

pub fn can_vote(env: &Env, voter: &Address, poll_id: u32) -> Result<bool, GovernanceError> {
    let poll = storage::get_poll(env, poll_id).ok_or(GovernanceError::PollNotFound)?;

    if !poll.is_active || env.ledger().timestamp() >= poll.end_time {
        return Ok(false);
    }

    if poll.votes.contains_key(voter.clone()) {
        return Ok(false);
    }

    let fractcore_contract = storage::get_fractcore_contract(env);
    let voting_power =
        utils::call_fractcore_balance(env, &fractcore_contract, voter, poll.asset_id)?;

    Ok(voting_power > 0)
}
