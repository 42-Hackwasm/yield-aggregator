use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_denom::Asset;

use crate::state::{Config, BORROWS, COLLATERAL, CONFIG};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_collateral(deps: Deps, user: &Addr) -> StdResult<Vec<Asset>> {
    COLLATERAL.load(deps.storage, user)
}

pub fn query_borrowed(deps: Deps, user: &Addr) -> StdResult<Vec<Asset>> {
    BORROWS.load(deps.storage, user)
}

#[cw_serde]
pub struct Assets {
    pub assets: Vec<Asset>,
}
