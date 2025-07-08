#[cfg(test)]
mod edge_case_tests {
    use crate::contract::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn create_test_env() -> Env {
        Env::default()
    }

    fn setup_governance_contract(env: &Env) -> (Address, Address, Address, Address) {
        let contract_id = env.register(GovernanceContract, ());
        let admin = Address::generate(env);
        let fractcore_contract = Address::generate(env);
        let funding_contract = Address::generate(env);

        let client = GovernanceContractClient::new(env, &contract_id);
        client.initialize(
            &admin,
            &fractcore_contract,
            &funding_contract,
            &60u32, // default threshold
            &40u32, // default quorum
            &7u32,  // default expiry days
        );

        (contract_id, admin, fractcore_contract, funding_contract)
    }

    // Edge Cases - Zero Values and Boundary Conditions
    #[test]
    fn test_zero_amount_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test creating a poll with zero distribution amount
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Zero Distribution Test"),
            &String::from_str(&env, "Testing zero amount distribution"),
            &PollAction::DistributeFunds(0u128, String::from_str(&env, "Zero amount test")),
            &Some(7),
        );

        let voter = Address::generate(&env);
        client.vote(&voter, &poll_id, &1u32);

        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::DistributeFunds(amount, _) => {
                assert_eq!(amount, 0u128);
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_maximum_value_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test creating a poll with maximum u128 value
        let max_amount = u128::MAX;
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Maximum Distribution Test"),
            &String::from_str(&env, "Testing maximum amount distribution"),
            &PollAction::DistributeFunds(max_amount, String::from_str(&env, "Max amount test")),
            &Some(7),
        );

        let voter = Address::generate(&env);
        client.vote(&voter, &poll_id, &1u32);

        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::DistributeFunds(amount, _) => {
                assert_eq!(amount, max_amount);
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_single_voter_scenarios() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with only one voter approving
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Single Voter Test"),
            &String::from_str(&env, "Only one person voting"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let lone_voter = Address::generate(&env);
        client.vote(&lone_voter, &poll_id, &1u32);

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 1);
        assert_eq!(results.winning_option, 1); // Approve wins
        assert_eq!(results.vote_counts.get(1).unwrap(), 1000); // Single approve vote
        assert_eq!(results.vote_counts.get(0).unwrap(), 0); // No deny votes

        let execution = client.check_poll_execution(&poll_id);
        assert_eq!(execution.approval_percentage, 100); // 100% approval with 1 voter
    }

    #[test]
    fn test_tied_vote_scenarios() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with tied votes (same number of approve/deny)
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Tied Vote Test"),
            &String::from_str(&env, "Testing tied voting scenario"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);

        client.vote(&voter1, &poll_id, &1u32); // Approve
        client.vote(&voter2, &poll_id, &0u32); // Deny

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 2);
        assert_eq!(results.vote_counts.get(1).unwrap(), 1000); // 1 approve
        assert_eq!(results.vote_counts.get(0).unwrap(), 1000); // 1 deny
        assert_eq!(results.winning_option, 0); // Deny wins in ties (approve needs MORE votes)

        let execution = client.check_poll_execution(&poll_id);
        assert!(!execution.should_execute); // Tied votes should not execute
        assert_eq!(execution.approval_percentage, 50); // 50% approval
    }

    #[test]
    fn test_empty_string_descriptions() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with empty strings
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, ""),
            &String::from_str(&env, ""),
            &PollAction::DistributeFunds(1000, String::from_str(&env, "")),
            &Some(7),
        );

        let poll = client.get_poll(&poll_id);
        assert_eq!(poll.title, String::from_str(&env, ""));
        assert_eq!(poll.description, String::from_str(&env, ""));

        match poll.action {
            PollAction::DistributeFunds(amount, description) => {
                assert_eq!(amount, 1000);
                assert_eq!(description, String::from_str(&env, ""));
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_very_long_descriptions() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with very long strings
        let long_title = "A".repeat(1000);
        let long_description = "B".repeat(2000);
        let long_action_desc = "C".repeat(500);

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, &long_title),
            &String::from_str(&env, &long_description),
            &PollAction::DistributeFunds(1000, String::from_str(&env, &long_action_desc)),
            &Some(7),
        );

        let poll = client.get_poll(&poll_id);
        assert_eq!(poll.title.len(), 1000);
        assert_eq!(poll.description.len(), 2000);

        match poll.action {
            PollAction::DistributeFunds(_, description) => {
                assert_eq!(description.len(), 500);
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_minimum_expiry_duration() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with minimum duration (1 day)
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Minimum Duration Test"),
            &String::from_str(&env, "Testing minimum 1-day duration"),
            &PollAction::NoExecution,
            &Some(1),
        );

        let poll = client.get_poll(&poll_id);
        // Duration should be at least 1 day (86400 seconds)
        assert!(poll.end_time > poll.start_time);
        let duration = poll.end_time - poll.start_time;
        assert!(duration >= 86400); // At least 1 day in seconds
    }

    #[test]
    fn test_maximum_expiry_duration() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with very long duration
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Maximum Duration Test"),
            &String::from_str(&env, "Testing very long duration"),
            &PollAction::NoExecution,
            &Some(365), // 1 year
        );

        let poll = client.get_poll(&poll_id);
        let duration = poll.end_time - poll.start_time;
        assert_eq!(duration, 365 * 86400); // 365 days in seconds
    }

    // Quorum and Threshold Edge Cases
    #[test]
    fn test_barely_meets_threshold() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Set threshold to 60%, so we need exactly 60% to pass
        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Threshold Edge Test"),
            &String::from_str(&env, "Testing exact threshold boundary"),
            &PollAction::NoExecution,
            &Some(7),
        );

        // 3 approve, 2 deny = 60% approval (exactly meets 60% threshold)
        let voters = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        client.vote(&voters[0], &poll_id, &1u32); // Approve
        client.vote(&voters[1], &poll_id, &1u32); // Approve
        client.vote(&voters[2], &poll_id, &1u32); // Approve
        client.vote(&voters[3], &poll_id, &0u32); // Deny
        client.vote(&voters[4], &poll_id, &0u32); // Deny

        let execution = client.check_poll_execution(&poll_id);
        assert_eq!(execution.approval_percentage, 60); // Exactly 60%
        assert!(execution.should_execute); // Should execute at exactly 60%
    }

    #[test]
    fn test_just_below_threshold() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Below Threshold Test"),
            &String::from_str(&env, "Testing just below threshold"),
            &PollAction::NoExecution,
            &Some(7),
        );

        // 2 approve, 3 deny = 40% approval (below 60% threshold)
        let voters = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        client.vote(&voters[0], &poll_id, &1u32); // Approve
        client.vote(&voters[1], &poll_id, &1u32); // Approve
        client.vote(&voters[2], &poll_id, &0u32); // Deny
        client.vote(&voters[3], &poll_id, &0u32); // Deny
        client.vote(&voters[4], &poll_id, &0u32); // Deny

        let execution = client.check_poll_execution(&poll_id);
        assert_eq!(execution.approval_percentage, 40); // 40% approval
        assert!(!execution.should_execute); // Should NOT execute below 60%
    }

    // Asset ID Edge Cases
    #[test]
    fn test_zero_asset_id() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with asset ID 0
        let poll_id = client.create_poll(
            &admin,
            &0u64,
            &String::from_str(&env, "Asset Zero Test"),
            &String::from_str(&env, "Testing asset ID zero"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let poll = client.get_poll(&poll_id);
        assert_eq!(poll.asset_id, 0u64);
    }

    #[test]
    fn test_maximum_asset_id() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Test with maximum u64 asset ID
        let max_asset_id = u64::MAX;
        let poll_id = client.create_poll(
            &admin,
            &max_asset_id,
            &String::from_str(&env, "Max Asset Test"),
            &String::from_str(&env, "Testing maximum asset ID"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let poll = client.get_poll(&poll_id);
        assert_eq!(poll.asset_id, max_asset_id);
    }

    // Multiple Concurrent Polls Edge Cases
    #[test]
    fn test_multiple_polls_same_asset() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Create multiple polls for the same asset
        let asset_id = 1u64;

        let poll_id1 = client.create_poll(
            &admin,
            &asset_id,
            &String::from_str(&env, "Poll 1"),
            &String::from_str(&env, "First poll for asset 1"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let poll_id2 = client.create_poll(
            &admin,
            &asset_id,
            &String::from_str(&env, "Poll 2"),
            &String::from_str(&env, "Second poll for asset 1"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let poll_id3 = client.create_poll(
            &admin,
            &asset_id,
            &String::from_str(&env, "Poll 3"),
            &String::from_str(&env, "Third poll for asset 1"),
            &PollAction::NoExecution,
            &Some(7),
        );

        // All polls should be created with different IDs
        assert_ne!(poll_id1, poll_id2);
        assert_ne!(poll_id2, poll_id3);
        assert_ne!(poll_id1, poll_id3);

        // Verify all polls exist
        let poll1 = client.get_poll(&poll_id1);
        let poll2 = client.get_poll(&poll_id2);
        let poll3 = client.get_poll(&poll_id3);

        assert_eq!(poll1.asset_id, asset_id);
        assert_eq!(poll2.asset_id, asset_id);
        assert_eq!(poll3.asset_id, asset_id);
    }

    #[test]
    fn test_voting_on_multiple_polls() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Create multiple polls
        let poll_id1 = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Multi Poll Test 1"),
            &String::from_str(&env, "First poll"),
            &PollAction::NoExecution,
            &Some(7),
        );

        let poll_id2 = client.create_poll(
            &admin,
            &2u64,
            &String::from_str(&env, "Multi Poll Test 2"),
            &String::from_str(&env, "Second poll"),
            &PollAction::NoExecution,
            &Some(7),
        );

        // Same voter should be able to vote on different polls
        let voter = Address::generate(&env);
        client.vote(&voter, &poll_id1, &1u32); // Approve poll 1
        client.vote(&voter, &poll_id2, &0u32); // Deny poll 2

        let results1 = client.get_vote_results(&poll_id1);
        let results2 = client.get_vote_results(&poll_id2);

        assert_eq!(results1.vote_counts.get(1).unwrap(), 1000); // Approve vote
        assert_eq!(results2.vote_counts.get(0).unwrap(), 1000); // Deny vote
    }
}
