#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Timestamp, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, SUBSCRIPTIONS, STATE};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            subscription_cost: Uint128::new(1000),
            subscription_period: 30 * 24 * 60 * 60, // 30 days
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.subscription_cost, Uint128::new(1000));
        assert_eq!(state.subscription_period, 30 * 24 * 60 * 60);
    }
    #[test]
    fn test_subscribe() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            subscription_cost: Uint128::new(1000),
            subscription_period: 30 * 24 * 60 * 60, // 30 days
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the subscribe message
        let subscribe_msg = ExecuteMsg::Subscribe { denom: "earth".to_string() };
        let user_info = mock_info("user", &coins(1000, "utoken"));

        // Execute the subscribe function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), subscribe_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the subscription was created correctly
        let subscription = SUBSCRIPTIONS.load(&deps.storage, Addr::unchecked("user")).unwrap();
        assert_eq!(subscription.user, Addr::unchecked("user"));
        assert_eq!(subscription.end_time, env.block.time.plus_seconds(30 * 24 * 60 * 60));
    }
    #[test]
    fn test_renew_subscription() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            subscription_cost: Uint128::new(1000),
            subscription_period: 30 * 24 * 60 * 60, // 30 days
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Subscribe first
        let subscribe_msg = ExecuteMsg::Subscribe { denom: "earth".to_string() };
        let user_info = mock_info("user", &coins(1000, "utoken"));
        let _res = execute(deps.as_mut(), env.clone(), user_info.clone(), subscribe_msg).unwrap();

        // Fast forward time by 15 days
        let mut env = env.clone();
        env.block.time = env.block.time.plus_seconds(15 * 24 * 60 * 60);

        // Prepare the renew message
        let renew_msg = ExecuteMsg::Renew { denom: "earth".to_string()};
        let user_info = mock_info("user", &coins(1000, "utoken"));

        // Execute the renew function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), renew_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the subscription was renewed correctly
        let subscription = SUBSCRIPTIONS.load(&deps.storage, Addr::unchecked("user")).unwrap();
        assert_eq!(subscription.user, Addr::unchecked("user"));
        assert_eq!(subscription.end_time, env.block.time.plus_seconds(15 * 24 * 60 * 60 + 30 * 24 * 60 * 60));
    }
    #[test]
    fn test_withdraw_funds() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            subscription_cost: Uint128::new(1000),
            subscription_period: 30 * 24 * 60 * 60, // 30 days
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the withdraw message
        let withdraw_msg = ExecuteMsg::WithdrawFunds {};
        let owner_info = mock_info("creator", &[]);

        // Execute the withdraw function
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), withdraw_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure the withdraw message is sent

        // Verify the withdraw message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Bank(bank_msg) => {
                match bank_msg {
                    cosmwasm_std::BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, "creator");
                        assert!(amount.len() > 0); // Ensure some amount is sent
                    }
                    _ => panic!("Unexpected bank message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }
    


}