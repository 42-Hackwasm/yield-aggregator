use cosmwasm_std::{Deps, StdResult};
use cosmwasm_std::Order::Ascending;

// use crate::msg::{SomeMsg};
use crate::msg::{ConfigResponse, VaultResponse};
// use cosmwasm_std::{Deps, Order, StdResult, Uint128};

use crate::state::{CONFIG, FUNDS, Funds, Vault, VAULTS};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        version: config.version,
        name: config.name,
    })
}

// get Funds a given user has sent to the contract
pub fn get_funds(deps: Deps, address: String) -> StdResult<Funds> {
    let funds = FUNDS.may_load(deps.storage, address)?;

    Ok(funds.unwrap_or_else(|| Funds {
        funds: vec![]
    }))
}

pub fn get_vaults(deps: Deps) -> StdResult<VaultResponse> {
    let res:StdResult<Vec<_>> = VAULTS.range(deps.storage, None, None, Ascending).collect();
    let vaults = res?;
    Ok(VaultResponse {
        vaults
    })
}
