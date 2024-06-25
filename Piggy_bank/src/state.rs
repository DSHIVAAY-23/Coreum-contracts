use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub lock_time: u64,
    pub tokens_in: Uint128,
    pub tokens_out: Uint128,
    pub deposit_count: u64,
}

#[cw_serde]
pub struct Deposit {
    pub deposit_id: u64,
    pub amount: Uint128,
    pub from: Addr,
    pub deposit_time: u64,
    pub unlock_time: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const DEPOSITS: Map<u64, Deposit> = Map::new("deposits");
