#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary, Addr, BankMsg, Coin, CosmosMsg, CustomMsg, StdError, Uint128
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::STATE;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            lending_pool: "lending_pool_address".to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.lending_pool, Addr::unchecked("lending_pool_address"));
    }

 
   

    #[test]
    fn test_execute_operation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            lending_pool: "lending_pool_address".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the execute operation message
        let execute_operation_msg = ExecuteMsg::ExecuteOperation {
            token: "earth".to_string(),
            amount: Uint128::new(100),
            premium: Uint128::new(10),
        };
        let user_info = mock_info("user", &coins(110, "earth")); // User has enough to repay loan + premium

        // Execute the operation
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), execute_operation_msg).unwrap();
        assert_eq!(res.messages.len(), 2); // Ensure two messages are sent (repayment and return collateral)

        // Verify the repayment message
        let msg = &res.messages[0].msg;
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "lending_pool_address");
                assert_eq!(amount[0], Coin { denom: "earth".to_string(), amount: Uint128::new(110) });
            }
            _ => panic!("Unexpected message type"),
        }

        // Verify the return collateral message
        let msg = &res.messages[1].msg;
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "user");
                assert_eq!(amount[0], Coin { denom: "earth".to_string(), amount: Uint128::new(110) });
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            lending_pool: "lending_pool_address".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the withdraw message
        let withdraw_msg = ExecuteMsg::Withdraw {
            token: "earth".to_string(),
        };
        let owner_info = mock_info("creator", &[]);

        // Execute the withdraw function
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), withdraw_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure the withdraw message is sent

        // Verify the withdraw message
        let msg = &res.messages[0].msg;
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "creator");
                assert_eq!(amount[0].denom, "earth");
            }
            _ => panic!("Unexpected message type"),
        }
    }

    // #[test]
    // fn test_query_balance() {
    //     let mut deps = mock_dependencies();
    //     let env = mock_env();
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // Instantiate the contract first
    //     let msg = InstantiateMsg {
    //         owner: "creator".to_string(),
    //         lending_pool: "lending_pool_address".to_string(),
    //     };
    //     let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Query the balance
    //     let query_msg = QueryMsg::GetBalance { token: "earth".to_string() };
    //     let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    //     let balance: Uint128 = from_binary(&res).unwrap();

    //     // Check the queried balance
    //     assert_eq!(balance, Uint128::new(1000)); // Assuming initial balance of 1000 earth tokens
    // }

    // #[test]
    // fn test_query_loan_info() {
    //     let mut deps = mock_dependencies();
    //     let env = mock_env();
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // Instantiate the contract first
    //     let msg = InstantiateMsg {
    //         owner: "creator".to_string(),
    //         lending_pool: "lending_pool_address".to_string(),
    //     };
    //     let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Query the loan info
    //     let query_msg = QueryMsg::LoanInfo {};
    //     let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    //     let state: State = from_binary(&res).unwrap();

    //     // Check the queried state
    //     assert_eq!(state.owner, Addr::unchecked("creator"));
    //     assert_eq!(state.lending_pool, Addr::unchecked("lending_pool_address"));
    // }
}
