use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{LiquidityPool, PoolType, OWNER, LIQUIDITY_POOLS, STATE, LIQUIDITY_PROVIDERS, REWARDS, State};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, BankMsg,
};
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "dex";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the contract with the owner, reward token, and reward rate.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Validate owner and reward token addresses
    let owner = deps.api.addr_validate(&msg.owner)?;
    let reward_token = deps.api.addr_validate(&msg.reward_token)?;

    // Initialize state with owner, reward token, and reward rate
    let state = State {
        owner: owner.clone(),
        pools: vec![],
        reward_token,
        reward_rate: msg.reward_rate,
    };
    STATE.save(deps.storage, &state)?;
    OWNER.save(deps.storage, &owner)?;

    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.to_string()))
}

/// Handles the different execution messages for the DEX.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        // Handle adding liquidity to a pool
        ExecuteMsg::AddLiquidity { token1_address, token2_address, amount1, amount2, pool_type } => {
            add_liquidity(deps, env, info, token1_address, token2_address, amount1, amount2, pool_type)
        },
        // Handle removing liquidity from a pool
        ExecuteMsg::RemoveLiquidity { token1_address, token2_address, amount1, amount2 } => {
            remove_liquidity(deps, env, info, token1_address, token2_address, amount1, amount2)
        },
        // Handle swapping tokens in a pool
        ExecuteMsg::SwapTokens { token_in, token_out, amount_in } => {
            swap_tokens(deps, env, info, token_in, token_out, amount_in)
        },
        // Handle distributing rewards to liquidity providers
        ExecuteMsg::DistributeRewards {} => distribute_rewards(deps, env, info),
    }
}



/// Adds liquidity to the specified pool.
fn add_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token1_address: String,
    token2_address: String,
    amount1: Uint128,
    amount2: Uint128,
    pool_type: PoolType,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Validate token addresses
    let token1 = deps.api.addr_validate(&token1_address)?;
    let token2 = deps.api.addr_validate(&token2_address)?;
    let pool_key = (token1.clone(), token2.clone());

    // Load or create a new liquidity pool
    let mut pool = LIQUIDITY_POOLS.may_load(deps.storage, pool_key.clone())?.unwrap_or(LiquidityPool {
        token1_address: token1.clone(),
        token2_address: token2.clone(),
        token1_reserve: Uint128::zero(),
        token2_reserve: Uint128::zero(),
        total_liquidity: Uint128::zero(),
        pool_type:pool_type.clone(),
    });

    // Update pool reserves and total liquidity
    pool.token1_reserve += amount1;
    pool.token2_reserve += amount2;
    pool.total_liquidity += amount1 + amount2;

    LIQUIDITY_POOLS.save(deps.storage, pool_key, &pool)?;

    // Update user's liquidity
    let user_key = (info.sender.clone(), token1.clone());
    let user_liquidity = LIQUIDITY_PROVIDERS.may_load(deps.storage, user_key.clone())?.unwrap_or(Uint128::zero());
    LIQUIDITY_PROVIDERS.save(deps.storage, user_key, &(user_liquidity + amount1 + amount2))?;

    // Update state to include new pools if necessary
    let mut state = STATE.load(deps.storage)?;
    if !state.pools.contains(&token1) {
        state.pools.push(token1.clone());
    }
    if !state.pools.contains(&token2) {
        state.pools.push(token2.clone());
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "add_liquidity")
        .add_attribute("token1", token1_address)
        .add_attribute("token2", token2_address)
        .add_attribute("amount1", amount1.to_string())
        .add_attribute("amount2", amount2.to_string())
        .add_attribute("pool_type", format!("{:?}", pool_type)))
}


/// Removes liquidity from the specified pool.
fn remove_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token1_address: String,
    token2_address: String,
    amount1: Uint128,
    amount2: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Validate token addresses
    let token1 = deps.api.addr_validate(&token1_address)?;
    let token2 = deps.api.addr_validate(&token2_address)?;
    let pool_key = (token1.clone(), token2.clone());

    // Load the liquidity pool
    let mut pool = LIQUIDITY_POOLS.load(deps.storage, pool_key.clone())?;

    // Check if the pool has enough reserves to remove liquidity
    if pool.token1_reserve < amount1 || pool.token2_reserve < amount2 {
        return Err((ContractError::InsufficientFunds {}));
    }

    // Update pool reserves and total liquidity
    pool.token1_reserve -= amount1;
    pool.token2_reserve -= amount2;
    pool.total_liquidity -= amount1 + amount2;

    LIQUIDITY_POOLS.save(deps.storage, pool_key, &pool)?;

    // Update user's liquidity
    let user_key = (info.sender.clone(), token1.clone());
    let user_liquidity = LIQUIDITY_PROVIDERS.load(deps.storage, user_key.clone())?;
    if user_liquidity < amount1 + amount2 {
        return Err(ContractError::InsufficientFunds {});
    }
    LIQUIDITY_PROVIDERS.save(deps.storage, user_key, &(user_liquidity - amount1 - amount2))?;

    Ok(Response::new()
        .add_attribute("method", "remove_liquidity")
        .add_attribute("token1", token1_address)
        .add_attribute("token2", token2_address)
        .add_attribute("amount1", amount1.to_string())
        .add_attribute("amount2", amount2.to_string()))
}
/// Swaps tokens in the specified pool.
fn swap_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_in: String,
    token_out: String,
    amount_in: Uint128,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Validate token addresses
    let token_in_addr = deps.api.addr_validate(&token_in)?;
    let token_out_addr = deps.api.addr_validate(&token_out)?;
    let pool_key = (token_in_addr.clone(), token_out_addr.clone());

    // Load the liquidity pool
    let mut pool = LIQUIDITY_POOLS.load(deps.storage, pool_key.clone())?;

    // Implement the swap logic based on pool type
    let amount_out = match pool.pool_type {
        PoolType::XYK => calculate_xyk_swap(&pool, amount_in)?,
        PoolType::Stable => calculate_stable_swap(&pool, amount_in)?,
        PoolType::MetaStable => calculate_metastable_swap(&pool, amount_in)?,
    };

    // Update pool reserves
    if pool.token1_address == token_in_addr {
        pool.token1_reserve += amount_in;
        pool.token2_reserve -= amount_out;
    } else {
        pool.token2_reserve += amount_in;
        pool.token1_reserve -= amount_out;
    }

    LIQUIDITY_POOLS.save(deps.storage, pool_key, &pool)?;

    // Update user's balance
    let token_out_user_key = (info.sender.clone(), token_out_addr.clone());
    let token_out_user_balance = LIQUIDITY_PROVIDERS.may_load(deps.storage, token_out_user_key.clone())?.unwrap_or(Uint128::zero());
    LIQUIDITY_PROVIDERS.save(deps.storage, token_out_user_key, &(token_out_user_balance + amount_out))?;

    Ok(Response::new()
        .add_attribute("method", "swap_tokens")
        .add_attribute("token_in", token_in)
        .add_attribute("token_out", token_out)
        .add_attribute("amount_in", amount_in.to_string())
        .add_attribute("amount_out", amount_out.to_string()))
}
/// Calculates the output amount for XYK swap.
fn calculate_xyk_swap(pool: &LiquidityPool, amount_in: Uint128) -> Result<Uint128, ContractError> {
    let token_in_reserve = pool.token1_reserve;
    let token_out_reserve = pool.token2_reserve;
    let amount_out = amount_in * token_out_reserve / (token_in_reserve + amount_in);
    Ok(amount_out)
}

