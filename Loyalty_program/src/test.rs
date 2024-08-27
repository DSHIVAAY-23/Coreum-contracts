#[cfg(test)]
mod tests {
    use coreum_wasm_sdk::assetft;
    use cosmwasm_std::{from_binary, Addr, Uint128, Coin, StdError};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, STATE};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.token_address, Addr::unchecked("token"));
    }

    #[test]
    fn test_earn_tokens_whitelisted() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the EarnTokens message
        let earn_tokens_msg = ExecuteMsg::EarnTokens {
            customer: "addr1...".to_string(),
            amount: Uint128::new(1000),
        };
        let user_info = mock_info("creator", &[]);

        // Execute the earn_tokens function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), earn_tokens_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure mint message is sent

        // Verify the mint message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Custom(custom_msg) => {
                match custom_msg {
                    coreum_wasm_sdk::core::CoreumMsg::AssetFT(assetft::Msg::Mint { coin }) => {
                        assert_eq!(coin.denom, "token");
                        assert_eq!(coin.amount, Uint128::new(1000));
                    }
                    _ => panic!("Unexpected custom message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_earn_tokens_not_whitelisted() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the EarnTokens message for a non-whitelisted customer
        let earn_tokens_msg = ExecuteMsg::EarnTokens {
            customer: "non-whitelisted".to_string(),
            amount: Uint128::new(1000),
        };
        let user_info = mock_info("creator", &[]);

        // Execute the earn_tokens function and expect an error
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), earn_tokens_msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_redeem_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the RedeemTokens message
        let redeem_tokens_msg = ExecuteMsg::RedeemTokens {
            customer: "addr1...".to_string(),
            amount: Uint128::new(500),
        };
        let user_info = mock_info("creator", &[]);

      

        // Execute the redeem_tokens function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), redeem_tokens_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure burn message is sent

        // Verify the burn message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Custom(custom_msg) => {
                match custom_msg {
                    coreum_wasm_sdk::core::CoreumMsg::AssetFT(assetft::Msg::Burn { coin }) => {
                        assert_eq!(coin.denom, "token");
                        assert_eq!(coin.amount, Uint128::new(500));
                    }
                    _ => panic!("Unexpected custom message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_transfer_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the TransferTokens message
        let transfer_tokens_msg = ExecuteMsg::TransferTokens {
            from: "addr1...".to_string(),
            to: "addr2...".to_string(),
            amount: Uint128::new(100),
        };
        let user_info = mock_info("creator", &[]);

        // Execute the transfer_tokens function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), transfer_tokens_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure transfer message is sent

        // Verify the transfer message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Bank(bank_msg) => {
                match bank_msg {
                    cosmwasm_std::BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, "addr2...");
                        assert_eq!(amount[0].denom, "token");
                        assert_eq!(amount[0].amount, Uint128::new(100));
                    }
                    _ => panic!("Unexpected bank message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_withdraw_tokens_as_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the WithdrawTokens message
        let withdraw_tokens_msg = ExecuteMsg::WithdrawTokens {
            amount: Uint128::new(1000),
        };
        let owner_info = mock_info("creator", &[]);

        // Execute the withdraw_tokens function
        let res = execute(deps.as_mut(), env.clone(), owner_info.clone(), withdraw_tokens_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure withdraw message is sent

        // Verify the withdraw message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Bank(bank_msg) => {
                match bank_msg {
                    cosmwasm_std::BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, "creator");
                        assert_eq!(amount[0].denom, "token");
                        assert_eq!(amount[0].amount, Uint128::new(1000));
                    }
                    _ => panic!("Unexpected bank message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn test_withdraw_tokens_not_as_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the WithdrawTokens message
        let withdraw_tokens_msg = ExecuteMsg::WithdrawTokens {
            amount: Uint128::new(1000),
        };
        let non_owner_info = mock_info("not_owner", &[]);

        // Execute the withdraw_tokens function as a non-owner and expect an error
        let res = execute(deps.as_mut(), env.clone(), non_owner_info.clone(), withdraw_tokens_msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    
}
