use soroban_sdk::{Address, Env, String};

use crate::contract::PollAction;

// Event topics
const POLL_CREATED: &str = "poll_created";
const VOTE_CAST: &str = "vote_cast";
const POLL_EXECUTED: &str = "poll_executed";
const POLL_REJECTED: &str = "poll_rejected";
const PARAMS_UPDATED: &str = "params_updated";

pub fn emit_poll_created(env: &Env, poll_id: u32, asset_id: u64, creator: &Address) {
    env.events().publish(
        (String::from_str(env, POLL_CREATED),),
        (poll_id, asset_id, creator),
    );
}

pub fn emit_vote_cast(
    env: &Env,
    poll_id: u32,
    voter: &Address,
    option_index: u32,
    voting_power: u64,
) {
    env.events().publish(
        (String::from_str(env, VOTE_CAST),),
        (poll_id, voter, option_index, voting_power),
    );
}

pub fn emit_poll_executed(
    env: &Env,
    poll_id: u32,
    winning_option: u32,
    approval_percentage: u32,
    _action: &PollAction,
) {
    env.events().publish(
        (String::from_str(env, POLL_EXECUTED),),
        (poll_id, winning_option, approval_percentage),
    );
}

pub fn emit_poll_rejected(env: &Env, poll_id: u32, approval_percentage: u32) {
    env.events().publish(
        (String::from_str(env, POLL_REJECTED),),
        (poll_id, approval_percentage),
    );
}

pub fn emit_params_updated(env: &Env, threshold_percentage: u32, quorum_percentage: u32) {
    env.events().publish(
        (String::from_str(env, PARAMS_UPDATED),),
        (threshold_percentage, quorum_percentage),
    );
}
