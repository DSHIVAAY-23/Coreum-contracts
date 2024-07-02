use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub creator: Addr,
    pub uri: String,
    pub access_level: Uint128,
    pub tips: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Subscription {
    pub subscriber: Addr,
    pub creator: Addr,
    pub amount: Uint128,
    pub expiry: u64,
}

pub const CONTENT: Map<&str, Content> = Map::new("content");
pub const SUBSCRIPTIONS: Map<(Addr, Addr), Subscription> = Map::new("subscriptions");
pub const COMMUNITY_FUND: Item<Uint128> = Item::new("community_fund");
