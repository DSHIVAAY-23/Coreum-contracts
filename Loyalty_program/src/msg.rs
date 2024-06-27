use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub token_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    EarnTokens { customer: String, amount: Uint128 },
    RedeemTokens { customer: String, amount: Uint128 },
    TransferTokens { from: String, to: String, amount: Uint128 },
    WithdrawTokens { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetBalance { customer: String },
}
