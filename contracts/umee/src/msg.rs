use std::vec;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Collaterize {},
    Borrow {
        denom: cw_denom::UncheckedDenom,
        amount: Uint128,
    },
    // Repay { denom: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
    #[returns(Vec<cw_denom::Asset>)]
    Collateral { account: String },
    #[returns(Vec<cw_denom::Asset>)]
    Borrowed { account: String },
}

#[cw_serde]
pub enum MigrateMsg {}

///// callback messages

#[cw_serde]
pub enum ReceiveMsg {
    /// Only valid cw20 message is to collaterize the tokens
    Collaterize {},
}
