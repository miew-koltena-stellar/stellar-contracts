use soroban_sdk::{panic_with_error, Address, Env};

use crate::contract::{GovernanceError, GovernanceParams};
use crate::events;
use crate::storage;

/// Initialize the governance contract
pub fn initialize(
    env: &Env,
    admin: &Address,
    fractcore_contract: &Address,
    funding_contract: &Address,
    default_threshold: u32,
    default_quorum: u32,
    default_expiry_days: u32,
) -> Result<(), GovernanceError> {
    if storage::is_initialized(env) {
        panic_with_error!(env, GovernanceError::AlreadyInitialized);
    }

    // Validate parameters: percentages must be <= 100, expiry between 1-365 days
    if default_threshold > 100 || default_quorum > 100 {
        panic_with_error!(env, GovernanceError::InvalidParameters);
    }

    if default_expiry_days == 0 || default_expiry_days > 365 {
        panic_with_error!(env, GovernanceError::InvalidParameters);
    }

    // Store contract references and admin address
    storage::set_admin(env, admin);
    storage::set_fractcore_contract(env, fractcore_contract);
    storage::set_funding_contract(env, funding_contract);

    // Set default governance parameters
    let params = GovernanceParams {
        threshold_percentage: default_threshold,
        quorum_percentage: default_quorum,
        default_expiry_days,
    };
    storage::set_governance_params(env, &params);

    storage::set_initialized(env);
    Ok(())
}

/// Set governance parameters using a GovernanceParams struct (admin only)
pub fn set_governance_params(
    env: &Env,
    admin: &Address,
    new_params: &GovernanceParams,
) -> Result<(), GovernanceError> {
    admin.require_auth();

    let stored_admin = storage::get_admin(env);
    if *admin != stored_admin {
        panic_with_error!(env, GovernanceError::Unauthorized);
    }

    // Percentages must be <= 100, expiry between 1-365 days
    if new_params.threshold_percentage > 100 || new_params.quorum_percentage > 100 {
        panic_with_error!(env, GovernanceError::InvalidParameters);
    }

    if new_params.default_expiry_days == 0 || new_params.default_expiry_days > 365 {
        panic_with_error!(env, GovernanceError::InvalidParameters);
    }

    storage::set_governance_params(env, new_params);

    events::emit_params_updated(
        env,
        new_params.threshold_percentage,
        new_params.quorum_percentage,
    );

    Ok(())
}

/// Admin function to update governance parameters
pub fn update_governance_params(
    env: &Env,
    admin: &Address,
    threshold_percentage: u32,
    quorum_percentage: u32,
    default_expiry_days: u32,
) -> Result<(), GovernanceError> {
    let params = GovernanceParams {
        threshold_percentage,
        quorum_percentage,
        default_expiry_days,
    };
    set_governance_params(env, admin, &params)
}
