use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg, CosmosMsg,
};
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "loyalty-program";
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
        ExecuteMsg::EarnTokens { customer, amount } => earn_tokens(deps, env, info, customer, amount),
        ExecuteMsg::RedeemTokens { customer, amount } => redeem_tokens(deps, env, info, customer, amount),
        ExecuteMsg::TransferTokens { from, to, amount } => transfer_tokens(deps, env, info, from, to, amount),
        ExecuteMsg::WithdrawTokens { amount } => withdraw_tokens(deps, info, amount),
    }
}

fn earn_tokens(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    _info: MessageInfo,
    customer: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let _customer_addr = deps.api.addr_validate(&customer)?;

    let earn_msg = CoreumMsg::AssetFT(assetft::Msg::Mint {
        coin: Coin {
            denom: state.token_address.to_string(),
            amount,
        },
    });

    Ok(Response::new()
        .add_attribute("method", "earn_tokens")
        .add_message(CosmosMsg::Custom(earn_msg)))
}

fn redeem_tokens(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    _info: MessageInfo,
    customer: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let _customer_addr = deps.api.addr_validate(&customer)?;

    let redeem_msg = CoreumMsg::AssetFT(assetft::Msg::Burn {
        coin: Coin {
            denom: state.token_address.to_string(),
            amount,
        },
    });

    Ok(Response::new()
        .add_attribute("method", "redeem_tokens")
        .add_message(CosmosMsg::Custom(redeem_msg)))
}

fn transfer_tokens(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    _info: MessageInfo,
    from: String,
    to: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    let _from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;

    let transfer_msg = BankMsg::Send {
        to_address: to_addr.to_string(),
        amount: vec![Coin {
            denom: state.token_address.to_string(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_attribute("method", "transfer_tokens")
        .add_message(CosmosMsg::Bank(transfer_msg)))
}

fn withdraw_tokens(
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
        .add_attribute("method", "withdraw_tokens")
        .add_message(CosmosMsg::Bank(withdraw_msg)))
}

#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { customer } => to_binary(&query_balance(deps, customer)?),
    }
}

fn query_balance(deps: Deps<CoreumQueries>, customer: String) -> StdResult<Uint128> {
    let customer_addr = deps.api.addr_validate(&customer)?;
    let balance = deps.querier.query_balance(&customer_addr, "token")?;
    Ok(balance.amount)
}
