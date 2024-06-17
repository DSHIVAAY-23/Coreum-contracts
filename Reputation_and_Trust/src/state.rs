use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub denom: String,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserReputation {
    pub reputation: u64,
}

pub const REPUTATIONS: Map<&Addr, UserReputation> = Map::new("reputations");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balances");
