use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can withdraw funds.")]
    Unauthorized {},

    #[error("Overflow")]
    Overflow{},


    #[error("Insufficient Funds")]
    InsufficientFunds {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },


}