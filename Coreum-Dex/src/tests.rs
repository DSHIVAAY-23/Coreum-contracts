#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, PoolType, SwapOperation};
    use crate::state::{POOL_INFO, LIQUIDITY_PROVIDERS};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            fee: Uint128::new(1), // 0.01%
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Test if the contract state is correctly initialized
        let state = POOL_INFO.load(&deps.storage, "earth".to_string()).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.fee, Uint128::new(1));
    }

    #[test]
    fn test_create_pool() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            fee: Uint128::new(1), // 0.01%
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare create pool message
        let create_pool_msg = ExecuteMsg::CreatePool {
            pool_type: PoolType::XYK,
            token1_denom: "token1".to_string(),
            token2_denom: "token2".to_string(),
            amount1: Uint128::new(1000),
            amount2: Uint128::new(1000),
        };

        // Execute the create pool function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), create_pool_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the pool was created correctly
        let pool_info = POOL_INFO.load(&deps.storage, ("token1".to_string(), "token2".to_string())).unwrap();
        assert_eq!(pool_info.pool_type, PoolType::XYK);
        assert_eq!(pool_info.token1_denom, "token1");
        assert_eq!(pool_info.token2_denom, "token2");
        assert_eq!(pool_info.amount1, Uint128::new(1000));
        assert_eq!(pool_info.amount2, Uint128::new(1000));
    }

    #[test]
    fn test_provide_liquidity() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            fee: Uint128::new(1), // 0.01%
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a pool first
        let create_pool_msg = ExecuteMsg::CreatePool {
            pool_type: PoolType::XYK,
            token1_denom: "token1".to_string(),
            token2_denom: "token2".to_string(),
            amount1: Uint128::new(1000),
            amount2: Uint128::new(1000),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_pool_msg).unwrap();

        // Prepare the provide liquidity message
        let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
            token1_denom: "token1".to_string(),
            token2_denom: "token2".to_string(),
            amount1: Uint128::new(500),
            amount2: Uint128::new(500),
        };
        let user_info = mock_info("user", &coins(500, "token1"));

        // Execute the provide liquidity function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), provide_liquidity_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify the user's liquidity provision
        let liquidity_provider = LIQUIDITY_PROVIDERS.load(&deps.storage, ("user".to_string(), "token1".to_string(), "token2".to_string())).unwrap();
        assert_eq!(liquidity_provider.user, Addr::unchecked("user"));
        assert_eq!(liquidity_provider.amount1, Uint128::new(500));
        assert_eq!(liquidity_provider.amount2, Uint128::new(500));
    }

    #[test]
    fn test_swap_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            fee: Uint128::new(1), // 0.01%
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a pool first
        let create_pool_msg = ExecuteMsg::CreatePool {
            pool_type: PoolType::XYK,
            token1_denom: "token1".to_string(),
            token2_denom: "token2".to_string(),
            amount1: Uint128::new(1000),
            amount2: Uint128::new(1000),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_pool_msg).unwrap();

        // Prepare the swap message
        let swap_msg = ExecuteMsg::Swap {
            operation: SwapOperation {
                token_in: "token1".to_string(),
                token_out: "token2".to_string(),
                amount_in: Uint128::new(100),
                min_amount_out: Uint128::new(50),
            },
        };
        let user_info = mock_info("user", &coins(100, "token1"));

        // Execute the swap function
        let res = execute(deps.as_mut(), env.clone(), user_info.clone(), swap_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Check the updated pool amounts
        let pool_info = POOL_INFO.load(&deps.storage, ("token1".to_string(), "token2".to_string())).unwrap();
        assert_eq!(pool_info.amount1, Uint128::new(1100)); // token1 increased
        assert!(pool_info.amount2 < Uint128::new(1000)); // token2 decreased
    }
}