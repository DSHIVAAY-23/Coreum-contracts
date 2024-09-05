use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, Derivative, STATE, DERIVATIVES};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg, CosmosMsg,
};
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "derivative-platform";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the contract with the owner, token address, and oracle address.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Initialize state with owner, token address, and oracle address
    let state = State {
        owner: deps.api.addr_validate(&msg.owner)?,
        token_address: deps.api.addr_validate(&msg.token_address)?,
        oracle_address: deps.api.addr_validate(&msg.oracle_address)?,
    };
    STATE.save(deps.storage, &state)?;

    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string()))
}

/// Handles the different execution messages to create, trade, and settle derivatives.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateDerivative { 
            derivative_type, 
            underlying_asset, 
            amount, 
            price, 
            expiry 
        } => create_derivative(deps, env, info, derivative_type, underlying_asset, amount, price, expiry),
        ExecuteMsg::TradeDerivative { id, buyer, amount } => trade_derivative(deps, env, info, id, buyer, amount),
        ExecuteMsg::SettleDerivative { id } => settle_derivative(deps, env, info, id),
    }
}

/// Creates a new derivative and saves it to storage.
fn create_derivative(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    derivative_type: String,
    underlying_asset: String,
    amount: Uint128,
    price: Uint128,
    expiry: Option<u64>,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Load state to access the token address
    let state = STATE.load(deps.storage)?;

    // Generate a unique ID for the new derivative
    let id = DERIVATIVES
        .range(deps.storage, None, None, cosmwasm_std::Order::Descending)
        .next()
        .map(|item| match item {
            Ok((key, _)) => key + 1,
            Err(_) => 1,
        })
        .unwrap_or(1);

    // Create a new derivative
    let derivative = Derivative {
        id,
        creator: info.sender.clone(),
        derivative_type,
        underlying_asset,
        amount,
        price,
        expiry,
    };

    // Save the derivative to storage
    DERIVATIVES.save(deps.storage, id, &derivative)?;

    Ok(Response::new()
        .add_attribute("method", "create_derivative")
        .add_attribute("creator", info.sender.to_string())
        .add_attribute("id", id.to_string()))
}

/// Trades a specified amount of the derivative to a buyer.
fn trade_derivative(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    buyer: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Load state to access the token address
    let state = STATE.load(deps.storage)?;
    
    // Load the derivative from storage
    let mut derivative = DERIVATIVES.load(deps.storage, id)?;

    // Ensure there is enough amount in the derivative
    let new_amount = derivative.amount.checked_sub(amount).map_err(|_| ContractError::InsufficientFunds {})?;
    derivative.amount = new_amount;

    // Remove the derivative if the amount is zero, otherwise save the updated derivative
    if derivative.amount == Uint128::zero() {
        DERIVATIVES.remove(deps.storage, id);
    } else {
        DERIVATIVES.save(deps.storage, id, &derivative)?;
    }

    // Validate buyer's address
    let buyer_addr = deps.api.addr_validate(&buyer)?;

    // Create a transfer message to send the derivative amount to the buyer
    let transfer_msg = BankMsg::Send {
        to_address: buyer_addr.to_string(),
        amount: vec![Coin {
            denom: state.token_address.to_string(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_attribute("method", "trade_derivative")
        .add_attribute("buyer", buyer)
        .add_message(CosmosMsg::Bank(transfer_msg)))
}

/// Settles a derivative by querying the price from the oracle and transferring the payout.
fn settle_derivative(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Load state to access the oracle address
    let state = STATE.load(deps.storage)?;
    
    // Load the derivative from storage
    let derivative = DERIVATIVES.load(deps.storage, id)?;

    // Query the price from the oracle
    let price_feed: Uint128 = deps
        .querier
        .query_wasm_smart(state.oracle_address.clone(), &QueryMsg::GetPrice { asset: derivative.underlying_asset.clone() })?;

    // Calculate the payout amount based on the queried price
    let payout = derivative.amount.multiply_ratio(price_feed, Uint128::new(1));

    // Create a transfer message to send the payout to the derivative creator
    let transfer_msg = BankMsg::Send {
        to_address: derivative.creator.to_string(),
        amount: vec![Coin {
            denom: state.token_address.to_string(),
            amount: payout,
        }],
    };

    // Remove the derivative from storage as it is settled
    DERIVATIVES.remove(deps.storage, id);

    Ok(Response::new()
        .add_attribute("method", "settle_derivative")
        .add_attribute("settlement_price", price_feed.to_string())
        .add_message(CosmosMsg::Bank(transfer_msg)))
}

/// Handles the different query messages to get derivative details and price.
#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDerivative { id } => to_binary(&query_derivative(deps, id)?),
        QueryMsg::GetAllDerivatives {} => to_binary(&query_all_derivatives(deps)?),
        QueryMsg::GetPrice { asset } => to_binary(&query_price(deps, asset)?),
    }
}

/// Queries the details of a specific derivative by ID.
fn query_derivative(deps: Deps<CoreumQueries>, id: u64) -> StdResult<Derivative> {
    // Load the specified derivative from storage
    let derivative = DERIVATIVES.load(deps.storage, id)?;
    Ok(derivative)
}

/// Queries the details of all derivatives.
fn query_all_derivatives(deps: Deps<CoreumQueries>) -> StdResult<Vec<Derivative>> {
    // Load all derivatives from storage and collect them into a vector
    let derivatives: Vec<Derivative> = DERIVATIVES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, derivative)) => Some(derivative),
            Err(_) => None,
        })
        .collect();
    Ok(derivatives)
}

/// Queries the price of a specific asset from the oracle.
fn query_price(deps: Deps<CoreumQueries>, asset: String) -> StdResult<Uint128> {
    // Load state to access the oracle address
    let state = STATE.load(deps.storage)?;
    
    // Query the price of the specified asset from the oracle
    let price: Uint128 = deps
        .querier
        .query_wasm_smart(state.oracle_address, &QueryMsg::GetPrice { asset })?;
    Ok(price)
}
