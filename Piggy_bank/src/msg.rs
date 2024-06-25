use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{Deposit, State};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DepositTokens { amount: Uint128 },
    WithdrawTokens { deposit_id: u64 },
    SetLockTime { lock_time: u64 },
    SetNewOwner { new_owner: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetTokensIn {},
    #[returns(Uint128)]
    GetTokensOut {},
    #[returns(Uint128)]
    GetBalance {},
    #[returns(State)]
    GetState {},
    #[returns(Deposit)]
    GetDeposit { deposit_id: u64 },
}
