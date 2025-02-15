use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unlock_time_not_reached!")]
    Unlock {},

    #[error("You are not the depositor!")]
    InvalidOwner {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
