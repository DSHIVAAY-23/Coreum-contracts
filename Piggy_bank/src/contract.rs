use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Deposit, State, STATE, DEPOSITS};
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg, CosmosMsg,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "piggy-bank";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError>  {
    let state = State {
        owner: deps.api.addr_validate(&msg.owner)?,
        lock_time: 120, // default to 2 minutes
        tokens_in: Uint128::zero(),
        tokens_out: Uint128::zero(),
        deposit_count: 0,
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
) -> Result<Response, ContractError>  {
    match msg {
        ExecuteMsg::DepositTokens { amount } => deposit_tokens(deps, env, info, amount),
        ExecuteMsg::WithdrawTokens { deposit_id, denom } => Ok(withdraw_tokens(deps, env, info, deposit_id, denom)?), // Updated call
        ExecuteMsg::SetLockTime { lock_time } => set_lock_time(deps, info, lock_time),
        ExecuteMsg::SetNewOwner { new_owner } => set_new_owner(deps, info, new_owner),
    }
}

fn deposit_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError>   {
    let mut state = STATE.load(deps.storage)?;
    let deposit_id = state.deposit_count;

    let new_deposit = Deposit {
        deposit_id,
        amount,
        from: info.sender.clone(),
        deposit_time: env.block.height,
        unlock_time: env.block.height + state.lock_time,
    };

    DEPOSITS.save(deps.storage, deposit_id, &new_deposit)?;

    state.tokens_in += amount;
    state.deposit_count += 1;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "deposit_tokens")
        .add_attribute("amount", amount.to_string()))
}


fn withdraw_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deposit_id: u64,
    denom: String, 
) -> Result<Response, ContractError>  {
    let state = STATE.load(deps.storage)?;
    let deposit = DEPOSITS.load(deps.storage, deposit_id)?;

    if env.block.height < deposit.unlock_time {
        return Err(ContractError::Unlock {});
    }

    if deposit.from != info.sender {
        return Err(ContractError::InvalidOwner {});
    }

    let send_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom, // Use the provided denom
            amount: deposit.amount,
        }],
    };

    DEPOSITS.remove(deps.storage, deposit_id);

    let mut new_state = state.clone();
    new_state.tokens_out += deposit.amount;
    STATE.save(deps.storage, &new_state)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw_tokens")
        .add_message(CosmosMsg::Bank(send_msg)))
}

fn set_lock_time(
    deps: DepsMut,
    info: MessageInfo,
    lock_time: u64,
) ->Result<Response, ContractError>  {
    let mut state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    state.lock_time = lock_time;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "set_lock_time")
        .add_attribute("lock_time", lock_time.to_string()))
}

fn set_new_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError>  {
    let mut state = STATE.load(deps.storage)?;

    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    state.owner = deps.api.addr_validate(&new_owner)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "set_new_owner")
        .add_attribute("new_owner", new_owner))
}

#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTokensIn {} => to_binary(&query_tokens_in(deps)?),
        QueryMsg::GetTokensOut {} => to_binary(&query_tokens_out(deps)?),
        QueryMsg::GetBalance {} => to_binary(&query_balance(deps)?),
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
        QueryMsg::GetDeposit { deposit_id } => to_binary(&query_deposit(deps, deposit_id)?),
    }
}

fn query_tokens_in(deps: Deps<CoreumQueries>) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    Ok(state.tokens_in)
}

fn query_tokens_out(deps: Deps<CoreumQueries>) -> StdResult<Uint128> {
    let state = STATE.load(deps.storage)?;
    Ok(state.tokens_out)
}

fn query_balance(deps: Deps<CoreumQueries>) -> StdResult<Uint128> {
    let address = deps.api.addr_humanize(&deps.api.addr_canonicalize("contract address")?)?;
    let balance = deps.querier.query_all_balances(address)?;
    let mut total_balance = Uint128::zero();
    for coin in balance {
        total_balance += coin.amount;
    }
    Ok(total_balance)
}

fn query_state(deps: Deps<CoreumQueries>) -> StdResult<State> {
    STATE.load(deps.storage)
}

fn query_deposit(deps: Deps<CoreumQueries>, deposit_id: u64) -> StdResult<Deposit> {
    DEPOSITS.load(deps.storage, deposit_id)
}
