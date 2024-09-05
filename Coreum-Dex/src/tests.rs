#[cfg(test)]
mod tests {
    use crate::{contract::{execute, instantiate}, msg::{ExecuteMsg, InstantiateMsg}, state::{PoolType, LIQUIDITY_POOLS, STATE}};

    use super::*;
    use cosmwasm_std::{
        Addr, Uint128, from_binary, testing::{mock_dependencies, mock_env, mock_info},
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            reward_token: "reward_token".to_string(),
            reward_rate: 100u128,
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.reward_token, Addr::unchecked("reward_token"));
        assert_eq!(state.reward_rate, 100u128);
    }
    #[test]
fn test_add_liquidity() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user1", &[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        owner: "creator".to_string(),
        reward_token: "reward_token".to_string(),
        reward_rate: 100u128,
    };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Add liquidity
    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        token1_address: "token1".to_string(),
        token2_address: "token2".to_string(),
        amount1: Uint128::new(100),
        amount2: Uint128::new(200),
        pool_type: PoolType::XYK,
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), add_liquidity_msg).unwrap();

    // Check if the pool was created
    let pool = LIQUIDITY_POOLS.load(&deps.storage, (Addr::unchecked("token1"), Addr::unchecked("token2"))).unwrap();
    assert_eq!(pool.token1_reserve, Uint128::new(100));
    assert_eq!(pool.token2_reserve, Uint128::new(200));
    assert_eq!(pool.total_liquidity, Uint128::new(300));
}

#[test]
fn test_swap_tokens() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user1", &[]);

    // Setup: Add liquidity
    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        token1_address: "token1".to_string(),
        token2_address: "token2".to_string(),
        amount1: Uint128::new(1000),
        amount2: Uint128::new(2000),
        pool_type: PoolType::XYK,
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), add_liquidity_msg).unwrap();

    // Execute swap
    let swap_tokens_msg = ExecuteMsg::SwapTokens {
        token_in: "token1".to_string(),
        token_out: "token2".to_string(),
        amount_in: Uint128::new(500),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), swap_tokens_msg).unwrap();

    // Assert swap results
    let pool = LIQUIDITY_POOLS.load(&deps.storage, (Addr::unchecked("token1"), Addr::unchecked("token2"))).unwrap();
    assert!(pool.token1_reserve > Uint128::new(500)); // Confirm reserve update
}

}
