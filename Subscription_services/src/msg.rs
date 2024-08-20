use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub subscription_cost: Uint128,
    pub subscription_period: u64, // in seconds
}

#[cw_serde]
pub enum ExecuteMsg {
    Subscribe { denom: String},
    Renew { denom: String},
    WithdrawFunds { },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsSubscribed { address: String },
}
