#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{Content, Subscription, COMMUNITY_FUND, CONTENT, SUBSCRIPTIONS};
    use coreum_wasm_sdk::core::{CoreumMsg};

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let community_fund = COMMUNITY_FUND.load(&deps.storage).unwrap();
        assert_eq!(community_fund, Uint128::new(1000));
    }

    #[test]
    fn test_create_content() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the create content message
        let create_content_msg = ExecuteMsg::CreateContent {
            uri: "ipfs://content_hash".to_string(),
            access_level: Uint128::new(50),
        };

        // Execute the create content function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), create_content_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the content was created correctly
        let content = CONTENT.load(&deps.storage, "creator").unwrap();
        assert_eq!(content.creator, Addr::unchecked("creator"));
        assert_eq!(content.uri, "ipfs://content_hash".to_string());
        assert_eq!(content.access_level, Uint128::new(50));
        assert_eq!(content.tips, Uint128::zero());
    }

    #[test]
    fn test_tip_content() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create content first
        let create_content_msg = ExecuteMsg::CreateContent {
            uri: "ipfs://content_hash".to_string(),
            access_level: Uint128::new(50),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_content_msg).unwrap();

        // Prepare the tip content message
        let tip_content_msg = ExecuteMsg::TipContent {
            content_id: "creator".to_string(),
            amount: Uint128::new(100),
        };
        let tipper_info = mock_info("tipper", &coins(100, "coreum"));

        // Execute the tip content function
        let res = execute(deps.as_mut(), env.clone(), tipper_info.clone(), tip_content_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the tip was added correctly
        let content = CONTENT.load(&deps.storage, "creator").unwrap();
        assert_eq!(content.tips, Uint128::new(100));
    }

    #[test]
    fn test_subscribe() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the subscribe message
        let subscribe_msg = ExecuteMsg::Subscribe {
            creator: Addr::unchecked("creator"),
            amount: Uint128::new(500),
            duration: 60 * 60 * 24 * 30, // 30 days
        };
        let subscriber_info = mock_info("subscriber", &coins(500, "coreum"));

        // Execute the subscribe function
        let res = execute(deps.as_mut(), env.clone(), subscriber_info.clone(), subscribe_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no extra messages are sent

        // Verify that the subscription was created correctly
        let subscription = SUBSCRIPTIONS.load(&deps.storage, (Addr::unchecked("subscriber"), Addr::unchecked("creator"))).unwrap();
        assert_eq!(subscription.subscriber, Addr::unchecked("subscriber"));
        assert_eq!(subscription.creator, Addr::unchecked("creator"));
        assert_eq!(subscription.amount, Uint128::new(500));
        assert_eq!(subscription.expiry, env.block.time.seconds() + 60 * 60 * 24 * 30);
    }

    #[test]
    fn test_access_content() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create content first
        let create_content_msg = ExecuteMsg::CreateContent {
            uri: "ipfs://content_hash".to_string(),
            access_level: Uint128::new(50),
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_content_msg).unwrap();

        // Prepare the access content message
        let access_content_msg = ExecuteMsg::AccessContent {
            content_id: "creator".to_string(),
        };
        let subscriber_info = mock_info("subscriber", &coins(50, "coreum"));

        // Execute the access content function
        let res = execute(deps.as_mut(), env.clone(), subscriber_info.clone(), access_content_msg.clone());
        assert!(res.is_ok()); // Ensure the content is accessible

        // Check access failure due to insufficient funds
        let insufficient_info = mock_info("subscriber", &coins(10, "coreum"));
        let res = execute(deps.as_mut(), env.clone(), insufficient_info.clone(), access_content_msg.clone());
        assert!(res.is_err()); // Ensure an error is returned for insufficient access level
    }

    #[test]
    fn test_distribute_funds() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "coreum"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            initial_fund: Uint128::new(1000),
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create some subscriptions
        let subscribe_msg = ExecuteMsg::Subscribe {
            creator: Addr::unchecked("creator"),
            amount: Uint128::new(500),
            duration: 60 * 60 * 24 * 30, // 30 days
        };
        let subscriber_info = mock_info("subscriber", &coins(500, "coreum"));
        execute(deps.as_mut(), env.clone(), subscriber_info.clone(), subscribe_msg).unwrap();

        // Prepare the distribute funds message
        let distribute_funds_msg = ExecuteMsg::DistributeFunds {};

        // Execute the distribute funds function
        let res = execute(deps.as_mut(), env.clone(), info.clone(), distribute_funds_msg).unwrap();
        assert!(res.messages.len() > 0); // Ensure messages are sent for fund distribution

        // Verify the community fund is updated
        let community_fund = COMMUNITY_FUND.load(&deps.storage).unwrap();
        assert!(community_fund < Uint128::new(1000)); // Fund should be reduced
    }
}
