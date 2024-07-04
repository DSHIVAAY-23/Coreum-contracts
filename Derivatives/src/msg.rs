use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::Derivative;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub token_address: String,
    pub oracle_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateDerivative {
        derivative_type: String,
        underlying_asset: String,
        amount: Uint128,
        price: Uint128,
        expiry: Option<u64>,
    },
    TradeDerivative {
        id: u64,
        buyer: String,
        amount: Uint128,
    },
    SettleDerivative {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Derivative)]
    GetDerivative { id: u64 },
    #[returns(Vec<Derivative>)]
    GetAllDerivatives {},
    #[returns(Uint128)]
    GetPrice { asset: String },
}
