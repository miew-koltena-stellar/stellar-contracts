use soroban_sdk::{panic_with_error, Address, Env, Map, String, Vec};

use crate::contract::{GovernanceError, Poll, PollAction};
use crate::events;
use crate::methods::utils;
use crate::storage;

pub fn create_poll(
    env: &Env,
    caller: &Address,
    asset_id: u64,
    title: &String,
    description: &String,
    action: &PollAction,
    duration_days: Option<u32>,
) -> Result<u32, GovernanceError> {
    caller.require_auth();

    let fractcore_contract = storage::get_fractcore_contract(env);
    let balance = utils::call_fractcore_balance(env, &fractcore_contract, caller, asset_id)?;
    let admin = storage::get_admin(env);

    if balance == 0 && *caller != admin {
        panic_with_error!(env, GovernanceError::InsufficientVotingPower);
    }

    let mut options = Vec::new(env);
    options.push_back(String::from_str(env, "Deny"));
    options.push_back(String::from_str(env, "Approve"));

    let params = storage::get_governance_params(env);
    let duration = duration_days.unwrap_or(params.default_expiry_days);

    if duration == 0 || duration > 365 {
        panic_with_error!(env, GovernanceError::InvalidDuration);
    }

    let poll_id = storage::get_next_poll_id(env);
    let end_time = env.ledger().timestamp() + (duration as u64 * 24 * 60 * 60);

    let poll = Poll {
        id: poll_id,
        asset_id,
        creator: caller.clone(),
        title: title.clone(),
        description: description.clone(),
        options,
        action: action.clone(),
        start_time: env.ledger().timestamp(),
        end_time,
        is_active: true,
        votes: Map::new(env),
        total_voters: 0,
    };

    storage::set_poll(env, poll_id, &poll);
    storage::add_asset_poll(env, asset_id, poll_id);
    storage::add_active_poll(env, poll_id);
    storage::increment_poll_counter(env);

    events::emit_poll_created(env, poll_id, asset_id, caller);

    Ok(poll_id)
}

pub fn check_and_execute_poll(env: &Env, poll_id: u32) -> Result<bool, GovernanceError> {
    let mut poll = storage::get_poll(env, poll_id).ok_or(GovernanceError::PollNotFound)?;

    if !poll.is_active {
        return Ok(false);
    }

    let current_time = env.ledger().timestamp();
    let fractcore_contract = storage::get_fractcore_contract(env);
    let total_asset_owners =
        utils::call_fractcore_owner_count(env, &fractcore_contract, poll.asset_id)?;

    let time_expired = current_time >= poll.end_time;
    let all_owners_voted = poll.total_voters >= total_asset_owners;
    let can_execute = time_expired || all_owners_voted;

    if !can_execute {
        return Ok(false);
    }

    let (winning_option, vote_counts) = utils::calculate_vote_results(env, &poll)?;
    let params = storage::get_governance_params(env);

    let execution_result = utils::check_execution_criteria(env, &poll, &vote_counts, &params)?;

    poll.is_active = false;
    storage::set_poll(env, poll_id, &poll);
    storage::remove_active_poll(env, poll_id);

    if execution_result.should_execute {
        let governance_contract = env.current_contract_address();
        utils::execute_poll_action(env, &poll.action, poll.asset_id, &governance_contract)?;

        events::emit_poll_executed(
            env,
            poll_id,
            winning_option,
            execution_result.approval_percentage,
            &poll.action,
        );
    } else {
        events::emit_poll_rejected(env, poll_id, execution_result.approval_percentage);
    }

    Ok(execution_result.should_execute)
}
