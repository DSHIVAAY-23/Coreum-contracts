#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, from_binary, Addr, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
    use crate::state::{Player, STATE, PLAYERS, GUILDS};
    use crate::error::ContractError;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let state = STATE.load(&deps.storage).unwrap();
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.token_address, Addr::unchecked("token"));
        assert_eq!(state.treasury_address, Addr::unchecked("treasury"));
        assert_eq!(state.breeding_contract, Addr::unchecked("breeding"));
        assert_eq!(state.asset_contract, Addr::unchecked("asset"));
    }

    #[test]
    fn reward_player() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the reward player message
        let reward_msg = ExecuteMsg::RewardPlayer { player: "player1".to_string(), amount: Uint128::new(100) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), reward_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure one message is sent

        // Verify that the player was rewarded correctly
        let player = PLAYERS.load(&deps.storage, &Addr::unchecked("player1")).unwrap();
        assert_eq!(player.rewards, 100);
    }

    #[test]
    fn collect_fee() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the collect fee message
        let collect_msg = ExecuteMsg::CollectFee { amount: Uint128::new(100) };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), collect_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure one message is sent

        // Verify the collect fee message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Bank(bank_msg) => {
                match bank_msg {
                    cosmwasm_std::BankMsg::Send { to_address, amount } => {
                        assert_eq!(to_address, "treasury");
                        assert_eq!(amount.len(), 1); // Ensure some amount is sent
                        assert_eq!(amount[0].amount, Uint128::new(100));
                    }
                    _ => panic!("Unexpected bank message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn breed_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the breed NFT message
        let breed_msg = ExecuteMsg::BreedNft { parent1: "parent1".to_string(), parent2: "parent2".to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), breed_msg).unwrap();
        assert_eq!(res.messages.len(), 1); // Ensure one message is sent

        // Verify the breed NFT message
        let msg = &res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Wasm(wasm_msg) => {
                match wasm_msg {
                    cosmwasm_std::WasmMsg::Execute { contract_addr, msg, funds } => {
                        assert_eq!(contract_addr, "breeding");
                        assert_eq!(funds.len(), 0); // Ensure no funds are sent
                    }
                    _ => panic!("Unexpected Wasm message type"),
                }
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[test]
    fn manage_guild_add_player() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Prepare the manage guild message
        let manage_guild_msg = ExecuteMsg::ManageGuild { guild: "guild1".to_string(), player: "player1".to_string(), action: "add".to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), manage_guild_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no messages are sent

        // Verify that the player was added to the guild
        let guild_members = GUILDS.load(&deps.storage, &Addr::unchecked("guild1")).unwrap();
        assert_eq!(guild_members.len(), 1);
        assert_eq!(guild_members[0], Addr::unchecked("player1"));
    }

    #[test]
    fn manage_guild_remove_player() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Instantiate the contract first
        let msg = InstantiateMsg {
            owner: "creator".to_string(),
            token_address: "token".to_string(),
            treasury_address: "treasury".to_string(),
            breeding_contract: "breeding".to_string(),
            asset_contract: "asset".to_string(),
        };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Add a player to the guild first
        let add_msg = ExecuteMsg::ManageGuild { guild: "guild1".to_string(), player: "player1".to_string(), action: "add".to_string() };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), add_msg).unwrap();

        // Prepare the manage guild message to remove the player
        let manage_guild_msg = ExecuteMsg::ManageGuild { guild: "guild1".to_string(), player: "player1".to_string(), action: "remove".to_string() };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), manage_guild_msg).unwrap();
        assert_eq!(res.messages.len(), 0); // Ensure no messages are sent

        // Verify that the player was removed from the guild
        let guild_members = GUILDS.load(&deps.storage, &Addr::unchecked("guild1")).unwrap();
        assert!(guild_members.is_empty());
    }
}