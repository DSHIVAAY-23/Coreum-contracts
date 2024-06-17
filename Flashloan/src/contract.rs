use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CustomMsg, RequestFlashLoan, RepayFlashLoan};
use crate::state::{State, STATE};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, BankMsg, Coin, StdError,
};
use cw2::set_contract_version;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};

const CONTRACT_NAME: &str = "flash-loan";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let state = State {
        owner: deps.api.addr_validate(&msg.owner)?,
        lending_pool: deps.api.addr_validate(&msg.lending_pool)?,
    };
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CustomMsg>, ContractError> {
    match msg {
        ExecuteMsg::RequestFlashLoan { token, amount, collateral } => request_flash_loan(deps, info, token, amount, collateral),
        ExecuteMsg::ExecuteOperation { token, amount, premium } => execute_operation(deps, info, token, amount, premium),
        ExecuteMsg::Withdraw { token } => withdraw(deps, info, token),
    }
}

pub fn request_flash_loan(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
    amount: Uint128,
    collateral: Uint128,
) -> Result<Response<CustomMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    // Transfer collateral to the contract
    let collateral_transfer = BankMsg::Send {
        to_address: state.lending_pool.clone().into(),
        amount: vec![Coin { denom: token.clone(), amount: collateral }],
    };

    // Custom logic to handle flash loan request
    let flash_loan_request = CustomMsg::RequestFlashLoan(RequestFlashLoan {
        recipient: info.sender.to_string(),
        token: token.clone(),
        amount,
    });

    Ok(Response::new()
        .add_attribute("method", "request_flash_loan")
        .add_message(CosmosMsg::Bank(collateral_transfer))
        .add_message(CosmosMsg::Custom(flash_loan_request)))
}

pub fn execute_operation(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
    amount: Uint128,
    premium: Uint128,
) -> Result<Response<CustomMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    // Ensure repayment of the loan with the premium
    let repay_amount = amount + premium;
    let repay_msg = CustomMsg::RepayFlashLoan(RepayFlashLoan {
        sender: info.sender.to_string(),
        token: token.clone(),
        amount: repay_amount,
    });

    // Check if there are sufficient funds to repay the loan plus premium
    let balance = deps.querier.query_balance(&info.sender, &token)?;
    if balance.amount < repay_amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient funds to repay loan with premium")));
    }

    // Return collateral if loan is repaid
    let return_collateral = BankMsg::Send {
        to_address: info.sender.into(),
        amount: vec![Coin { denom: token.clone(), amount: repay_amount }],
    };

    Ok(Response::new()
        .add_attribute("method", "execute_operation")
        .add_message(CosmosMsg::Custom(repay_msg))
        .add_message(CosmosMsg::Bank(return_collateral)))
}

fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
) -> Result<Response<CustomMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps.querier.query_balance(&state.owner, &token)?;
    let withdraw_msg = BankMsg::Send {
        to_address: state.owner.into(),
        amount: vec![balance],
    };

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_message(CosmosMsg::Bank(withdraw_msg)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
    QueryMsg::LoanInfo{}=>loan_info(deps),
    QueryMsg::GetBalance { token } => todo!(), }
}

fn loan_info(deps: Deps<CoreumQueries>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_binary(&state)
}


