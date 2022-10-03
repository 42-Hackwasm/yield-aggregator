use std::cmp::{max, min};
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

use crate::denom_utils::{denom_is_native, denom_to_string};
#[cfg(not(feature = "library"))]
// COSMWASM
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BalanceResponse, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cosmwasm_std::{Decimal, Uint128};
use cw20::{Cw20Contract, Cw20QueryMsg};

// ERRORS & MESSAGES
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// CUSTOM CRATES
use crate::queries;
// execute

// STATE
use crate::state::{Config, Deposit, Funds, BALANCES, CONFIG, DEPOSIT, FUNDS};

// CW2
use cw2::set_contract_version;
const CONTRACT_NAME: &str = "crates.io:yield-aggregator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::wasmswap_msg::InfoResponse;
use crate::wasmswap_msg::QueryMsg as WasmSwapQueryMsg;
use crate::wasmswap_msg::Token1ForToken2PriceResponse;
use crate::wasmswap_msg::Token2ForToken1PriceResponse;
use crate::wasmswap_msg::{ExecuteMsg as WasmSwapExecuteMsg, TokenSelect};

const SWAP_REPLY_ID: u64 = 1u64;
const QUERY_REPLY_ID: u64 = 2u64;

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
        ExecuteMsg::PrepareLiquidity { token_bought } => {
            prepare_liquidity(deps, env, info, token_bought)
        }
        ExecuteMsg::AddLiquidity {} => add_liquidity(deps, env, info),
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
    _env: Env,
    info: MessageInfo,
    pool_addr: Addr,
    token1_amount: Uint128,
    token2_amount: Uint128,
) -> Result<Response, ContractError> {
    let pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(pool_addr.clone(), &WasmSwapQueryMsg::Info {})?;

    let token1_denom = &pool_info.token1_denom;
    let token2_denom = &pool_info.token2_denom;

    let min_token1_for_price = max(token1_amount, Uint128::one());
    let min_token2_for_price = max(token2_amount, Uint128::one());

    let token1_to_token2_price_response: Token1ForToken2PriceResponse =
        deps.querier.query_wasm_smart(
            pool_addr.clone(),
            &WasmSwapQueryMsg::Token1ForToken2Price {
                token1_amount: min_token1_for_price.clone(),
            },
        )?;
    let token2_to_token1_price_response: Token2ForToken1PriceResponse =
        deps.querier.query_wasm_smart(
            pool_addr.clone(),
            &WasmSwapQueryMsg::Token2ForToken1Price {
                token2_amount: min_token2_for_price,
            },
        )?;

    let mut attrs: Vec<(&str, String)> = vec![];
    let token1_price = Decimal::from_ratio(
        token1_to_token2_price_response.token2_amount.clone(),
        min_token1_for_price.clone(),
    );
    let token2_price = Decimal::from_ratio(
        token2_to_token1_price_response.token1_amount.clone(),
        min_token2_for_price.clone(),
    );

    let token1_value: Uint128 = token1_amount.mul(token1_price.clone());
    let token2_value = token2_amount.mul(token2_price.clone());

    let total_token1_value = token1_amount + token2_value;
    let total_token2_value = token2_amount + token1_value;

    let mut token1_amount_to_swap = Uint128::zero();
    let mut token2_amount_to_swap = Uint128::zero();

    if token1_value > token2_amount {
        if token2_amount.is_zero() {
            token1_amount_to_swap = Uint128::one().mul(Decimal::from_ratio(token1_amount, 2u128));
        } else {
            let token2_required_amount =
                Uint128::one().mul(Decimal::from_ratio(total_token2_value, 2u128));
            let token2_missing_amount = token2_required_amount - token2_amount;
            token1_amount_to_swap = token2_missing_amount * token2_price;
        }
    } else if token2_value > token1_amount {
        if token1_amount.is_zero() {
            token2_amount_to_swap = Uint128::one().mul(Decimal::from_ratio(token2_amount, 2u128));
        } else {
            let token1_required_amount =
                Uint128::one().mul(Decimal::from_ratio(total_token1_value, 2u128));
            let token1_missing_amount = token1_required_amount - token1_amount;
            token2_amount_to_swap = token1_missing_amount * token1_price;
        }
    };

    let mut res = Response::new();
    res = res.add_attributes(vec![
        ("token1_amount_to_swap", token1_amount_to_swap.to_string()),
        ("token2_amount_to_swap", token2_amount_to_swap.to_string()),
    ]);

    let token_amount_to_swap: Uint128;
    let mut deposit = Deposit {
        token1_amount,
        token1_denom: token1_denom.clone(),
        token2_amount,
        token2_denom: token2_denom.clone(),
        swapped_token: String::new(),
        pool_addr: pool_addr.clone(),
    };
    let swap_fee = 50u128;
    if token1_amount_to_swap + token2_amount_to_swap > Uint128::zero() {
        let token_to_swap = if token1_amount_to_swap > token2_amount_to_swap {
            token_amount_to_swap = token1_amount_to_swap;
            deposit.token1_amount = token1_amount.sub(token_amount_to_swap);
            deposit.token1_amount = deposit.token1_amount
                - Uint128::one().mul(Decimal::from_ratio(deposit.token1_amount, swap_fee));
            deposit.swapped_token = denom_to_string(token1_denom);
            (TokenSelect::Token1, token1_denom)
        } else {
            token_amount_to_swap = token2_amount_to_swap;
            deposit.token2_amount = token2_amount.sub(token_amount_to_swap);
            deposit.token2_amount = deposit.token2_amount
                - Uint128::one().mul(Decimal::from_ratio(deposit.token2_amount, swap_fee));
            deposit.swapped_token = denom_to_string(&deposit.token2_denom);
            (TokenSelect::Token2, token2_denom)
        };
        DEPOSIT.save(deps.storage, &deposit).unwrap();
        /*
        &WasmSwapExecuteMsg::SwapAndSendTo {
                                input_token: token_to_swap.0,
                                input_amount: token_amount_to_swap.clone(),
                                recipient: info.sender.to_string(),
                                min_token: Uint128::zero(),
                                expiration: None,
                            }
                 */

        if denom_is_native(token_to_swap.1) {
            let denom_str = denom_to_string(&token_to_swap.1);
            res = res.add_submessages(vec![SubMsg {
                id: SWAP_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: pool_addr.to_string(),
                    msg: to_binary(&WasmSwapExecuteMsg::Swap {
                        input_token: token_to_swap.0,
                        input_amount: token_amount_to_swap.clone(),
                        min_output: Uint128::zero(),
                        expiration: None,
                    })
                    .unwrap(),
                    funds: vec![coin(token_amount_to_swap.u128(), denom_str)],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Always,
            }]);
        } else {
            res = res.add_submessages(vec![
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: denom_to_string(&token_to_swap.1),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: pool_addr.to_string(),
                        amount: token_amount_to_swap.clone(),
                        expires: None,
                    })
                    .unwrap(),
                    funds: vec![],
                })),
                SubMsg {
                    id: SWAP_REPLY_ID,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: pool_addr.to_string(),
                        msg: to_binary(&WasmSwapExecuteMsg::Swap {
                            input_token: token_to_swap.0,
                            input_amount: token_amount_to_swap.clone(),
                            min_output: Uint128::zero(), //todo: slippage
                            expiration: None,
                        })
                        .unwrap(),
                        funds: vec![coin(
                            token_amount_to_swap.u128(),
                            denom_to_string(&token_to_swap.1),
                        )],
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Always,
                },
            ]);
        };
    } else {
        //TODO: Call Add Liquidity
    };
    Ok(res)
}

