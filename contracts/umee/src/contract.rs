#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_denom::Asset;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use cw2::set_contract_version;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    const CONTRACT_NAME: &str = "crates.io:yield-aggregator";
    const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {};
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("version", CONTRACT_VERSION)
        .add_attribute("name", CONTRACT_NAME))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use crate::execute;
    match msg {
        ExecuteMsg::Collaterize {} => {
            let coin = cw_utils::one_coin(&info)?;
            execute::collaterize(deps, &info.sender, Asset::from_native_checked(coin))
        }
        ExecuteMsg::Borrow { denom, amount } => {
            let denom = denom.into_checked(deps.as_ref())?;
            execute::borrow(deps, &info.sender, denom, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use crate::queries::*;
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Borrowed { account } => {
            let a = deps.api.addr_validate(&account)?;
            to_binary(&query_borrowed(deps, &a)?)
        }
        QueryMsg::Collateral { account } => {
            let a = deps.api.addr_validate(&account)?;
            to_binary(&query_collateral(deps, &a)?)
        }
    }
}
