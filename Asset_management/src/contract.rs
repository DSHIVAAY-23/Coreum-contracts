use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, AssetType as MsgAssetType};
use crate::state::{TokenizedAsset, ASSETS, FRACTIONAL_BALANCES, NEXT_TOKEN_ID, AssetType as StateAssetType};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, Uint128, WasmMsg
};
use cw2::set_contract_version;
use crate::smarttoken::{BALANCES, TOKEN_INFO};
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};

const CONTRACT_NAME: &str = "tokenized-asset-management";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the contract
#[entry_point]
pub fn instantiate(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError>  {
    // Validate the owner's address
    let owner = deps.api.addr_validate(&msg.owner)?;
    // Initialize the token ID counter
    NEXT_TOKEN_ID.save(deps.storage, &1)?;
    // Set the contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Return a response
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner.to_string()))
}

/// Handle execute messages
#[entry_point]
pub fn execute(
    deps: DepsMut<CoreumQueries>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateAsset { total_supply, price, uri, asset_type } => create_asset(deps, info, total_supply, price, uri, asset_type),
        ExecuteMsg::TransferOwnership { token_id, to, amount } => transfer_ownership(deps, info, token_id, to, amount),
        ExecuteMsg::PayoutDividends { token_id } => payout_dividends(deps, info, token_id),
        ExecuteMsg::MintSmartToken { to, amount } => execute_mint_smart_token(deps, info, to, amount),
        ExecuteMsg::TransferSmartToken { to, amount } => execute_transfer_smart_token(deps, info, to, amount),
    }
}

/// Create a new asset
fn create_asset(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    total_supply: Uint128,
    price: Uint128,
    uri: String,
    asset_type: MsgAssetType,
) -> Result<Response<CoreumMsg>, ContractError> {
    let owner = info.sender.clone();
    let token_id = NEXT_TOKEN_ID.load(deps.storage)?;

    // Map the message asset type to the state asset type
    let asset_type = match asset_type {
        MsgAssetType::RealWorldAsset => StateAssetType::RealWorldAsset,
        MsgAssetType::IntellectualProperty => StateAssetType::IntellectualProperty,
        MsgAssetType::BondOrSecurity => StateAssetType::BondOrSecurity,
    };

    // Create a new tokenized asset
    let asset = TokenizedAsset {
        owner: owner.clone(),
        total_supply,
        remaining_supply: total_supply,
        price,
        uri,
        asset_type,
    };

    // Save the asset and increment the token ID counter
    ASSETS.save(deps.storage, token_id, &asset)?;
    NEXT_TOKEN_ID.save(deps.storage, &(token_id + 1))?;

    Ok(Response::new()
        .add_attribute("method", "create_asset")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("owner", owner.to_string()))
}

/// Transfer ownership of a fractional asset
fn transfer_ownership(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    token_id: u64,
    to: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let mut asset = ASSETS.load(deps.storage, token_id)?;

    // Ensure the sender is the owner of the asset
    if info.sender != asset.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure there is enough remaining supply to transfer
    if amount > asset.remaining_supply {
        return Err(ContractError::Std(StdError::generic_err("Invalid amount to transfer")));
    }

    // Update the remaining supply
    asset.remaining_supply = asset.remaining_supply.checked_sub(amount)
        .map_err(|e| ContractError::Std(StdError::generic_err(format!("Overflow error: {}", e))))?;
    ASSETS.save(deps.storage, token_id, &asset)?;

    // Update the recipient's balance
    let to_addr = deps.api.addr_validate(&to)?;
    let balance = FRACTIONAL_BALANCES.may_load(deps.storage, (to_addr.clone(), token_id))?.unwrap_or_default();
    FRACTIONAL_BALANCES.save(deps.storage, (to_addr.clone(), token_id), &(balance + amount))?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("from", info.sender.to_string())
        .add_attribute("to", to_addr.to_string())
        .add_attribute("amount", amount.to_string()))
}

