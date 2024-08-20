use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Subscription, State, STATE, SUBSCRIPTIONS};
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "subscription-service";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: deps.api.addr_validate(&msg.owner)?,
        subscription_cost: msg.subscription_cost,
        subscription_period: msg.subscription_period,
    };
    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Subscribe {denom} => subscribe(deps, env, info,denom),
        ExecuteMsg::Renew {denom} => renew(deps, env, info,denom),
        ExecuteMsg::WithdrawFunds {} => withdraw_funds(deps, info),
    }
}

fn subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

     // Check if the user has sent enough funds
     let sent_funds = info.funds.iter().find(|c| c.denom == denom).unwrap_or(&Coin { denom: denom.clone(), amount: Uint128::zero() }).amount;
     check_funds(sent_funds, state.subscription_cost)?;
     if sent_funds < state.subscription_cost {
         return Err(ContractError::InsufficientFunds {});
     }

     let end_time = match SUBSCRIPTIONS.may_load(deps.storage, info.sender.clone())? {
        Some(existing_sub) => {
            // If existing subscription, extend from the current end_time
            if env.block.time > existing_sub.end_time {
                env.block.time.plus_seconds(state.subscription_period)
            } else {
                existing_sub.end_time.plus_seconds(state.subscription_period)
            }
        }
        None => env.block.time.plus_seconds(state.subscription_period),
    };
    let subscription = Subscription {
        user: info.sender.clone(),
        end_time,
    };

    SUBSCRIPTIONS.save(deps.storage, info.sender.clone(), &subscription)?;

    Ok(Response::new()
        .add_attribute("method", "subscribe")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("end_time", end_time.to_string()))
}

fn renew(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    // Check if the user has sent enough funds
    let sent_funds = info.funds.iter().find(|c| c.denom == denom).unwrap_or(&Coin { denom: denom.clone(), amount: Uint128::zero() }).amount;
    if sent_funds < state.subscription_cost {
        return Err(ContractError::InsufficientFunds {});
    }

    let mut subscription = SUBSCRIPTIONS.load(deps.storage, info.sender.clone())?;
    if env.block.time > subscription.end_time {
        subscription.end_time = env.block.time.plus_seconds(state.subscription_period);
    } else {
        subscription.end_time = subscription.end_time.plus_seconds(state.subscription_period);
    }
    SUBSCRIPTIONS.save(deps.storage, info.sender.clone(), &subscription)?;

    Ok(Response::new()
        .add_attribute("method", "renew")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("end_time", subscription.end_time.to_string()))
}

fn withdraw_funds(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    let balance = deps.querier.query_all_balances(&state.owner)?;
    let withdraw_msg = BankMsg::Send {
        to_address: state.owner.to_string(),
        amount: balance,
    };
    Ok(Response::new()
        .add_attribute("method", "withdraw_funds")
        .add_message(CosmosMsg::Bank(withdraw_msg)))
}

#[entry_point]
pub fn query(
    deps: Deps<CoreumQueries>,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsSubscribed { address } => to_binary(&query_is_subscribed(deps, env, address)?),
    }
}

fn query_is_subscribed(
    deps: Deps<CoreumQueries>,
    env: Env,
    address: String,
) -> StdResult<bool> {
    // Validate address format
    let addr = deps.api.addr_validate(&address).map_err(|_| StdError::generic_err("Invalid address format"))?;
    let subscription = SUBSCRIPTIONS.may_load(deps.storage, addr)?;
    if let Some(sub) = subscription {
        if env.block.time < sub.end_time {
            return Ok(true);
        }
    }
    Ok(false)
}
fn check_funds(sent_funds: Uint128, required_amount: Uint128) -> Result<(), ContractError> {
    const TOLERANCE: u128 = 1; // Define tolerance level

    if sent_funds < required_amount || sent_funds.u128().checked_sub(required_amount.u128()).unwrap_or_default() > TOLERANCE {
        return Err(ContractError::InsufficientFunds {});
    }
    Ok(())
}