/// Calculates the output amount for Stable swap using a simplified formula.
fn calculate_stable_swap(pool: &LiquidityPool, amount_in: Uint128) -> Result<Uint128, ContractError> {
    let token_in_reserve = pool.token1_reserve;
    let token_out_reserve = pool.token2_reserve;

    // Simplified stable swap formula
    let amount_out = (amount_in * token_out_reserve) / (token_in_reserve + amount_in);
    Ok(amount_out)
}

/// Calculates the output amount for Meta-stable swap using a simplified formula.
fn calculate_metastable_swap(pool: &LiquidityPool, amount_in: Uint128) -> Result<Uint128, ContractError> {
    let token_in_reserve = pool.token1_reserve;
    let token_out_reserve = pool.token2_reserve;

    // Simplified meta-stable swap formula
    let amount_out = (amount_in * token_out_reserve) / (token_in_reserve + amount_in);
    Ok(amount_out)
}

/// Distributes rewards to liquidity providers based on their stake.
fn distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response<CoreumMsg>, ContractError> {
    // Load state to access reward token and rate
    let state = STATE.load(deps.storage)?;

    for pool_addr in state.pools.iter() {
        let pool_key = (state.reward_token.clone(), pool_addr.clone());

        if let Some(mut pool) = LIQUIDITY_POOLS.may_load(deps.storage, pool_key.clone())? {
            // Calculate reward amount based on reward rate
            let reward_amount = Uint128::from(state.reward_rate);

            // Create a mint message to mint the reward tokens
            let mint_msg = CoreumMsg::AssetFT(assetft::Msg::Mint {
                coin: Coin {
                    denom: state.reward_token.to_string(),
                    amount: reward_amount,
                },
            });

            // Update pool reserves with the minted rewards
            pool.token1_reserve += reward_amount;

            LIQUIDITY_POOLS.save(deps.storage, pool_key, &pool)?;

            return Ok(Response::new()
                .add_attribute("method", "distribute_rewards")
                .add_attribute("pool", pool_addr.to_string())
                .add_attribute("reward_amount", reward_amount.to_string())
                .add_message(CosmosMsg::Custom(mint_msg)));
        }
    }

    Ok(Response::new()
        .add_attribute("method", "distribute_rewards")
        .add_attribute("status", "no_rewards_distributed"))
}


/// Handles queries to the contract.
#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Handle querying pool reserves
        QueryMsg::GetPoolReserves { token1_address, token2_address } => {
            to_binary(&query_pool_reserves(deps, token1_address, token2_address)?)
        },
        // Handle querying user's liquidity in a pool
        QueryMsg::GetLiquidity { pool, user } => {
            to_binary(&query_liquidity(deps, pool, user)?)
        },
        // Handle querying user's rewards
        QueryMsg::GetRewards { user } => {
            to_binary(&query_rewards(deps, user)?)
        },
    }
}

/// Queries the reserves of the specified liquidity pool.
fn query_pool_reserves(deps: Deps<CoreumQueries>, token1_address: String, token2_address: String) -> StdResult<LiquidityPool> {
    // Validate token addresses and load pool reserves
    let token1 = deps.api.addr_validate(&token1_address)?;
    let token2 = deps.api.addr_validate(&token2_address)?;
    let pool_key = (token1, token2);

    LIQUIDITY_POOLS.load(deps.storage, pool_key)
}

/// Queries the liquidity of a user in the specified pool.
fn query_liquidity(deps: Deps<CoreumQueries>, pool: Addr, user: Addr) -> StdResult<Uint128> {
    // Load user's liquidity in the specified pool
    let key = (user, pool);
    let liquidity = LIQUIDITY_PROVIDERS.load(deps.storage, key)?;
    Ok(liquidity)
}

/// Queries the rewards of a user.
fn query_rewards(deps: Deps<CoreumQueries>, user: Addr) -> StdResult<Uint128> {
    // Load user's rewards
    let rewards = REWARDS.load(deps.storage, user)?;
    Ok(rewards)
}
