use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No funds were send when they are required")]
    NoFundsSend {},

    #[error("You do not have any funds in this contract")]
    NoFundsInContract {},

    #[error("Coin is not found.")]
    CoinNotFound {},

    #[error("Not enough funds. You only have {amount:?} {denom:?} in the contract")]
    NotEnoughFunds { denom: String, amount: String },
    #[error("No vault found with that id")]
    VaultDoesntExist {},
}
