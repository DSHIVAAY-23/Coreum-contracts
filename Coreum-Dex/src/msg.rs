use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub reward_token: String,
    pub reward_rate: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddLiquidity {
        token1_address: String,
        token2_address: String,
        amount1: Uint128,
        amount2: Uint128,
    },
    RemoveLiquidity {
        token1_address: String,
        token2_address: String,
        amount1: Uint128,
        amount2: Uint128,
    },
    SwapTokens {
        token_in: String,
        token_out: String,
        amount_in: Uint128,
    },
    DistributeRewards {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetPoolReserves {
        token1_address: String,
        token2_address: String,
    },
    #[returns(Uint128)]
    GetLiquidity {
        pool: Addr,
        user: Addr,
    },
    #[returns(Uint128)]
    GetRewards {
        user: Addr,
    },
}
