use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use cosmwasm_std::Uint128;
use cosmwasm_std::Timestamp;

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub subscription_cost: Uint128,
    pub subscription_period: u64, // in seconds
}

#[cw_serde]
pub struct Subscription {
    pub user: Addr,
    pub end_time: Timestamp,
}

pub const STATE: Item<State> = Item::new("state");
pub const SUBSCRIPTIONS: Map<Addr, Subscription> = Map::new("subscriptions");
