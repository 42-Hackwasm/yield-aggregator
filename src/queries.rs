

use cosmwasm_std::{Deps, StdResult};

// use crate::msg::{SomeMsg};
use crate::msg::{ConfigResponse};
// use cosmwasm_std::{Deps, Order, StdResult, Uint128};

use crate::state::{CONFIG};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        version: config.version,
        name: config.name
    })
}