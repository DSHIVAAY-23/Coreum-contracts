#[cfg(test)]
mod tests {

    use crate::contract::{execute, instantiate};
    use crate::execute::register_pool;
    use crate::query::{query_all_pools, query_fee, query_pool, query_simulate_swap};
    use crate::state::{State, STATE,POOLS};

    use cosmwasm_std::{coins, from_binary, Response, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use euclid::error::ContractError;
    use euclid::fee::Fee;
    use euclid::msgs::vlp::{AllPoolsResponse, ExecuteMsg, FeeResponse, GetSwapResponse, InstantiateMsg, PoolResponse};
    use euclid::pool::Pool;
    use euclid::token::{Pair, PairInfo, Token, TokenInfo, TokenType};

    #[test]
    // Write a test for instantiation
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
    #[test]
    fn test_execute_register_pool() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
        
        // Instantiate the contract first
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
    
        // Prepare the pool registration message
        let pair_info = PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        };
    
        let msg = ExecuteMsg::RegisterPool {
            chain_id: "chain_id".to_string(),
            pair_info: pair_info.clone(),
        };
    
        // Execute the register_pool function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent
    
        // Verify that the pool was registered correctly
        let pool_data = POOLS.load(deps.as_ref().storage, &"chain_id".to_string()).unwrap();
        assert_eq!(pool_data.chain, "chain_id".to_string());
        assert_eq!(pool_data.pair.token_1, pair_info.token_1);
        assert_eq!(pool_data.pair.token_2, pair_info.token_2);
        assert_eq!(pool_data.reserve_1, Uint128::zero());
        assert_eq!(pool_data.reserve_2, Uint128::zero());
    }
    
    #[test]
    fn test_add_liquidity_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Register the pool
        let pair_info = PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        };
        let chain_id = "chain_id".to_string();
        let msg = ExecuteMsg::RegisterPool {
            chain_id: chain_id.clone(),
            pair_info: pair_info.clone(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Add liquidity
        let token_1_liquidity = Uint128::new(100);
        let token_2_liquidity = Uint128::new(200);
        let slippage_tolerance = 1;

        let msg = ExecuteMsg::AddLiquidity {
            chain_id: chain_id.clone(),
            token_1_liquidity,
            token_2_liquidity,
            slippage_tolerance,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].value, "add_liquidity");
        assert_eq!(res.attributes[1].value, chain_id);
        assert_eq!(res.attributes[2].value, "100");
        assert_eq!(res.attributes[3].value, "200");

        // Verify the pool state
        let pool_data = POOLS.load(deps.as_ref().storage, &chain_id).unwrap();
        assert_eq!(pool_data.reserve_1, Uint128::new(100));
        assert_eq!(pool_data.reserve_2, Uint128::new(200));
    }

    #[test]
    fn test_add_liquidity_invalid_slippage() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Register the pool
        let pair_info = PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        };
        let chain_id = "chain_id".to_string();
        let msg = ExecuteMsg::RegisterPool {
            chain_id: chain_id.clone(),
            pair_info: pair_info.clone(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Add liquidity with invalid slippage tolerance
        let token_1_liquidity = Uint128::new(100);
        let token_2_liquidity = Uint128::new(200);
        let slippage_tolerance = 200; // Invalid value

        let msg = ExecuteMsg::AddLiquidity {
            chain_id: chain_id.clone(),
            token_1_liquidity,
            token_2_liquidity,
            slippage_tolerance,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ContractError::InvalidSlippageTolerance {});
    }

    #[test]
    fn test_remove_liquidity_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
    
        // Instantiate the contract first
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Register the pool
        let pair_info = PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        };
        let chain_id = "chain_id".to_string();
        let msg = ExecuteMsg::RegisterPool {
            chain_id: chain_id.clone(),
            pair_info: pair_info.clone(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Add liquidity
        let token_1_liquidity = Uint128::new(100);
        let token_2_liquidity = Uint128::new(200);
        let slippage_tolerance = 1;
    
        let msg = ExecuteMsg::AddLiquidity {
            chain_id: chain_id.clone(),
            token_1_liquidity,
            token_2_liquidity,
            slippage_tolerance,
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Remove liquidity
        let lp_allocation = Uint128::new(50);
    
        let msg = ExecuteMsg::RemoveLiquidity {
            chain_id: chain_id.clone(),
            lp_allocation,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].value, "remove_liquidity");
        assert_eq!(res.attributes[1].value, chain_id);
        assert_eq!(res.attributes[2].value, "50");
    
        // Verify the pool state
        let pool_data = POOLS.load(deps.as_ref().storage, &chain_id).unwrap();
        assert_eq!(pool_data.reserve_1, Uint128::new(50));
        assert_eq!(pool_data.reserve_2, Uint128::new(100));
    }
    
    #[test]
    fn test_remove_liquidity_insufficient_liquidity() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));
    
        // Instantiate the contract first
        let msg = InstantiateMsg {
            router: "router".to_string(),
            vcoin: "vcoin".to_string(),
            pair: Pair {
                token_1: Token {
                    id: "token_1".to_string(),
                },
                token_2: Token {
                    id: "token_2".to_string(),
                },
            },
            fee: Fee {
                lp_fee: 1,
                treasury_fee: 1,
                staker_fee: 1,
            },
            execute: None,
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Register the pool
        let pair_info = PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        };
        let chain_id = "chain_id".to_string();
        let msg = ExecuteMsg::RegisterPool {
            chain_id: chain_id.clone(),
            pair_info: pair_info.clone(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Add liquidity
        let token_1_liquidity = Uint128::new(100);
        let token_2_liquidity = Uint128::new(200);
        let slippage_tolerance = 1;
    
        let msg = ExecuteMsg::AddLiquidity {
            chain_id: chain_id.clone(),
            token_1_liquidity,
            token_2_liquidity,
            slippage_tolerance,
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    
        // Remove liquidity with insufficient allocation
        let lp_allocation = Uint128::new(150); // More than available
    
        let msg = ExecuteMsg::RemoveLiquidity {
            chain_id: chain_id.clone(),
            lp_allocation,
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), ContractError::InsufficientDeposit {});
    }
    #[test]
fn test_execute_swap_success() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate the contract first
    let msg = InstantiateMsg {
        router: "router".to_string(),
        vcoin: "vcoin".to_string(),
        pair: Pair {
            token_1: Token {
                id: "token_1".to_string(),
            },
            token_2: Token {
                id: "token_2".to_string(),
            },
        },
        fee: Fee {
            lp_fee: 1,
            treasury_fee: 1,
            staker_fee: 1,
        },
        execute: None,
    };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Register the pool
    let pair_info = PairInfo {
        token_1: TokenInfo {
            token: Token { id: "token_1".to_string() },
            token_type: TokenType::Native { denom: "token_1".to_string() },
        },
        token_2: TokenInfo {
            token: Token { id: "token_2".to_string() },
            token_type: TokenType::Native { denom: "token_2".to_string() },
        },
    };
    let chain_id = "chain_id".to_string();
    let msg = ExecuteMsg::RegisterPool {
        chain_id: chain_id.clone(),
        pair_info: pair_info.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add liquidity
    let token_1_liquidity = Uint128::new(100);
    let token_2_liquidity = Uint128::new(200);
    let slippage_tolerance = 1;

    let msg = ExecuteMsg::AddLiquidity {
        chain_id: chain_id.clone(),
        token_1_liquidity,
        token_2_liquidity,
        slippage_tolerance,
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Execute swap
    let offer_amount = Uint128::new(10);
    let min_receive = Uint128::new(15);

     // Execute swap
     let msg = ExecuteMsg::Swap {
        to_chain_id: "chain_id".to_string(),
        to_address: "recipient".to_string(),
        asset_in:Token{id: "token_1".to_string()}, 
        amount_in: Uint128::new(100),
        min_token_out: Uint128::new(50),
        swap_id: "swap_1".to_string(),
        next_swaps: vec![],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.attributes[0].value, "execute_swap");
    assert_eq!(res.attributes[1].value, chain_id);
    assert_eq!(res.attributes[2].value, "10");
    assert_eq!(res.attributes[3].value, "15");

    // Verify the pool state
    let pool_data = POOLS.load(deps.as_ref().storage, &chain_id).unwrap();
    assert_eq!(pool_data.reserve_1, Uint128::new(110));
    assert_eq!(pool_data.reserve_2, Uint128::new(185));
}

#[test]
fn test_execute_swap_min_receive_not_met() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("creator", &coins(1000, "earth"));

    // Instantiate the contract first
    let msg = InstantiateMsg {
        router: "router".to_string(),
        vcoin: "vcoin".to_string(),
        pair: Pair {
            token_1: Token {
                id: "token_1".to_string(),
            },
            token_2: Token {
                id: "token_2".to_string(),
            },
        },
        fee: Fee {
            lp_fee: 1,
            treasury_fee: 1,
            staker_fee: 1,
        },
        execute: None,
    };
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Register the pool
    let pair_info = PairInfo {
        token_1: TokenInfo {
            token: Token { id: "token_1".to_string() },
            token_type: TokenType::Native { denom: "token_1".to_string() },
        },
        token_2: TokenInfo {
            token: Token { id: "token_2".to_string() },
            token_type: TokenType::Native { denom: "token_2".to_string() },
        },
    };
    let chain_id = "chain_id".to_string();
    let msg = ExecuteMsg::RegisterPool {
        chain_id: chain_id.clone(),
        pair_info: pair_info.clone(),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Add liquidity
    let token_1_liquidity = Uint128::new(100);
    let token_2_liquidity = Uint128::new(200);
    let slippage_tolerance = 1;

    let msg = ExecuteMsg::AddLiquidity {
        chain_id: chain_id.clone(),
        token_1_liquidity,
        token_2_liquidity,
        slippage_tolerance,
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Execute swap with min_receive not met
    let offer_amount = Uint128::new(10);
    let min_receive = Uint128::new(20); // More than possible

   let msg = ExecuteMsg::Swap {
        to_chain_id: "chain_id".to_string(),
        to_address: "address".to_string(),
        asset_in:Token{id: "token_1".to_string()}, 
        amount_in: Uint128::new(1000),
        min_token_out: Uint128::new(900),
        swap_id: "swap_id".to_string(),
        next_swaps: vec![],
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        ContractError::SlippageExceeded {
            amount: Uint128::new(1000),
            min_amount_out: Uint128::new(1100)
        }
    );
}
#[test]
fn test_query_simulate_swap_success() {
    // Tests if simulate swap query returns correct output when given valid inputs
    // Setup mock dependencies and state
    let mut deps = mock_dependencies();
    let env = mock_env();
    let state = State {
        router: "router".to_string(),
        vcoin: "vcoin".to_string(),
        last_updated:0,
        pair: Pair {
            token_1: Token { id: "token_1".to_string() },
            token_2: Token { id: "token_2".to_string() },
        },
        total_reserve_1: Uint128::new(1000),
        total_reserve_2: Uint128::new(2000),
        total_lp_tokens: Uint128::new(3000),
        fee: Fee {
            lp_fee: 1,
            treasury_fee: 1,
            staker_fee: 1,
        },
    };
    STATE.save(&mut deps.storage, &state).unwrap();

    // Call query_simulate_swap
    let asset = Token { id: "token_1".to_string() };
    let asset_amount = Uint128::new(100);
    let next_swaps = vec![];
    let res = query_simulate_swap(deps.as_ref(), asset, asset_amount, next_swaps).unwrap();

    // Validate the result
    let swap_response: GetSwapResponse = from_binary(&res).unwrap();
    assert_eq!(swap_response.amount_out, Uint128::new(47)); // Example expected value
    assert_eq!(swap_response.asset_out.id, "token_2".to_string());
}
#[test]
fn test_query_simulate_swap_success(){
    // Tests if simulate swap query returns correct output when given valid inputs
    // Setup mock dependencies and state
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let state = State {
        pair: Pair {
            token_1: Token::Native("token_1".to_string()),
            token_2: Token::Native("token_2".to_string()),
        },
        total_reserve_1: Uint128::new(1000),
        total_reserve_2: Uint128::new(2000),
        total_lp_tokens: Uint128::new(3000),
        fee: Fee {
            lp_fee: 1,
            treasury_fee: 1,
            staker_fee: 1,
        },
    };
    STATE.save(&mut deps.storage, &state).unwrap();

    // Call query_simulate_swap
    let asset = Token::Native("token_1".to_string());
    let asset_amount = Uint128::new(100);
    let next_swaps = vec![];
    let res = query_simulate_swap(deps.as_ref(), asset, asset_amount, next_swaps).unwrap();

    // Validate the result
    let swap_response: GetSwapResponse = from_binary(&res).unwrap();
    assert_eq!(swap_response.amount_out, Uint128::new(47)); // Example expected value
    assert_eq!(swap_response.asset_out, Token::Native("token_2".to_string()));
}
#[test]
fn test_query_fee_success() {
    // Tests if fee query returns correct fee information
    let mut deps = mock_dependencies();
    let state = State {
        router: "router".to_string(),
        vcoin: "vcoin".to_string(),
        last_updated:0,
        pair: Pair {
            token_1: Token { id: "token_1".to_string() },
            token_2: Token { id: "token_2".to_string() },
        },
        total_reserve_1: Uint128::new(1000),
        total_reserve_2: Uint128::new(2000),
        total_lp_tokens: Uint128::new(3000),
        fee: Fee {
            lp_fee: 1,
            treasury_fee: 1,
            staker_fee: 1,
        },
    };
    STATE.save(&mut deps.storage, &state).unwrap();

    let res = query_fee(deps.as_ref()).unwrap();
    let fee_response: FeeResponse = from_binary(&res).unwrap();

    assert_eq!(fee_response.fee.lp_fee, 1);
    assert_eq!(fee_response.fee.treasury_fee, 1);
    assert_eq!(fee_response.fee.staker_fee, 1);
}
#[test]
fn test_query_pool_success() {
    // Tests if pool query returns correct pool information for a given chain ID
    let mut deps = mock_dependencies();
    let chain_id = "chain_id".to_string();

    let pool = Pool {
        chain:"chain".to_string(),
        reserve_1: Uint128::new(1000),
        reserve_2: Uint128::new(2000),
        pair: PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        },
       

    };
    POOLS.save(&mut deps.storage, &chain_id, &pool).unwrap();

    let res = query_pool(deps.as_ref(), chain_id.clone()).unwrap();
    let pool_response: PoolResponse = from_binary(&res).unwrap();

    assert_eq!(pool_response.pool.reserve_1, Uint128::new(1000));
    assert_eq!(pool_response.pool.reserve_2, Uint128::new(2000));
}
#[test]
fn test_query_all_pools_success() {
    // Tests if all pools query returns correct information for all pools
    let mut deps = mock_dependencies();
    let chain_id_1 = "chain_id_1".to_string();
    let chain_id_2 = "chain_id_2".to_string();
    let pool_1 =  Pool {
        chain:"chain".to_string(),
        reserve_1: Uint128::new(1000),
        reserve_2: Uint128::new(2000),
        pair: PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        },
       

    };
    let pool_2 = Pool {
        chain:"chain".to_string(),
        reserve_1: Uint128::new(4000),
        reserve_2: Uint128::new(5000),
        pair: PairInfo {
            token_1: TokenInfo {
                token: Token { id: "token_1".to_string() },
                token_type: TokenType::Native { denom: "token_1".to_string() },
            },
            token_2: TokenInfo {
                token: Token { id: "token_2".to_string() },
                token_type: TokenType::Native { denom: "token_2".to_string() },
            },
        },
       

    };
    POOLS.save(&mut deps.storage, &chain_id_1, &pool_1).unwrap();
    POOLS.save(&mut deps.storage, &chain_id_2, &pool_2).unwrap();

    let res = query_all_pools(deps.as_ref()).unwrap();
    let all_pools_response: AllPoolsResponse = from_binary(&res).unwrap();

    assert_eq!(all_pools_response.pools.len(), 2);
    assert_eq!(all_pools_response.pools[0].chain, chain_id_1);
    assert_eq!(all_pools_response.pools[0].pool.reserve_1, Uint128::new(1000));
    assert_eq!(all_pools_response.pools[0].pool.reserve_2, Uint128::new(2000));
    assert_eq!(all_pools_response.pools[1].chain, chain_id_2);
    assert_eq!(all_pools_response.pools[1].pool.reserve_1, Uint128::new(4000));
    assert_eq!(all_pools_response.pools[1].pool.reserve_2, Uint128::new(5000));
    assert_eq!(all_pools_response.pools[1].pool.lp_tokens, Uint128::new(6000));
}

}