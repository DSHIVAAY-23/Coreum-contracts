use crate::error::ContractError;
use crate::msg::{AssetMsg, BreedingMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, Player, STATE, PLAYERS, GUILDS};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg, CosmosMsg, WasmMsg, WasmQuery,
};
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "play-to-earn";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = State {
        owner: deps.api.addr_validate(&msg.owner)?,
        token_address: deps.api.addr_validate(&msg.token_address)?,
        treasury_address: deps.api.addr_validate(&msg.treasury_address)?,
        breeding_contract: deps.api.addr_validate(&msg.breeding_contract)?,
       asset_contract: deps.api.addr_validate(&msg.asset_contract)?,
    };
    STATE.save(deps.storage, &state)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut<CoreumQueries>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::RewardPlayer { player, amount } => reward_player(deps, env, info, player, amount),
        ExecuteMsg::CollectFee { amount } => collect_fee(deps, info, amount),
        ExecuteMsg::BreedNft { parent1, parent2 } => breed_nft(deps, info, parent1, parent2),
        ExecuteMsg::BattleOutcome { player, result } => battle_outcome(deps, info, player, result),
        ExecuteMsg::ManageGuild { guild, player, action } => manage_guild(deps, info, guild, player, action),
        ExecuteMsg::ChargeTransactionFee { amount } => charge_transaction_fee(deps, info, amount),
        ExecuteMsg::SellAsset { asset_id, amount } => sell_asset(deps, info, asset_id, amount),
        ExecuteMsg::WithdrawTreasury { amount } => withdraw_treasury(deps, info, amount),
    }
}

fn reward_player(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    _info: MessageInfo,
    player: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let player_addr = deps.api.addr_validate(&player)?;

    let mut player = PLAYERS.may_load(deps.storage, &player_addr)?.unwrap_or(Player {
        address: player_addr.clone(),
        rewards: 0,
    });

    player.rewards += amount.u128();
    PLAYERS.save(deps.storage, &player_addr, &player)?;

    let reward_msg = CoreumMsg::AssetFT(assetft::Msg::Mint {
        coin: Coin {
            denom: state.token_address.to_string(),
            amount,
        },
    });

    Ok(Response::new()
        .add_attribute("method", "reward_player")
        .add_message(CosmosMsg::Custom(reward_msg)))
}

fn collect_fee(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    let collect_msg = BankMsg::Send {
        to_address: state.treasury_address.to_string(),
        amount: vec![Coin { denom: state.token_address.to_string(), amount }],
    };

    Ok(Response::new()
        .add_attribute("method", "collect_fee")
        .add_message(CosmosMsg::Bank(collect_msg)))
}


fn breed_nft(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    parent1: String,
    parent2: String,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let breed_msg = WasmMsg::Execute {
        contract_addr: state.breeding_contract.to_string(),
        msg: to_binary(&BreedingMsg {
            parent1: parent1.clone(),
            parent2: parent2.clone(),
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("method", "breed_nft")
        .add_attribute("parent1", parent1)
        .add_attribute("parent2", parent2)
        .add_message(CosmosMsg::Wasm(breed_msg)))
}
fn battle_outcome(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    player: String,
    result: String,
) -> Result<Response<CoreumMsg>, ContractError> {
    let player_addr = deps.api.addr_validate(&player)?;
    let mut player = PLAYERS.may_load(deps.storage, &player_addr)?.unwrap_or(Player {
        address: player_addr.clone(),
        rewards: 0,
    });

    let reward_amount = match result.as_str() {
        "win" => Uint128::new(100),
        "lose" => Uint128::new(50),
        _ => Uint128::new(0),
    };

    player.rewards += reward_amount.u128();
    PLAYERS.save(deps.storage, &player_addr, &player)?;

    let reward_msg = CoreumMsg::AssetFT(assetft::Msg::Mint {
        coin: Coin {
            denom: "token".to_string(),
            amount: reward_amount,
        },
    });

    Ok(Response::new()
        .add_attribute("method", "battle_outcome")
        .add_attribute("result", result)
        .add_message(CosmosMsg::Custom(reward_msg)))
}


fn manage_guild(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    guild: String,
    player: String,
    action: String,
) -> Result<Response<CoreumMsg>, ContractError> {
    let guild_addr = deps.api.addr_validate(&guild)?;
    let player_addr = deps.api.addr_validate(&player)?;
    
    match action.as_str() {
        "add" => {
            let mut members = GUILDS.may_load(deps.storage, &guild_addr)?.unwrap_or(Vec::new());
            members.push(player_addr.clone());
            GUILDS.save(deps.storage, &guild_addr, &members)?;
        },
        "remove" => {
            let mut members = GUILDS.may_load(deps.storage, &guild_addr)?.unwrap_or(Vec::new());
            members.retain(|x| x != &player_addr);
            GUILDS.save(deps.storage, &guild_addr, &members)?;
        },
        _ => return Err(ContractError::InvalidAction {}),
    }

    Ok(Response::new()
        .add_attribute("method", "manage_guild")
        .add_attribute("guild", guild)
        .add_attribute("player", player)
        .add_attribute("action", action))
}

fn charge_transaction_fee(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    let fee_msg = BankMsg::Send {
        to_address: state.treasury_address.to_string(),
        amount: vec![Coin { denom: state.token_address.to_string(), amount }],
    };

    Ok(Response::new()
        .add_attribute("method", "charge_transaction_fee")
        .add_message(CosmosMsg::Bank(fee_msg)))
}

fn sell_asset(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    asset_id: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let sell_msg = WasmMsg::Execute {
        contract_addr: state.asset_contract.to_string(),
        msg: to_binary(&AssetMsg {
            asset_id: asset_id.clone(),
            amount,
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_attribute("method", "sell_asset")
        .add_attribute("asset_id", asset_id)
        .add_attribute("amount", amount.to_string())
        .add_message(CosmosMsg::Wasm(sell_msg)))
}

fn withdraw_treasury(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let withdraw_msg = BankMsg::Send {
        to_address: state.owner.to_string(),
        amount: vec![Coin { denom: state.token_address.to_string(), amount }],
    };

    Ok(Response::new()
        .add_attribute("method", "withdraw_treasury")
        .add_message(CosmosMsg::Bank(withdraw_msg)))
}

#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPlayerRewards { player } => to_binary(&query_player_rewards(deps, player)?),
        QueryMsg::GetTreasuryBalance {} => to_binary(&query_treasury_balance(deps)?),
        QueryMsg::GetGuildMembers { guild } => to_binary(&query_guild_members(deps, guild)?),
    }
}

fn query_player_rewards(deps: Deps<CoreumQueries>, player: String) -> StdResult<Uint128> {
    let player_addr = deps.api.addr_validate(&player)?;
    let player = PLAYERS.load(deps.storage, &player_addr)?;
    Ok(Uint128::from(player.rewards))
}

fn query_treasury_balance(deps: Deps<CoreumQueries>) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    let balance = deps.querier.query_balance(&state.treasury_address, "token")?;
    Ok(balance.amount)
}

fn query_guild_members(deps: Deps<CoreumQueries>, guild: String) -> StdResult<Vec<String>> {
    let guild_addr = deps.api.addr_validate(&guild)?;
    let members = GUILDS.load(deps.storage, &guild_addr)?;
    Ok(members.into_iter().map(|addr| addr.to_string()).collect())
}
