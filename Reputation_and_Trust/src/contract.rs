use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, UserReputation, BALANCES, REPUTATIONS, STATE};
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cosmwasm_std::{
    entry_point, to_binary, Binary,  Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response,  StdResult, Uint128,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "reputation-trust";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut<CoreumQueries>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let issue_msg = CoreumMsg::AssetFT(assetft::Msg::Issue {
        symbol: msg.symbol,
        subunit: msg.subunit.clone(),
        precision: msg.precision,
        initial_amount: msg.initial_amount,
        description: None,
        features: Some(vec![0]), // 0 - minting
        burn_rate: Some("0".into()),
        send_commission_rate: Some("0.1".into()), // 10% commission for sending
    });

    let denom = format!("{}-{}", msg.subunit, env.contract.address).to_lowercase();
    let state = State {
        owner: info.sender.clone(),
        denom,
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", state.owner.to_string())
        .add_attribute("denom", state.denom)
        .add_message(issue_msg))
}

#[entry_point]
pub fn execute(
    deps: DepsMut<CoreumQueries>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::UpdateReputation { user, reputation } => {
            update_reputation(deps, info, user, reputation)
        }
        ExecuteMsg::ResetReputation { user } => reset_reputation(deps, info, user),
        ExecuteMsg::Transfer { recipient, amount } => transfer(deps, info, recipient, amount),
    }
}

fn update_reputation(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    user: String,
    reputation: u64,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let user_addr = deps.api.addr_validate(&user)?;
    let mut user_reputation =
        REPUTATIONS.may_load(deps.storage, &user_addr)?.unwrap_or(UserReputation { reputation: 0 });

    user_reputation.reputation = reputation;
    REPUTATIONS.save(deps.storage, &user_addr, &user_reputation)?;

    Ok(Response::new()
        .add_attribute("method", "update_reputation")
        .add_attribute("user", user))
}

fn reset_reputation(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    user: String,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let user_addr = deps.api.addr_validate(&user)?;
    REPUTATIONS.remove(deps.storage, &user_addr);

    Ok(Response::new()
        .add_attribute("method", "reset_reputation")
        .add_attribute("user", user))
}

pub fn transfer(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    let sender_addr = info.sender.clone();

    let mut sender_balance =
        BALANCES.may_load(deps.storage, &sender_addr)?.unwrap_or(Uint128::zero());
    if sender_balance < amount {
        return Err(ContractError::InsufficientBalance {});
    }

    sender_balance = sender_balance.checked_sub(amount).map_err(|_| ContractError::Overflow {})?;
    BALANCES.save(deps.storage, &sender_addr, &sender_balance)?;

    let mut recipient_balance =  BALANCES.may_load(deps.storage, &recipient_addr)?.unwrap_or(Uint128::zero());
    recipient_balance = recipient_balance.checked_add(amount).map_err(|_| ContractError::Overflow {})?;
    BALANCES.save(deps.storage, &recipient_addr, &recipient_balance)?;

    Ok(Response::new()
        .add_attribute("method", "transfer")
        .add_attribute("from", sender_addr.to_string())
        .add_attribute("to", recipient)
        .add_attribute("amount", amount.to_string()))
}

#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Token {} => token(deps),
        QueryMsg::GetReputation { user } => query_reputation(deps, user),
        QueryMsg::GetBalance { user } => query_balance(deps, user),
    }
}

fn token(deps: Deps<CoreumQueries>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let request: QueryRequest<CoreumQueries> =
        CoreumQueries::AssetFT(assetft::Query::Token { denom: state.denom }).into();
    let res: assetft::TokenResponse = deps.querier.query(&request)?;
    to_binary(&res)
}

fn query_reputation(deps: Deps<CoreumQueries>, user: String) -> StdResult<Binary> {
    let user_addr = deps.api.addr_validate(&user)?;
    let reputation =
        REPUTATIONS.may_load(deps.storage, &user_addr)?.unwrap_or(UserReputation { reputation: 0 });
    to_binary(&reputation)
}

fn query_balance(deps: Deps<CoreumQueries>, user: String) -> StdResult<Binary> {
    let user_addr = deps.api.addr_validate(&user)?;
    let balance = BALANCES.may_load(deps.storage, &user_addr)?.unwrap_or(Uint128::zero());
    to_binary(&balance)
}
