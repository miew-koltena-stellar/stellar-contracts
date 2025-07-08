#[cfg(test)]
mod comprehensive_funding_tests {
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

    // Basic Functionality Tests
    #[test]
    fn test_initialization_and_basic_flow() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Basic Test Poll"),
            &String::from_str(&env, "Should we proceed?"),
            &PollAction::NoExecution,
            &None,
        );

        assert_eq!(poll_id, 1);

        let poll = client.get_poll(&poll_id);
        assert_eq!(poll.title, String::from_str(&env, "Basic Test Poll"));
        assert_eq!(poll.asset_id, 1u64);
        assert!(poll.is_active);
        assert_eq!(poll.options.len(), 2);
    }

    // Fund Distribution Scenarios
    #[test]
    fn test_tournament_prize_distribution_approve() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Tournament Prize Distribution"),
            &String::from_str(
                &env,
                "We won $10K! Should we distribute half to token holders?",
            ),
            &PollAction::DistributeFunds(
                5000,
                String::from_str(&env, "Tournament winnings distribution"),
            ),
            &Some(7),
        );

        let member1 = Address::generate(&env);
        let member2 = Address::generate(&env);
        let member3 = Address::generate(&env);

        client.vote(&member1, &poll_id, &1u32); // Approve
        client.vote(&member2, &poll_id, &1u32); // Approve
        client.vote(&member3, &poll_id, &1u32); // Approve

        // Check vote results
        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 3);
        assert_eq!(results.winning_option, 1); // Approve wins
        assert_eq!(results.vote_counts.get(1).unwrap(), 3000); // 3 approve votes

        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::DistributeFunds(amount, description) => {
                assert_eq!(amount, 5000);
                assert_eq!(
                    description,
                    String::from_str(&env, "Tournament winnings distribution")
                );
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_tournament_prize_distribution_deny() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Tournament Prize Distribution"),
            &String::from_str(&env, "Should we distribute all winnings to token holders?"),
            &PollAction::DistributeFunds(10000, String::from_str(&env, "Full tournament winnings")),
            &Some(7),
        );

        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let voter4 = Address::generate(&env);

        client.vote(&voter1, &poll_id, &0u32); // Deny
        client.vote(&voter2, &poll_id, &0u32); // Deny
        client.vote(&voter3, &poll_id, &0u32); // Deny
        client.vote(&voter4, &poll_id, &1u32); // Approve

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 4);
        assert_eq!(results.winning_option, 0); // Deny wins
        assert_eq!(results.vote_counts.get(0).unwrap(), 3000); // 3 deny votes
        assert_eq!(results.vote_counts.get(1).unwrap(), 1000); // 1 approve vote
    }

    #[test]
    fn test_sponsorship_revenue_sharing() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &2u64, // Different asset for sponsorship team
            &String::from_str(&env, "Sponsorship Revenue Share"),
            &String::from_str(&env, "We got $30K sponsorship. Share $20K with community?"),
            &PollAction::DistributeFunds(
                20000,
                String::from_str(&env, "Sponsorship revenue sharing"),
            ),
            &Some(5),
        );

        // Community votes overwhelmingly to approve
        let supporters = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        for supporter in &supporters {
            client.vote(supporter, &poll_id, &1u32); // All approve
        }

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 5);
        assert_eq!(results.winning_option, 1);
        assert_eq!(results.vote_counts.get(1).unwrap(), 5000); // All approve votes
    }

    // Emergency Transfer Scenarios
    #[test]
    fn test_emergency_medical_transfer() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let injured_member = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Emergency Medical Fund"),
            &String::from_str(&env, "Team member injured, need $5K for medical expenses"),
            &PollAction::TransferTokens(injured_member.clone(), 5000u64), // Use u64
            &Some(1),                                                     // Emergency - 1 day only
        );

        // Quick community response for emergency
        let responders = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        for responder in &responders {
            client.vote(responder, &poll_id, &1u32); // All approve emergency
        }

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 3);
        assert_eq!(results.winning_option, 1);

        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::TransferTokens(to, amount) => {
                assert_eq!(to, injured_member);
                assert_eq!(amount, 5000u64);
            }
            _ => panic!("Expected TransferTokens action"),
        }
    }

    #[test]
    fn test_training_investment_mixed_voting() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let training_facility = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &3u64,
            &String::from_str(&env, "Elite Training Bootcamp"),
            &String::from_str(&env, "Invest $8K in 2-week intensive training program?"),
            &PollAction::TransferTokens(training_facility.clone(), 8000u64),
            &Some(10),
        );

        // Mixed community response
        let performance_voters = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        let budget_voters = [Address::generate(&env), Address::generate(&env)];

        for voter in &performance_voters {
            client.vote(voter, &poll_id, &1u32); // Approve
        }

        // Budget-conscious voters deny
        for voter in &budget_voters {
            client.vote(voter, &poll_id, &0u32); // Deny
        }

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 5);
        assert_eq!(results.winning_option, 1); // Approve wins 3-2
        assert_eq!(results.vote_counts.get(0).unwrap(), 2000); // 2 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 3000); // 3 approve
    }

    // Equipment Purchase Scenarios
    #[test]
    fn test_gaming_equipment_upgrade() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let equipment_vendor = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &4u64,
            &String::from_str(&env, "Gaming Setup Upgrade"),
            &String::from_str(&env, "New monitors, keyboards, headsets - $6K total"),
            &PollAction::TransferTokens(equipment_vendor.clone(), 6000u64),
            &Some(5),
        );

        let team_players = [Address::generate(&env), Address::generate(&env)];

        let fans = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        for player in &team_players {
            client.vote(player, &poll_id, &1u32); // Approve
        }

        // Fans mostly support
        client.vote(&fans[0], &poll_id, &1u32); // Approve
        client.vote(&fans[1], &poll_id, &1u32); // Approve
        client.vote(&fans[2], &poll_id, &0u32); // Deny

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 5);
        assert_eq!(results.winning_option, 1); // Approve wins 4-1
    }

    // Rejected Proposal Scenarios
    #[test]
    fn test_luxury_expense_rejection() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &5u64,
            &String::from_str(&env, "Luxury Gaming House"),
            &String::from_str(&env, "Rent $15K/month luxury gaming house?"),
            &PollAction::DistributeFunds(
                15000,
                String::from_str(&env, "Monthly luxury house rental"),
            ),
            &Some(14),
        );

        let supporters = [Address::generate(&env), Address::generate(&env)];

        let budget_conscious = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        for supporter in &supporters {
            client.vote(supporter, &poll_id, &1u32); // Approve
        }

        for voter in &budget_conscious {
            client.vote(voter, &poll_id, &0u32); // Deny
        }

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 6);
        assert_eq!(results.winning_option, 0); // Deny wins 4-2
        assert_eq!(results.vote_counts.get(0).unwrap(), 4000); // 4 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 2000); // 2 approve
    }

    // Multi-stage Tournament Scenario
    #[test]
    fn test_tournament_entry_then_prize_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let tournament_organizer = Address::generate(&env);

        // Stage 1: Approve tournament entry fee
        let poll_id1 = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Tournament Entry Fee"),
            &String::from_str(&env, "Pay $2K entry fee for major tournament?"),
            &PollAction::TransferTokens(tournament_organizer.clone(), 2000u64),
            &Some(3),
        );

        // Community approves entry
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        client.vote(&voter1, &poll_id1, &1u32);
        client.vote(&voter2, &poll_id1, &1u32);

        let results1 = client.get_vote_results(&poll_id1);
        assert_eq!(results1.winning_option, 1); // Entry approved

        // Stage 2: Team wins! Decide prize distribution
        let poll_id2 = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Victory Prize Distribution"),
            &String::from_str(&env, "WE WON! $30K prize. Share $20K with community?"),
            &PollAction::DistributeFunds(
                20000,
                String::from_str(&env, "Tournament victory celebration"),
            ),
            &Some(7),
        );

        // Enthusiastic approval for celebration
        let celebrants = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        for celebrant in &celebrants {
            client.vote(celebrant, &poll_id2, &1u32); // All approve celebration
        }

        let results2 = client.get_vote_results(&poll_id2);
        assert_eq!(results2.total_voters, 4);
        assert_eq!(results2.winning_option, 1);
        assert_eq!(results2.vote_counts.get(1).unwrap(), 4000); // Unanimous approval
    }

    // Governance Parameter Testing
    #[test]
    fn test_governance_params_and_query() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Get initial parameters
        let initial_params = client.get_governance_params();
        assert_eq!(initial_params.threshold_percentage, 60);
        assert_eq!(initial_params.quorum_percentage, 40);
        assert_eq!(initial_params.default_expiry_days, 7);

        // Update parameters
        client.update_governance_params(&admin, &75u32, &50u32, &14u32);

        // Verify updated parameters
        let updated_params = client.get_governance_params();
        assert_eq!(updated_params.threshold_percentage, 75);
        assert_eq!(updated_params.quorum_percentage, 50);
        assert_eq!(updated_params.default_expiry_days, 14);
    }

    // Voting Capability Testing
    #[test]
    fn test_can_vote_functionality() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Voting Test"),
            &String::from_str(&env, "Test can_vote functionality"),
            &PollAction::NoExecution,
            &None,
        );

        let voter = Address::generate(&env);

        // Should be able to vote initially
        let can_vote_before = client.can_vote(&voter, &poll_id);
        assert!(can_vote_before);

        // Vote
        client.vote(&voter, &poll_id, &1u32);

        // Should not be able to vote again
        let can_vote_after = client.can_vote(&voter, &poll_id);
        assert!(!can_vote_after);
    }

    // Error Condition Testing
    #[test]
    #[should_panic(expected = "Error(Contract, #8)")]
    fn test_double_voting_error() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Double Vote Test"),
            &String::from_str(&env, "Test double voting protection"),
            &PollAction::NoExecution,
            &None,
        );

        let voter = Address::generate(&env);
        client.vote(&voter, &poll_id, &1u32); // First vote
        client.vote(&voter, &poll_id, &0u32); // Should fail
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn test_invalid_option_error() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let poll_id = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Invalid Option Test"),
            &String::from_str(&env, "Test invalid option protection"),
            &PollAction::NoExecution,
            &None,
        );

        let voter = Address::generate(&env);
        client.vote(&voter, &poll_id, &2u32); // Invalid option (only 0,1 allowed)
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn test_nonexistent_poll_error() {
        let env = create_test_env();
        let (contract_id, _admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let voter = Address::generate(&env);
        client.vote(&voter, &999u32, &1u32); // Non-existent poll
    }
}
