use cosmwasm_std::StdError;
use cw_denom::DenomError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Coin is not found.")]
    CoinNotFound {},

    #[error("Not enough funds. You only have {amount:?} {denom:?} in the contract")]
    NotEnoughFunds { denom: String, amount: String },

    #[error("No vault found with that id")]
    VaultDoesntExist {},

    #[error("{0}")]
    Denom(#[from] DenomError),

    #[error("{0}")]
    Payment(#[from] cw_utils::PaymentError),
}
