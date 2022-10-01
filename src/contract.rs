#[cfg(not(feature = "library"))]
// COSMWASM
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

// ERRORS & MESSAGES
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// CUSTOM CRATES
use crate::queries;
// execute

// STATE
use crate::state::{Config, Funds, CONFIG, FUNDS};

// CW2
use cw2::set_contract_version;
const CONTRACT_NAME: &str = "crates.io:yield-aggregator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// LOGIC
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // deps.api.addr_validate(&msg.contract_admin)?;

    // admin if set, if not, the sender = contract admin
    let admin = deps
        .api
        .addr_validate(&msg.contract_admin.unwrap_or(info.sender.to_string()))?;

    let config = Config {
        admin: admin.clone(),
        version: CONTRACT_VERSION.to_string(),
        name: CONTRACT_NAME.to_string(),
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
        .add_attribute("version", CONTRACT_VERSION)
        .add_attribute("name", CONTRACT_NAME))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddFunds {} => add_funds(deps, env, info),
    }
}

pub fn add_funds(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let address = info.sender.to_string();
    // let users_funds = FUNDS.may_load(deps.storage, address.clone())?;

    // check if info.funds.clone() is empty
    if info.funds.clone().is_empty() {
        return Err(ContractError::NoFundsSend {});
    }

    // // if users_funds is empty, save blank funds to storage
    // if users_funds.is_none() {
    //     let funds = Funds { funds: vec![] };
    //     FUNDS.save(deps.storage, address.clone(), &funds)?;
    // }

    // let mut funds = users_funds.unwrap();
    // funds.funds.append(&mut info.funds.clone());
    // FUNDS.save(deps.storage, address.clone(), &funds)?;

    // load FUNDS from stroage, if none, create new Funds & save. Or else just append funds to existing Funds
    let mut funds = FUNDS
        .may_load(deps.storage, address.clone())?
        .unwrap_or_else(|| Funds { funds: vec![] });
    funds.funds.append(&mut info.funds.clone());
    FUNDS.save(deps.storage, address.clone(), &funds)?;

    // convert info.funds into a string
    let funds = info
        .funds
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let new_funds = FUNDS.load(deps.storage, address.clone())?;
    let new_funds = new_funds
        .funds
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    Ok(Response::new()
        .add_attribute("action", "add_funds")
        .add_attribute("added_funds", funds)
        .add_attribute("new_funds", new_funds))
}

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let pool_addr = Addr::unchecked("fake pool addr");
    let funds = info.funds.clone();

    todo!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::GetFunds { address } => to_binary(&queries::get_funds(deps, address)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // https://docs.cosmwasm.com/docs/1.0/smart-contracts/migration/
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= (*CONTRACT_VERSION).to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }
    // set the new version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // update some data...

    Ok(Response::default()
        .add_attribute("action", "migration")
        .add_attribute("version", CONTRACT_VERSION)
        .add_attribute("contract", CONTRACT_NAME))
}
