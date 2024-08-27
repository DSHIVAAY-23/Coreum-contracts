use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Content, Subscription, COMMUNITY_FUND, CONTENT, SUBSCRIPTIONS};
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "content-sharing";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) ->  Result<Response<CoreumMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    COMMUNITY_FUND.save(deps.storage, &msg.initial_fund)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("initial_fund", msg.initial_fund))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) ->  Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::CreateContent { uri, access_level } => {
            create_content(deps, info, uri, access_level)
        }
        ExecuteMsg::TipContent { content_id, amount } => {
            tip_content(deps, info, content_id, amount)
        }
        ExecuteMsg::Subscribe {
            creator,
            amount,
            duration,
        } => subscribe(deps, env, info, creator, amount, duration),
        ExecuteMsg::AccessContent { content_id } => access_content(deps, info, content_id),
        ExecuteMsg::DistributeFunds {} => distribute_funds(deps, env),
    }
}

fn create_content(
    deps: DepsMut,
    info: MessageInfo,
    uri: String,
    access_level: Uint128,
) ->  Result<Response<CoreumMsg>, ContractError>{
    let content = Content {
        creator: info.sender.clone(),
        uri,
        access_level,
        tips: Uint128::zero(),
    };
    CONTENT.save(deps.storage, &info.sender.to_string(), &content)?;
    Ok(Response::new()
        .add_attribute("method", "create_content")
        .add_attribute("creator", info.sender))
}

fn tip_content(
    deps: DepsMut,
    info: MessageInfo,
    content_id: String,
    amount: Uint128,
) ->  Result<Response<CoreumMsg>, ContractError> {
    let mut content = CONTENT.load(deps.storage, &content_id)?;
    content.tips += amount;
    CONTENT.save(deps.storage, &content_id, &content)?;
    Ok(Response::new()
        .add_attribute("method", "tip_content")
        .add_attribute("content_id", content_id))
}

fn subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    creator: Addr,
    amount: Uint128,
    duration: u64,
) ->  Result<Response<CoreumMsg>, ContractError> {
    let expiry = env.block.time.seconds() + duration;
    let subscription = Subscription {
        subscriber: info.sender.clone(),
        creator: creator.clone(),
        amount,
        expiry,
    };
    SUBSCRIPTIONS.save(
        deps.storage,
        (info.sender.clone(), creator.clone()),
        &subscription,
    )?;
    Ok(Response::new()
        .add_attribute("method", "subscribe")
        .add_attribute("subscriber", info.sender)
        .add_attribute("creator", creator))
}

fn access_content(
    deps: DepsMut,
    info: MessageInfo,
    content_id: String,
) ->  Result<Response<CoreumMsg>, ContractError> {
    let content = CONTENT.load(deps.storage, &content_id)?;
    if content.access_level <= Uint128::from(info.funds[0].amount) {
        Ok(Response::new()
            .add_attribute("method", "access_content")
            .add_attribute("content_id", content_id))
    } else {
        Err(ContractError::Unauthorized {  }
        
        )
    }
}

fn distribute_funds(deps: DepsMut, env: Env) -> Result<Response<CoreumMsg>, ContractError> {
    let community_fund = COMMUNITY_FUND.load(deps.storage)?;
    let mut total_subscriptions = Uint128::zero();

    // First loop to calculate total subscriptions
    let subscriptions: Vec<Subscription> = SUBSCRIPTIONS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(_, subscription)| subscription)
        .collect();

    for subscription in subscriptions.iter() {
        if subscription.expiry > env.block.time.seconds() {
            total_subscriptions += subscription.amount;
        }
    }

    let mut response = Response::new().add_attribute("method", "distribute_funds");
    let mut remaining_fund = community_fund;

    for mut subscription in subscriptions {
        if subscription.expiry > env.block.time.seconds() {
            let share = community_fund * subscription.amount / total_subscriptions;
            remaining_fund -= share;
            subscription.amount += share;
            SUBSCRIPTIONS.save(
                deps.storage,
                (
                    subscription.subscriber.clone(),
                    subscription.creator.clone(),
                ),
                &subscription,
            )?;
            response = response.add_message(cosmwasm_std::BankMsg::Send {
                to_address: subscription.creator.to_string(),
                amount: vec![cosmwasm_std::Coin {
                    denom: "coreum".to_string(),
                    amount: share,
                }],
            });
        }
    }

    COMMUNITY_FUND.save(deps.storage, &remaining_fund)?;
    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContent { content_id } => to_binary(&query_content(deps, content_id)?),
        QueryMsg::GetSubscription {
            subscriber,
            creator,
        } => to_binary(&query_subscription(deps, subscriber, creator)?),
        QueryMsg::GetCommunityFund {} => to_binary(&query_community_fund(deps)?),
    }
}

fn query_content(deps: Deps<CoreumQueries>, content_id: String) -> StdResult<Content> {
    let content = CONTENT.load(deps.storage, &content_id)?;
    Ok(content)
}

fn query_subscription(
    deps: Deps<CoreumQueries>,
    subscriber: Addr,
    creator: Addr,
) -> StdResult<Subscription> {
    let subscription = SUBSCRIPTIONS.load(deps.storage, (subscriber, creator))?;
    Ok(subscription)
}

fn query_community_fund(deps: Deps<CoreumQueries>) -> StdResult<Uint128> {
    let fund = COMMUNITY_FUND.load(deps.storage)?;
    Ok(fund)
}
