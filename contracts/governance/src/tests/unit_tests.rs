#[cfg(test)]
mod tests {
    use crate::contract::{GovernanceParams, PollAction};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_governance_params_creation() {
        let params = GovernanceParams {
            threshold_percentage: 60,
            quorum_percentage: 50,
            default_expiry_days: 30,
        };

        assert_eq!(params.threshold_percentage, 60);
        assert_eq!(params.quorum_percentage, 50);
        assert_eq!(params.default_expiry_days, 30);
    }

    #[test]
    fn test_poll_action_creation() {
        let action1 = PollAction::NoExecution;
        let env = Env::default();
        let action2 =
            PollAction::DistributeFunds(1000, String::from_str(&env, "Test distribution"));

        let env = Env::default();
        let addr = Address::generate(&env);
        let action3 = PollAction::TransferTokens(addr, 500);

        match action1 {
            PollAction::NoExecution => assert!(true),
            _ => assert!(false),
        }

        match action2 {
            PollAction::DistributeFunds(amount, description) => {
                assert_eq!(amount, 1000);
                assert_eq!(description, String::from_str(&env, "Test distribution"));
            }
            _ => assert!(false),
        }

        match action3 {
            PollAction::TransferTokens(_, amount) => assert_eq!(amount, 500),
            _ => assert!(false),
        }
    }
}
