use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LiquidityPool {
    pub token1_address: Addr,
    pub token2_address: Addr,
    pub token1_reserve: Uint128,
    pub token2_reserve: Uint128,
    pub total_liquidity: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub pools: Vec<Addr>,
    pub reward_token: Addr,
    pub reward_rate: u128,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const LIQUIDITY_POOLS: Map<(Addr, Addr), LiquidityPool> = Map::new("liquidity_pools");
pub const STATE: Item<State> = Item::new("state");
pub const LIQUIDITY_PROVIDERS: Map<(Addr, Addr), Uint128> = Map::new("liquidity_providers");
pub const REWARDS: Map<Addr, Uint128> = Map::new("rewards");