/// Payout dividends to fractional owners
fn payout_dividends(
    deps: DepsMut<CoreumQueries>,
    _info: MessageInfo,
    token_id: u64,
) -> Result<Response<CoreumMsg>, ContractError> {
    let asset = ASSETS.load(deps.storage, token_id)?;

    // Ensure the asset is a bond or security
    if asset.asset_type != StateAssetType::BondOrSecurity {
        return Err(ContractError::Unauthorized {});
    }

    // Calculate the total dividends
    let total_dividends = asset.total_supply.checked_sub(asset.remaining_supply)
        .map_err(|e| ContractError::Std(StdError::generic_err(format!("Overflow error: {}", e))))?;
    if total_dividends.is_zero() {
        return Err(ContractError::Unauthorized {});
    }

    // Distribute dividends to fractional owners
    let mut messages = vec![];
    let balances: StdResult<Vec<_>> = FRACTIONAL_BALANCES
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let balances = balances?;

    for ((owner_raw, balance_token_id), balance) in balances {
        if balance_token_id == token_id {
            let owner = deps.api.addr_humanize(&CanonicalAddr::from(owner_raw.as_bytes()))?;
            let dividend = total_dividends.multiply_ratio(balance, asset.total_supply);
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: owner.to_string(),
                amount: vec![Coin { denom: "uasset".to_string(), amount: dividend }],
            }));
        }
    }

    Ok(Response::new()
        .add_attribute("method", "payout_dividends")
        .add_attribute("token_id", token_id.to_string())
        .add_messages(messages))
}

/// Mint new smart tokens
fn execute_mint_smart_token(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    to: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let token_info = TOKEN_INFO.load(deps.storage)?;

    // Ensure the sender is the owner of the token
    if info.sender != token_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Update the recipient's balance
    let to_addr = deps.api.addr_validate(&to)?;
    let balance = BALANCES.may_load(deps.storage, to_addr.clone())?.unwrap_or_default();
    BALANCES.save(deps.storage, to_addr.clone(), &(balance + amount))?;

    Ok(Response::new()
        .add_attribute("method", "mint_smart_token")
        .add_attribute("to", to_addr.to_string())
        .add_attribute("amount", amount.to_string()))
}

/// Transfer smart tokens
fn execute_transfer_smart_token(
    deps: DepsMut<CoreumQueries>,
    info: MessageInfo,
    to: String,
    amount: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let sender_addr = info.sender.clone();
    let to_addr = deps.api.addr_validate(&to)?;

    // Ensure the sender has enough balance
    let sender_balance = BALANCES.load(deps.storage, sender_addr.clone())?;
    if sender_balance < amount {
        return Err(ContractError::Unauthorized {});
    }

    // Update the sender's and recipient's balances
    BALANCES.save(deps.storage, sender_addr.clone(), &(sender_balance - amount))?;
    let recipient_balance = BALANCES.may_load(deps.storage, to_addr.clone())?.unwrap_or_default();
    BALANCES.save(deps.storage, to_addr.clone(), &(recipient_balance + amount))?;

    Ok(Response::new()
        .add_attribute("method", "transfer_smart_token")
        .add_attribute("from", sender_addr.to_string())
        .add_attribute("to", to_addr.to_string())
        .add_attribute("amount", amount.to_string()))
}

/// Handle query messages
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::FractionalOwnership { token_id, owner } => to_binary(&query_fractional_ownership(deps, token_id, owner)?),
        QueryMsg::TokenURI { token_id } => to_binary(&query_token_uri(deps, token_id)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),

    }
}

fn query_fractional_ownership(deps: Deps, token_id: u64, owner: String) -> StdResult<Uint128> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let balance = FRACTIONAL_BALANCES.may_load(deps.storage, (owner_addr, token_id))?.unwrap_or_default();
    Ok(balance)
}

fn query_token_uri(deps: Deps, token_id: u64) -> StdResult<String> {
    let asset = ASSETS.load(deps.storage, token_id)?;
    Ok(asset.uri)
}


fn query_balance(
    deps: Deps,
    address: String,
) -> StdResult<Uint128> {
    let addr = deps.api.addr_validate(&address)?;
    let balance = BALANCES.may_load(deps.storage, addr)?.unwrap_or_default();
    Ok(balance)
}
