#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128, Coin, BankMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{State, STATE, NFTS, SALES, RENTALS, EDITIONS};
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
    }

    #[test]
    fn create_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let nft = NFTS.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(nft.id, "nft1");
        assert_eq!(nft.metadata, "metadata");
        assert_eq!(nft.royalties, Some(10));
        assert_eq!(nft.owner, Addr::unchecked("creator"));
    }

    #[test]
    fn list_for_sale() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let list_for_sale_msg = ExecuteMsg::ListForSale { 
            id: "nft1".to_string(), 
            price: Uint128::new(1000) 
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), list_for_sale_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let sale_info = SALES.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(sale_info.price, Uint128::new(1000));
        assert_eq!(sale_info.royalty, Some(10));
    }

    #[test]
    fn buy_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let list_for_sale_msg = ExecuteMsg::ListForSale { 
            id: "nft1".to_string(), 
            price: Uint128::new(1000) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), list_for_sale_msg).unwrap();

        let buyer_info = mock_info("buyer", &coins(1000, "uscrt"));
        let buy_nft_msg = ExecuteMsg::BuyNFT { id: "nft1".to_string() };
        let res = execute(deps.as_mut(), env.clone(), buyer_info.clone(), buy_nft_msg).unwrap();
        assert_eq!(1, res.messages.len());

        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "creator");
                assert_eq!(amount[0], Coin { denom: "uscrt".to_string(), amount: Uint128::new(900) });
            }
            _ => panic!("Unexpected message type"),
        }

        let nft = NFTS.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(nft.owner, Addr::unchecked("buyer"));
    }

    #[test]
    fn rent_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let rent_nft_msg = ExecuteMsg::RentNFT { 
            id: "nft1".to_string(), 
            duration: 30 
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), rent_nft_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let rental_info = RENTALS.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(rental_info.0, Addr::unchecked("creator"));
        assert_eq!(rental_info.1, 30);
    }

    #[test]
    fn return_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let rent_nft_msg = ExecuteMsg::RentNFT { 
            id: "nft1".to_string(), 
            duration: 30 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), rent_nft_msg).unwrap();

        let return_nft_msg = ExecuteMsg::ReturnNFT { id: "nft1".to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), return_nft_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let rental_info = RENTALS.may_load(&deps.storage, "nft1".to_string());
        assert!(rental_info.is_err());
    }

    #[test]
    fn mint_edition() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let mint_edition_msg = ExecuteMsg::MintEdition { 
            id: "nft1".to_string(), 
            edition: 1 
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), mint_edition_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let edition = EDITIONS.load(&deps.storage, "nft1_1".to_string()).unwrap();
        assert_eq!(edition.0, Addr::unchecked("creator"));
    }

    #[test]
    fn update_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let update_nft_msg = ExecuteMsg::UpdateNFT { 
            id: "nft1".to_string(), 
            metadata: "new metadata".to_string(), 
            royalties: Some(20),
            new_metadata: todo!(), 
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), update_nft_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let nft = NFTS.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(nft.metadata, "new metadata");
        assert_eq!(nft.royalties, Some(20));
    }

    #[test]
    fn transfer_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_nft_msg = ExecuteMsg::CreateNFT { 
            id: "nft1".to_string(), 
            metadata: "metadata".to_string(), 
            royalties: Some(10) 
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), create_nft_msg).unwrap();

        let transfer_nft_msg = ExecuteMsg::TransferNFT { 
            id: "nft1".to_string(), 
            new_owner: "new_owner".to_string() 
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), transfer_nft_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let nft = NFTS.load(&deps.storage, "nft1".to_string()).unwrap();
        assert_eq!(nft.owner, Addr::unchecked("new_owner"));
    }

    #[test]
    fn withdraw_funds() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let withdraw_funds_msg = ExecuteMsg::WithdrawFunds {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), withdraw_funds_msg).unwrap();
        assert_eq!(1, res.messages.len());

        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "creator");
                assert!(!amount.is_empty());
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
