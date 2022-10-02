use std::ops::Mul;

use crate::denom_utils::denom_to_string;
#[cfg(not(feature = "library"))]
// COSMWASM
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cosmwasm_std::{Decimal, Uint128};

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

use crate::wasmswap_msg::InfoResponse;
use crate::wasmswap_msg::QueryMsg as WasmSwapQueryMsg;
use crate::wasmswap_msg::Token1ForToken2PriceResponse;
use crate::wasmswap_msg::Token2ForToken1PriceResponse;

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
        ExecuteMsg::Deposit {
            pool_addr,
            token1_amount,
            token2_amount,
        } => deposit(deps, env, info, pool_addr, token1_amount, token2_amount),
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

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_addr: Addr,
    token1_amount: Uint128,
    token2_amount: Uint128,
) -> Result<Response, ContractError> {
    //todo: check funds are 2 tokens
    //todo: support for cw20
    let pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(pool_addr.clone(), &WasmSwapQueryMsg::Info {})?;

    let funds = info.funds.clone();
    let token1_denom = &denom_to_string(&pool_info.token1_denom);
    let token2_denom = &denom_to_string(&pool_info.token2_denom);
    /*
        let token1_amount = funds
            .iter()
            .find(|coin| coin.denom.eq(token1_denom))
            .unwrap()
            .amount;
        let token2_amount = funds
            .iter()
            .find(|coin| coin.denom.eq(token2_denom))
            .unwrap()
            .amount;
    */
    //todo: get price
    let token1_to_token2_price_response: Token1ForToken2PriceResponse =
        deps.querier.query_wasm_smart(
            pool_addr.clone(),
            &WasmSwapQueryMsg::Token1ForToken2Price { token1_amount },
        )?;
    let token2_to_token1_price_response: Token2ForToken1PriceResponse =
        deps.querier.query_wasm_smart(
            pool_addr.clone(),
            &WasmSwapQueryMsg::Token2ForToken1Price { token2_amount },
        )?;

    let token1_price =
        Decimal::from_ratio(token1_to_token2_price_response.token2_amount, token1_amount);
    let token2_price =
        Decimal::from_ratio(token2_to_token1_price_response.token1_amount, token2_amount);

    let token1_value: Uint128 = token1_amount.mul(token1_price.clone());
    let token2_value = token2_amount.mul(token2_price.clone());

    let total_token1_value = token1_amount + token2_value;
    let total_token2_value = token2_amount + token1_value;

    let mut token1_missing_amount = Uint128::zero();
    let mut token2_missing_amount = Uint128::zero();
    let mut token1_amount_to_swap = Uint128::zero();
    let mut token2_amount_to_swap = Uint128::zero();

    let message = if token1_value > token2_amount {
        let token2_required_amount =
            Uint128::one().mul(Decimal::from_ratio(total_token2_value, 2u128));
        token2_missing_amount = token2_required_amount - token2_amount;
        token1_amount_to_swap = token2_missing_amount * token2_price;
        "Should convert a bit of token_1 into token_2"
    } else if token2_value > token1_amount {
        let token1_required_amount =
            Uint128::one().mul(Decimal::from_ratio(total_token1_value, 2u128));
        token1_missing_amount = token1_required_amount - token1_amount;
        token2_amount_to_swap = token1_missing_amount * token1_price;
        "Should convert a bit of token_2 into token_1"
    } else {
        "Ready to LP"
    };

    let attrs = vec![
        (
            "token1_to_token2_amount",
            token1_to_token2_price_response.token2_amount.to_string(),
        ),
        (
            "token2_to_token1_amount",
            token2_to_token1_price_response.token1_amount.to_string(),
        ),
        ("token1_price", token1_price.to_string()),
        ("token2_price", token2_price.to_string()),
        ("token1_value", token1_value.to_string()),
        ("token2_value", token2_value.to_string()),
        ("token1_total_value", total_token1_value.to_string()),
        ("token2_total_value", total_token2_value.to_string()),
        ("token1_missing_amount", token1_missing_amount.to_string()),
        ("token2_missing_amount", token2_missing_amount.to_string()),
        ("token1_amount_to_swap", token1_amount_to_swap.to_string()),
        ("token2_amount_to_swap", token2_amount_to_swap.to_string()),
        ("message", message.to_string()),
    ];
    let res = Response::new().add_attributes(attrs);
    Ok(res)
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
