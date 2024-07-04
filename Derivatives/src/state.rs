use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub token_address: Addr,
    pub oracle_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Derivative {
    pub id: u64,
    pub creator: Addr,
    pub derivative_type: String,
    pub underlying_asset: String,
    pub amount: Uint128,
    pub price: Uint128,
    pub expiry: Option<u64>, // Optional expiry for perpetuals
}

pub const STATE: Item<State> = Item::new("state");
pub const DERIVATIVES: Map<u64, Derivative> = Map::new("derivatives");
