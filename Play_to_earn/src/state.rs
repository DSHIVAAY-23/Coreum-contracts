use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub owner: Addr,
    pub token_address: Addr,
    pub treasury_address: Addr,
    pub breeding_contract: Addr,
     pub asset_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub address: Addr,
    pub rewards: u128,
}

pub const STATE: Item<State> = Item::new("state");
pub const PLAYERS: Map<&Addr, Player> = Map::new("players");
pub const GUILDS: Map<&Addr, Vec<Addr>> = Map::new("guilds");
