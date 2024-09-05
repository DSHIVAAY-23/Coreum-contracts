#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, DERIVATIVES, STATE};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "core"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "core_token".to_string(),
            oracle_address: "oracle_address".to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.token_address, Addr::unchecked("core_token"));
        assert_eq!(state.oracle_address, Addr::unchecked("oracle_address"));
    }

    #[test]
    fn test_create_derivative() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "core"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "core_token".to_string(),
            oracle_address: "oracle_address".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the create_derivative message
        let create_msg = ExecuteMsg::CreateDerivative {
            derivative_type: "futures".to_string(),
            underlying_asset: "BTC".to_string(),
            amount: Uint128::new(1000),
            price: Uint128::new(50000),
            expiry: Some(172800), // 2 days
        };

        // Execute the create_derivative function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), create_msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Verify that the derivative was created
        let derivative = DERIVATIVES.load(&deps.storage, 1).unwrap();
        assert_eq!(derivative.creator, Addr::unchecked("creator"));
        assert_eq!(derivative.underlying_asset, "BTC");
        assert_eq!(derivative.amount, Uint128::new(1000));
        assert_eq!(derivative.price, Uint128::new(50000));
        assert_eq!(derivative.expiry, Some(172800));
    }

    #[test]
    fn test_trade_derivative() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "core"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "core_token".to_string(),
            oracle_address: "oracle_address".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a derivative
        let create_msg = ExecuteMsg::CreateDerivative {
            derivative_type: "futures".to_string(),
            underlying_asset: "BTC".to_string(),
            amount: Uint128::new(1000),
            price: Uint128::new(50000),
            expiry: Some(172800), // 2 days
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_msg).unwrap();

        // Prepare the trade_derivative message
        let trade_msg = ExecuteMsg::TradeDerivative {
            id: 1,
            buyer: "buyer".to_string(),
            amount: Uint128::new(500),
        };
        let buyer_info = mock_info("buyer", &coins(500, "core"));

        // Execute the trade_derivative function
        let res = execute(deps.as_mut(), env.clone(), buyer_info.clone(), trade_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // Verify that the amount has been updated
        let derivative = DERIVATIVES.load(&deps.storage, 1).unwrap();
        assert_eq!(derivative.amount, Uint128::new(500));
    }

    #[test]
    fn test_settle_derivative() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "core"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "core_token".to_string(),
            oracle_address: "oracle_address".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a derivative
        let create_msg = ExecuteMsg::CreateDerivative {
            derivative_type: "futures".to_string(),
            underlying_asset: "BTC".to_string(),
            amount: Uint128::new(1000),
            price: Uint128::new(50000),
            expiry: Some(172800), // 2 days
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_msg).unwrap();

        // Prepare the settle_derivative message
        let settle_msg = ExecuteMsg::SettleDerivative { id: 1 };

        // Execute the settle_derivative function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), settle_msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // Verify that the derivative is removed after settlement
        let derivative = DERIVATIVES.may_load(&deps.storage, 1).unwrap();
        assert!(derivative.is_none());
    }
}
