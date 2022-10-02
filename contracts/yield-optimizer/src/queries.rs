use cosmwasm_std::{Deps, StdResult};

// use crate::msg::{SomeMsg};
use crate::msg::ConfigResponse;
// use cosmwasm_std::{Deps, Order, StdResult, Uint128};

use crate::state::{Funds, CONFIG, FUNDS};

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

    Ok(funds.unwrap_or_else(|| Funds { funds: vec![] }))
}
