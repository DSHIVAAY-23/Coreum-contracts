use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Uint128};


#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub lending_pool: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RequestFlashLoan { token: String, amount: Uint128, collateral: Uint128 },
    ExecuteOperation { token: String, amount: Uint128, premium: Uint128 },
    Withdraw { token: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetBalance { token: String },
}

#[cw_serde]
pub struct RequestFlashLoan {
    pub recipient: String,
    pub token: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RepayFlashLoan {
    pub sender: String,
    pub token: String,
    pub amount: Uint128,
}
#[cw_serde]

pub enum CustomMsg {
    RequestFlashLoan(RequestFlashLoan),
    RepayFlashLoan(RepayFlashLoan),
}

impl From<CustomMsg> for CosmosMsg<CustomMsg> {
    fn from(msg: CustomMsg) -> Self {
        CosmosMsg::Custom(msg)
    }
}