pub fn prepare_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_bought: Uint128,
) -> Result<Response, ContractError> {
    let mut deposit = DEPOSIT.load(deps.storage).unwrap();
    let token1_denom = denom_to_string(&deposit.token1_denom);
    let token2_denom = denom_to_string(&deposit.token2_denom);
    if token1_denom == deposit.swapped_token {
        deposit.token2_amount = deposit.token2_amount + token_bought;
    } else {
        deposit.token1_amount = deposit.token1_amount + token_bought;
    }
    DEPOSIT.save(deps.storage, &deposit).unwrap();

    let token1_balance = deps
        .querier
        .query_balance(
            env.contract.address.to_string(),
            denom_to_string(&deposit.token1_denom),
        )
        .unwrap();
    let cw20addr = deps
        .api
        .addr_validate(&denom_to_string(&deposit.token2_denom))
        .unwrap();
    let token2_balance_response: BalanceResponse = deps
        .querier
        .query_wasm_smart(
            cw20addr.clone(),
            &Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )
        .unwrap_or(BalanceResponse {
            amount: coin(0u128, denom_to_string(&deposit.token2_denom)),
        });

    BALANCES
        .save(deps.storage, token1_denom.clone(), &token1_balance.amount)
        .unwrap();
    BALANCES
        .save(
            deps.storage,
            token2_denom.clone(),
            &token2_balance_response.amount.amount,
        )
        .unwrap();
    // Deduct Swap Fee
    let swap_fee = 50u128;
    let token1_amount_to_add = deposit.token1_amount
        - Uint128::one().mul(Decimal::from_ratio(deposit.token1_amount, swap_fee));

    let res = Response::new().add_attributes(vec![
        ("cw20addr", cw20addr.to_string()),
        ("token1_amount", token1_amount_to_add.to_string()),
        ("max_token2", deposit.token2_amount.to_string()),
        ("token1_balance", token1_balance.amount.to_string()),
        (
            "token2_balance",
            token2_balance_response.amount.amount.to_string(),
        ),
    ]);
    Ok(res)
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut deposit = DEPOSIT.load(deps.storage).unwrap();
    let token1_denom = denom_to_string(&deposit.token1_denom);
    let token2_denom = denom_to_string(&deposit.token2_denom);
    let mut token1_balance = BALANCES.load(deps.storage, token1_denom.clone()).unwrap();
    token1_balance =
        token1_balance - Uint128::one().mul(Decimal::from_ratio(token1_balance, 50u128));
    let token2_balance = BALANCES.load(deps.storage, token2_denom.clone()).unwrap();
    let cw20addr = deps
        .api
        .addr_validate(&denom_to_string(&deposit.token2_denom))
        .unwrap();
    let mut funds: Vec<Coin> = vec![];
    if denom_is_native(&deposit.token1_denom) {
        funds.push(coin(token1_balance.u128(), token1_denom.clone()));
    }
    if denom_is_native(&deposit.token2_denom) && !deposit.token2_amount.is_zero() {
        funds.push(coin(deposit.token2_amount.u128(), token2_denom.clone()));
    }

    // let token1_amount = Uint128::one().mul(Decimal::from_ratio(token1_balance, 2u128));
    let res = Response::default().add_submessages(vec![
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: deposit.pool_addr.to_string(),
                amount: deposit.token2_amount.clone(),
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deposit.pool_addr.to_string(),
            msg: to_binary(&WasmSwapExecuteMsg::AddLiquidity {
                token1_amount: token1_balance.clone(),
                min_liquidity: Uint128::zero(),
                max_token2: deposit.token2_amount.clone(),
                expiration: None,
            })
            .unwrap(),
            funds,
        })),
    ]);
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        SWAP_REPLY_ID => {
            let mut token_bought = Uint128::zero();
            let events = msg.result.unwrap().events;
            for e in events {
                if e.ty.eq("wasm") {
                    for attr in e.attributes {
                        if attr.key.eq("token_bought") {
                            token_bought = Uint128::from_str(&attr.value).unwrap();
                        }
                    }
                }
            }
            let funds: Vec<Coin> = vec![];
            let res =
                Response::new().add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::PrepareLiquidity { token_bought }).unwrap(),
                    funds,
                })));
            Ok(res)
        }
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}
