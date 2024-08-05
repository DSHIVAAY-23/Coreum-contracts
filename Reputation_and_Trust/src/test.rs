#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, UserReputation, REPUTATIONS, BALANCES, STATE};
    use crate::error::ContractError;
    use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            symbol: "RPT".to_string(),
            subunit: "rpt".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.denom, "rpt-cosmos2contract");
    }

    #[test]
    fn test_update_reputation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            symbol: "RPT".to_string(),
            subunit: "rpt".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let update_msg = ExecuteMsg::UpdateReputation {
            user: "user1".to_string(),
            reputation: 10,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), update_msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let reputation = REPUTATIONS.load(&deps.storage, &Addr::unchecked("user1")).unwrap();
        assert_eq!(reputation.reputation, 10);
    }

    #[test]
    fn test_reset_reputation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            symbol: "RPT".to_string(),
            subunit: "rpt".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let update_msg = ExecuteMsg::UpdateReputation {
            user: "user1".to_string(),
            reputation: 10,
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), update_msg).unwrap();

        let reset_msg = ExecuteMsg::ResetReputation { user: "user1".to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), reset_msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let reputation = REPUTATIONS.may_load(&deps.storage, &Addr::unchecked("user1")).unwrap();
        assert!(reputation.is_none());
    }

    #[test]
    fn test_transfer() {
        let mut deps = mock_dependencies(); // Create mock dependencies
        let env = mock_env(); // Mock environment
        let info = mock_info("creator", &coins(1000, "earth")); // Mock info for instantiation
    
        // Instantiate the contract
        let msg = InstantiateMsg {
            symbol: "RPT".to_string(),
            subunit: "rpt".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Set initial balances for users
        // Use mutable reference to storage when saving balances
        BALANCES.save(deps.as_mut().storage, &Addr::unchecked("user1"), &Uint128::new(500)).unwrap();
        BALANCES.save(deps.as_mut().storage, &Addr::unchecked("user2"), &Uint128::new(200)).unwrap();
    
        // Create transfer message
        let transfer_msg = ExecuteMsg::Transfer {
            recipient: "user2".to_string(),
            amount: Uint128::new(100),
        };
        let user_info = mock_info("user1", &[]); // User1 info for execution
    
        // Execute the transfer
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), transfer_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Check no messages are returned
    
        // Check updated balances
        let sender_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("user1")).unwrap();
        let recipient_balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("user2")).unwrap();
        assert_eq!(sender_balance, Uint128::new(400)); // Check sender balance after transfer
        assert_eq!(recipient_balance, Uint128::new(300)); // Check recipient balance after transfer
    }
}
