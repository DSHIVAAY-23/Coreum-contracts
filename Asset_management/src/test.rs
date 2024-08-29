#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_binary, Addr, BankMsg, Coin, CosmosMsg, StdError, Uint128,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute};
    use crate::msg::{InstantiateMsg, ExecuteMsg,  AssetType};
    use crate::state::{ AssetType as StateAssetType, ASSETS};
    use crate::error::ContractError;


    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

       
    }

    #[test]
    fn test_create_asset() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the CreateAsset message
        let create_asset_msg = ExecuteMsg::CreateAsset {
            total_supply: Uint128::new(1000),
            price: Uint128::new(100),
            uri: "http://example.com/asset1".to_string(),
            asset_type: AssetType::RealWorldAsset,
        };

        // Execute the create_asset function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), create_asset_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no messages are sent

        // Verify the asset was created
        let asset = ASSETS.load(&deps.storage, 1).unwrap();
        assert_eq!(asset.total_supply, Uint128::new(1000));
        assert_eq!(asset.price, Uint128::new(100));
        assert_eq!(asset.uri, "http://example.com/asset1");
        assert_eq!(asset.asset_type, StateAssetType::RealWorldAsset);
    }

    #[test]
    fn test_create_asset_not_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the CreateAsset message
        let create_asset_msg = ExecuteMsg::CreateAsset {
            total_supply: Uint128::new(1000),
            price: Uint128::new(100),
            uri: "http://example.com/asset1".to_string(),
            asset_type: AssetType::RealWorldAsset,
        };
        let non_owner_info = mock_info("not_owner", &[]);

        // Execute the create_asset function as a non-owner and expect an error
        let res = execute(deps.as_mut(), env.clone(), non_owner_info.clone(), create_asset_msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_transfer_ownership() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // First, create an asset
        let create_asset_msg = ExecuteMsg::CreateAsset {
            total_supply: Uint128::new(1000),
            price: Uint128::new(100),
            uri: "http://example.com/asset1".to_string(),
            asset_type: AssetType::RealWorldAsset,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_asset_msg).unwrap();

        // Prepare the TransferOwnership message
        let transfer_ownership_msg = ExecuteMsg::TransferOwnership {
            token_id: 1,
            to: "new_owner".to_string(),
            amount: Uint128::new(100),
        };

        // Execute the transfer_ownership function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), transfer_ownership_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no messages are sent

        // Verify the ownership was transferred
        let asset = ASSETS.load(&deps.storage, 1).unwrap();
        assert_eq!(asset.owner, Addr::unchecked("new_owner"));
    }

    #[test]
    fn test_transfer_ownership_not_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // First, create an asset
        let create_asset_msg = ExecuteMsg::CreateAsset {
            total_supply: Uint128::new(1000),
            price: Uint128::new(100),
            uri: "http://example.com/asset1".to_string(),
            asset_type: AssetType::RealWorldAsset,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_asset_msg).unwrap();

        // Prepare the TransferOwnership message
        let transfer_ownership_msg = ExecuteMsg::TransferOwnership {
            token_id: 1,
            to: "new_owner".to_string(),
            amount: Uint128::new(100),
        };
        let non_owner_info = mock_info("not_owner", &[]);

        // Execute the transfer_ownership function as a non-owner and expect an error
        let res = execute(deps.as_mut(), env.clone(), non_owner_info.clone(), transfer_ownership_msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_pay_out_dividends() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            symbol: "TAM".to_string(),
            subunit: "tam".to_string(),
            precision: 6,
            initial_amount: Uint128::new(1000000),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // First, create an asset
        let create_asset_msg = ExecuteMsg::CreateAsset {
            total_supply: Uint128::new(1000),
            price: Uint128::new(100),
            uri: "http://example.com/asset1".to_string(),
            asset_type: AssetType::RealWorldAsset,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_asset_msg).unwrap();

        // Prepare the PayoutDividends message
        let payout_dividends_msg = ExecuteMsg::PayoutDividends {
            token_id: 1,
        };

        // Execute the payout_dividends function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), payout_dividends_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure a bank message is sent

        // Verify the bank message
        let msg = &res.messages[0].msg;
        match msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "creator"); // Assuming payout to creator
                assert_eq!(amount[0].denom, "tam");
                assert_eq!(amount[0].amount, Uint128::new(100)); // Example value
            }
            _ => panic!("Expected a BankMsg::Send message"),
        }
    }
}
