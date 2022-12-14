use std::vec;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{Funds, UserPosition, Vault};

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
    CreateVault {
        vault: Vault,
    },
    DisableVault {
        vault_id: u128,
    },
    // Borrow {
    //     denom: cw_denom::UncheckedDenom,
    // },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(Funds)]
    GetFunds { address: String },
    #[returns(VaultResponse)]
    GetVaults {},
    #[returns(PositionsResponse)]
    GetUserPositions { address: String },
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

#[cw_serde]
pub struct VaultResponse {
    pub vaults: Vec<(u128, Vault)>,
}

#[cw_serde]
pub struct PositionsResponse {
    pub positions: Vec<(u128, UserPosition)>,
}
