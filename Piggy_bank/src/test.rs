#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, BankMsg, Coin, CosmosMsg, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, Deposit, STATE, DEPOSITS};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.lock_time, 120);
        assert_eq!(state.tokens_in, Uint128::zero());
        assert_eq!(state.tokens_out, Uint128::zero());
        assert_eq!(state.deposit_count, 0);
    }

    #[test]
    fn test_deposit_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the deposit message
        let deposit_msg = ExecuteMsg::DepositTokens { amount: Uint128::new(1000) };
        let user_info = mock_info("user", &coins(1000, "earth"));

        // Execute the deposit function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), deposit_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the deposit was created correctly
        let deposit = DEPOSITS.load(&deps.storage, 0).unwrap();
        assert_eq!(deposit.deposit_id, 0);
        assert_eq!(deposit.amount, Uint128::new(1000));
        assert_eq!(deposit.from, Addr::unchecked("user"));
        assert_eq!(deposit.deposit_time, env.block.height);
        assert_eq!(deposit.unlock_time, env.block.height + 120);

        // Verify the state update
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.tokens_in, Uint128::new(1000));
        assert_eq!(state.deposit_count, 1);
    }

    #[test]
    fn test_withdraw_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Deposit tokens first
        let deposit_msg = ExecuteMsg::DepositTokens { amount: Uint128::new(1000) };
        let user_info = mock_info("user", &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), env.clone(), user_info.clone(), deposit_msg).unwrap();

        // Fast forward time to unlock tokens
        let mut env = env.clone();
        env.block.height += 121;

        // Prepare the withdraw message
        let withdraw_msg = ExecuteMsg::WithdrawTokens { deposit_id: 0, denom: "earth".to_string() };

        // Execute the withdraw function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), withdraw_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure the withdraw message is sent

        // Verify the withdraw message
        let msg = &res.messages[0].msg;
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "user");
                assert_eq!(amount[0], Coin {
                    denom: "earth".to_string(),
                    amount: Uint128::new(1000),
                });
            }
            _ => panic!("Unexpected message type"),
        }

        // Verify the state update
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.tokens_out, Uint128::new(1000));
    }

    #[test]
    fn test_set_lock_time() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the set lock time message
        let lock_time_msg = ExecuteMsg::SetLockTime { lock_time: 300 };
        let owner_info = mock_info("creator", &[]);

        // Execute the set lock time function
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), lock_time_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify the state update
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.lock_time, 300);
    }

    #[test]
    fn test_set_new_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the set new owner message
        let new_owner_msg = ExecuteMsg::SetNewOwner { new_owner: "new_owner".to_string() };
        let owner_info = mock_info("creator", &[]);

        // Execute the set new owner function
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), new_owner_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify the state update
        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("new_owner"));
    }


}
