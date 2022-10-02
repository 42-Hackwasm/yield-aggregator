use std::vec;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::Funds;

#[cw_serde]
pub struct InstantiateMsg {
    pub contract_admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddFunds {},
    WithdrawFunds {
        denom: String,
        amount: Uint128,
    },

    TransferFunds {
        recipient_contract_address: String,
        channel_id: String,
        denom: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(Funds)]
    GetFunds { address: String },
}

#[cw_serde]
pub enum MigrateMsg {}

// RESPONSES (maybe move to their own file?)
#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub version: String,
    pub name: String,
}
