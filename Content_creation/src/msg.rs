use cosmwasm_std::{Addr, Uint128};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateContent {
        uri: String,
        access_level: Uint128,
    },
    TipContent {
        content_id: String,
        amount: Uint128,
    },
    Subscribe {
        creator: Addr,
        amount: Uint128,
        duration: u64,
    },
    AccessContent {
        content_id: String,
    },
    DistributeFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetContent { content_id: String },
    GetSubscription { subscriber: Addr, creator: Addr },
    GetCommunityFund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InstantiateMsg {
    pub initial_fund: Uint128,
}
