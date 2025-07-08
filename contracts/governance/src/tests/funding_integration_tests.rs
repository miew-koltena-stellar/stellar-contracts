#[cfg(test)]
mod funding_integration_tests {
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

    // Tournament Funding Scenarios
    #[test]
    fn test_tournament_prize_pool_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team won a tournament and has $50,000 in prize pool
        // They want to distribute $25,000 (half) to token holders
        let team_asset_id = 1u64;
        let distribution_amount = 25000u128; // Half of the $50K prize

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Tournament Prize Distribution - Half Share"),
            &String::from_str(&env, "We won $50K in the championship! Should we distribute $25K to all token holders and keep $25K for team operations?"),
            &PollAction::DistributeFunds(
                distribution_amount,
                String::from_str(&env, "Championship winnings - 50% distribution to community")
            ),
            &Some(7), // One week voting period
        );

        // Team members and supporters vote
        let team_captain = Address::generate(&env);
        let team_member1 = Address::generate(&env);
        let team_member2 = Address::generate(&env);
        let supporter1 = Address::generate(&env);
        let supporter2 = Address::generate(&env);

        // Voting pattern: Most approve, one member wants to keep more for team
        client.vote(&team_captain, &poll_id, &1u32); // Approve
        client.vote(&team_member1, &poll_id, &1u32); // Approve
        client.vote(&team_member2, &poll_id, &0u32); // Deny (wants more for team)
        client.vote(&supporter1, &poll_id, &1u32); // Approve
        client.vote(&supporter2, &poll_id, &1u32); // Approve

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 5);
        assert_eq!(results.vote_counts.get(0).unwrap(), 1000); // 1 deny vote
        assert_eq!(results.vote_counts.get(1).unwrap(), 4000); // 4 approve votes
        assert_eq!(results.winning_option, 1); // Approve wins

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 80); // 4000/5000 * 100

        // Verify the distribution details
        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::DistributeFunds(amount, description) => {
                assert_eq!(amount, distribution_amount);
                // For now, just check that the description exists and has expected content
                assert_eq!(
                    description,
                    String::from_str(
                        &env,
                        "Championship winnings - 50% distribution to community"
                    )
                );
            }
            _ => panic!("Expected DistributeFunds action"),
        }
    }

    #[test]
    fn test_sponsorship_revenue_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team got a $30,000 sponsorship deal
        // Proposal to distribute $20,000 to token holders, keep $10,000 for operations
        let team_asset_id = 2u64;
        let distribution_amount = 20000u128; // $20K out of $30K total

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Sponsorship Revenue Distribution"),
            &String::from_str(&env, "We secured a $30K sponsorship deal! Proposal: $20K to token holders, $10K for team operations and equipment."),
            &PollAction::DistributeFunds(
                distribution_amount,
                String::from_str(&env, "Sponsorship revenue sharing - 67% to community")
            ),
            &Some(5), // 5 days for faster decision
        );

        // Community votes
        let community_member1 = Address::generate(&env);
        let community_member2 = Address::generate(&env);
        let community_member3 = Address::generate(&env);
        let team_manager = Address::generate(&env);

        client.vote(&community_member1, &poll_id, &1u32); // Approve
        client.vote(&community_member2, &poll_id, &1u32); // Approve
        client.vote(&community_member3, &poll_id, &1u32); // Approve
        client.vote(&team_manager, &poll_id, &1u32); // Approve

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.winning_option, 1);
        assert_eq!(results.vote_counts.get(1).unwrap(), 4000); // All approve

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 100);
    }

    #[test]
    fn test_merchandise_sales_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team sold merchandise and made $8,000 profit
        // Proposal to distribute all of it to token holders as bonus
        let team_asset_id = 3u64;
        let merchandise_profit = 8000u128;

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Merchandise Profit Bonus Distribution"),
            &String::from_str(&env, "Our new jersey sales exceeded expectations! $8K profit. Should we distribute it all as a bonus to our amazing community?"),
            &PollAction::DistributeFunds(
                merchandise_profit,
                String::from_str(&env, "Merchandise sales bonus - 100% to loyal supporters")
            ),
            &Some(3), // Quick 3-day vote
        );

        // Enthusiastic community response
        let fan1 = Address::generate(&env);
        let fan2 = Address::generate(&env);
        let fan3 = Address::generate(&env);
        let fan4 = Address::generate(&env);
        let fan5 = Address::generate(&env);

        client.vote(&fan1, &poll_id, &1u32); // Approve
        client.vote(&fan2, &poll_id, &1u32); // Approve
        client.vote(&fan3, &poll_id, &1u32); // Approve
        client.vote(&fan4, &poll_id, &1u32); // Approve
        client.vote(&fan5, &poll_id, &1u32); // Approve

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 100);
    }

    #[test]
    fn test_streaming_revenue_distribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team's streaming and content creation generated $15,000
        // They want to distribute $10,000 to token holders
        let team_asset_id = 4u64;
        let distribution_amount = 10000u128; // $10K out of $15K total

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Streaming Revenue Share"),
            &String::from_str(&env, "Our streams and YouTube content made $15K this quarter! Proposal: $10K to community, $5K for content production costs."),
            &PollAction::DistributeFunds(
                distribution_amount,
                String::from_str(&env, "Q1 streaming revenue share - supporting our content creators")
            ),
            &Some(7),
        );

        // Mixed response from community
        let content_fan1 = Address::generate(&env);
        let content_fan2 = Address::generate(&env);
        let investor1 = Address::generate(&env);
        let investor2 = Address::generate(&env);
        let team_streamer = Address::generate(&env);

        client.vote(&content_fan1, &poll_id, &1u32); // Approve
        client.vote(&content_fan2, &poll_id, &1u32); // Approve
        client.vote(&investor1, &poll_id, &0u32); // Deny (wants more reinvestment)
        client.vote(&investor2, &poll_id, &1u32); // Approve
        client.vote(&team_streamer, &poll_id, &1u32); // Approve

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.vote_counts.get(0).unwrap(), 1000); // 1 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 4000); // 4 approve
        assert_eq!(results.winning_option, 1);

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 80);
    }

    #[test]
    fn test_emergency_funding_redistribution() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team member injured, need emergency medical funds
        // Redistribute $5,000 from community fund to affected member
        let team_asset_id = 5u64;
        let emergency_amount = 5000u64;
        let injured_member = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Emergency Medical Fund Transfer"),
            &String::from_str(&env, "Our teammate was injured and needs immediate medical care. We should transfer $5K from our community fund."),
            &PollAction::TransferTokens(injured_member.clone(), emergency_amount),
            &Some(1), // Emergency - only 1 day
        );

        // Quick community response for emergency
        let teammate1 = Address::generate(&env);
        let teammate2 = Address::generate(&env);
        let supporter1 = Address::generate(&env);
        let supporter2 = Address::generate(&env);

        client.vote(&teammate1, &poll_id, &1u32); // Approve
        client.vote(&teammate2, &poll_id, &1u32); // Approve
        client.vote(&supporter1, &poll_id, &1u32); // Approve
        client.vote(&supporter2, &poll_id, &1u32); // Approve

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 100);

        // Verify emergency transfer details
        let poll = client.get_poll(&poll_id);
        match poll.action {
            PollAction::TransferTokens(recipient, amount) => {
                assert_eq!(recipient, injured_member);
                assert_eq!(amount, emergency_amount);
            }
            _ => panic!("Expected TransferTokens action"),
        }
    }

    #[test]
    fn test_training_camp_funding() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team wants to attend expensive training bootcamp
        // Cost: $12,000, proposal to fund from community treasury
        let team_asset_id = 6u64;
        let training_cost = 12000u64;
        let training_facility = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Training Bootcamp Investment"),
            &String::from_str(&env, "Elite training bootcamp opportunity - $12K for 2 weeks intensive training. Investment in our competitive future."),
            &PollAction::TransferTokens(training_facility.clone(), training_cost),
            &Some(10), // 10 days for careful consideration
        );

        // Community debates the investment
        let performance_focused1 = Address::generate(&env);
        let performance_focused2 = Address::generate(&env);
        let performance_focused3 = Address::generate(&env);
        let budget_conscious1 = Address::generate(&env);
        let budget_conscious2 = Address::generate(&env);

        client.vote(&performance_focused1, &poll_id, &1u32); // Approve
        client.vote(&performance_focused2, &poll_id, &1u32); // Approve
        client.vote(&performance_focused3, &poll_id, &1u32); // Approve
        client.vote(&budget_conscious1, &poll_id, &0u32); // Deny
        client.vote(&budget_conscious2, &poll_id, &0u32); // Deny

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.vote_counts.get(0).unwrap(), 2000); // 2 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 3000); // 3 approve
        assert_eq!(results.winning_option, 1);

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 60);
    }

    #[test]
    fn test_equipment_purchase_funding() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Team needs new gaming equipment worth $8,000
        let team_asset_id = 7u64;
        let equipment_cost = 8000u64;
        let equipment_vendor = Address::generate(&env);

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Gaming Equipment Upgrade"),
            &String::from_str(&env, "New monitors, keyboards, and headsets needed for competitive edge. Total cost: $8K."),
            &PollAction::TransferTokens(equipment_vendor.clone(), equipment_cost),
            &Some(5),
        );

        // Team and supporters vote
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let coach = Address::generate(&env);
        let fan1 = Address::generate(&env);
        let fan2 = Address::generate(&env);
        let fan3 = Address::generate(&env);

        client.vote(&player1, &poll_id, &1u32); // Approve
        client.vote(&player2, &poll_id, &1u32); // Approve
        client.vote(&coach, &poll_id, &1u32); // Approve
        client.vote(&fan1, &poll_id, &1u32); // Approve
        client.vote(&fan2, &poll_id, &0u32); // Deny
        client.vote(&fan3, &poll_id, &1u32); // Approve

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        // 5 approve vs 1 deny = 83% approval
        assert_eq!(execution.approval_percentage, 83);
    }

    #[test]
    fn test_unsuccessful_funding_proposal() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Expensive luxury purchase that community rejects
        let team_asset_id = 8u64;
        let luxury_cost = 25000u128;

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Luxury Team House Proposal"),
            &String::from_str(&env, "Proposal to rent expensive gaming house for $25K/month. Would provide better practice environment."),
            &PollAction::DistributeFunds(
                luxury_cost,
                String::from_str(&env, "Monthly luxury gaming house rental")
            ),
            &Some(14), // 2 weeks for thorough discussion
        );

        // Community largely rejects expensive proposal
        let supporter1 = Address::generate(&env);
        let supporter2 = Address::generate(&env);
        let budget_voter1 = Address::generate(&env);
        let budget_voter2 = Address::generate(&env);
        let budget_voter3 = Address::generate(&env);
        let budget_voter4 = Address::generate(&env);

        client.vote(&supporter1, &poll_id, &1u32); // Approve
        client.vote(&supporter2, &poll_id, &1u32); // Approve
        client.vote(&budget_voter1, &poll_id, &0u32); // Deny
        client.vote(&budget_voter2, &poll_id, &0u32); // Deny
        client.vote(&budget_voter3, &poll_id, &0u32); // Deny
        client.vote(&budget_voter4, &poll_id, &0u32); // Deny

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.vote_counts.get(0).unwrap(), 4000); // 4 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 2000); // 2 approve
        assert_eq!(results.winning_option, 0); // Deny wins

        let execution = client.check_poll_execution(&poll_id);
        assert!(!execution.should_execute); // Should NOT execute since deny won
        assert_eq!(execution.approval_percentage, 33); // 2000/6000 * 100
    }

    #[test]
    fn test_quarterly_profit_sharing() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Scenario: Regular quarterly profit sharing from multiple revenue streams
        let team_asset_id = 9u64;
        let community_share = 20000u128; // ~57% of $35K total profit

        let poll_id = client.create_poll(
            &admin,
            &team_asset_id,
            &String::from_str(&env, "Q4 Profit Sharing Distribution"),
            &String::from_str(&env, "Q4 was our best quarter! Total profit: $35K from tournaments, sponsorships, and content. Proposal: $20K to community, $15K reinvested."),
            &PollAction::DistributeFunds(
                community_share,
                String::from_str(&env, "Q4 profit sharing - 57% community distribution")
            ),
            &Some(7),
        );

        // Broad community participation
        let participants = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        // 6 approve, 2 deny
        client.vote(&participants[0], &poll_id, &1u32); // Approve
        client.vote(&participants[1], &poll_id, &1u32); // Approve
        client.vote(&participants[2], &poll_id, &0u32); // Deny
        client.vote(&participants[3], &poll_id, &1u32); // Approve
        client.vote(&participants[4], &poll_id, &1u32); // Approve
        client.vote(&participants[5], &poll_id, &1u32); // Approve
        client.vote(&participants[6], &poll_id, &0u32); // Deny
        client.vote(&participants[7], &poll_id, &1u32); // Approve

        let results = client.get_vote_results(&poll_id);
        assert_eq!(results.total_voters, 8);
        assert_eq!(results.vote_counts.get(0).unwrap(), 2000); // 2 deny
        assert_eq!(results.vote_counts.get(1).unwrap(), 6000); // 6 approve
        assert_eq!(results.winning_option, 1);

        let execution = client.check_poll_execution(&poll_id);
        assert!(execution.should_execute);
        assert_eq!(execution.approval_percentage, 75); // 6000/8000 * 100
    }

    // Complex Multi-Stage Funding Scenarios
    #[test]
    fn test_multi_stage_tournament_funding() {
        let env = create_test_env();
        let (contract_id, admin, _fractcore_contract, _funding_contract) =
            setup_governance_contract(&env);
        let client = GovernanceContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Stage 1: Approve initial tournament entry fee
        let entry_fee = 2000u64;
        let tournament_organizer = Address::generate(&env);

        let poll_id1 = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Tournament Entry Fee"),
            &String::from_str(
                &env,
                "Major tournament entry fee: $2K. Prize pool: $100K. Should we participate?",
            ),
            &PollAction::TransferTokens(tournament_organizer.clone(), entry_fee),
            &Some(3),
        );

        // Community approves entry (need enough votes to meet quorum)
        let voter1 = Address::generate(&env);
        let voter2 = Address::generate(&env);
        let voter3 = Address::generate(&env);
        let voter4 = Address::generate(&env);
        let voter5 = Address::generate(&env);

        client.vote(&voter1, &poll_id1, &1u32);
        client.vote(&voter2, &poll_id1, &1u32);
        client.vote(&voter3, &poll_id1, &1u32);
        client.vote(&voter4, &poll_id1, &1u32);
        client.vote(&voter5, &poll_id1, &1u32);

        let execution1 = client.check_poll_execution(&poll_id1);
        assert!(execution1.should_execute);

        // Stage 2: Team wins! Now decide prize distribution
        let community_share = 30000u128; // $30K out of $50K prize

        let poll_id2 = client.create_poll(
            &admin,
            &1u64,
            &String::from_str(&env, "Prize Distribution - Tournament Success!"),
            &String::from_str(&env, "WE WON 2nd PLACE! $50K prize! Proposal: $30K to community, $20K for next tournament prep."),
            &PollAction::DistributeFunds(
                community_share,
                String::from_str(&env, "Tournament prize distribution - community celebration!")
            ),
            &Some(7),
        );

        // Enthusiastic approval
        let voter3 = Address::generate(&env);
        let voter4 = Address::generate(&env);
        let voter5 = Address::generate(&env);

        client.vote(&voter1, &poll_id2, &1u32);
        client.vote(&voter2, &poll_id2, &1u32);
        client.vote(&voter3, &poll_id2, &1u32);
        client.vote(&voter4, &poll_id2, &1u32);
        client.vote(&voter5, &poll_id2, &1u32);

        let execution2 = client.check_poll_execution(&poll_id2);
        assert!(execution2.should_execute);
        assert_eq!(execution2.approval_percentage, 100);
    }
}
