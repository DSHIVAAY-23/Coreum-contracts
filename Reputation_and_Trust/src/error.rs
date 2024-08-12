use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidInitialAmount")]
    InvalidInitialAmount {},
    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Overflow error")]

    Overflow {},

    #[error("Invalid Feature Flag")]
    InvalidFeatureFlag { },
    // other variants...
}
