#[cfg(not(feature = "library"))]
// COSMWASM
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, IbcMsg, IbcTimeout, IbcTimeoutBlock,
    MessageInfo, Response, StdError, StdResult, SubMsg, Uint128
};

use crate::coin_helpers::convert_coins_vec_to_string;
// ERRORS & MESSAGES
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// CUSTOM CRATES
use crate::queries;
// execute

// STATE
use crate::state::{Config, CONFIG, FUNDS, Funds, Vault, VAULTS, VAULTS_COUNTER};

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
        ExecuteMsg::AddFunds {} => {
            let address = info.sender.to_string();
            // let users_funds = FUNDS.may_load(deps.storage, address.clone())?;

            // check if info.funds.clone() is empty
            if info.funds.clone().is_empty() {
                return Err(ContractError::NoFundsSend {});
            }

            // load FUNDS from stroage, if none, create new Funds & save. Or else just append funds to existing Funds
            let mut funds = FUNDS
                .may_load(deps.storage, address.clone())?
                .unwrap_or_else(|| Funds { funds: vec![] });

            // loop through funds.funds & add coins if they are the same denom
            for coin in info.funds.clone() {
                let mut found = false;
                for i in 0..funds.funds.len() {
                    if funds.funds[i].denom == coin.denom {
                        funds.funds[i].amount += coin.amount;
                        found = true;
                    }
                }
                if !found {
                    funds.funds.push(coin);
                }
            }

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

        // TODO: can we set array of coins here instead? how to send through via CLI/Website?
        ExecuteMsg::WithdrawFunds { denom, amount } => {
            let address = info.sender.to_string();
            let funds = FUNDS.may_load(deps.storage, address.clone())?;

            // check if funds is none
            if funds.is_none() {
                return Err(ContractError::NoFundsInContract {});
            } else {
                // loop through funds.funds & subtract coins if they are the same denom
                let mut funds: Funds = funds.unwrap();
                for coin in funds.funds.clone() {
                    if coin.denom == denom {
                        if coin.amount < amount {
                            return Err(ContractError::NotEnoughFunds {
                                denom: denom,
                                amount: amount.to_string(),
                            });
                        } else {
                            funds.funds.retain(|c| c.denom != denom);
                            if coin.amount > amount {
                                // funds.funds.push(Coin { denom: denom.clone(), amount: coin.amount - amount });
                                let idx = funds.funds.iter().position(|c| c.denom == denom);

                                if let Some(idx) = idx {
                                    funds.funds[idx].amount -= amount;
                                }
                            }
                        }
                    }
                }
                FUNDS.save(deps.storage, address.clone(), &funds)?;
            }

            Ok(Response::new()
                .add_attribute("action", "withdraw_funds")
                .add_message(BankMsg::Send {
                    to_address: address.clone(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: amount,
                    }],
                }))
        }

        // move funds to another chains vault contract address
        ExecuteMsg::TransferFunds {
            recipient_contract_address,
            channel_id,
            denom,
            amount,
        } => {
            let address = info.sender.to_string();
            let funds = FUNDS.may_load(deps.storage, address.clone())?;

            // check if funds is none
            if funds.is_none() {
                return Err(ContractError::NoFundsInContract {});
            }

            let ibc_timeout_block = IbcTimeout::with_block(IbcTimeoutBlock {
                height: env.block.height + 1000,
                revision: 0,
            });

            // TODO: we should instead somehow use ICA, need to find documentation on ICA

            // transfer via an IBC channel
            let ibc_msg = IbcMsg::Transfer {
                /// exisiting channel to send the tokens over
                channel_id: channel_id.clone(),
                /// address on the remote chain to receive these tokens
                to_address: recipient_contract_address,
                /// packet data only supports one coin
                /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
                amount: Coin {
                    denom: denom.clone(),
                    amount: amount,
                },
                /// when packet times out, measured on remote chain
                timeout: ibc_timeout_block.clone(),
            };

            let ibc_msg_2 = IbcMsg::SendPacket {
                channel_id: channel_id,
                data: Binary::from(b"hello world"),
                timeout: ibc_timeout_block,
            };

            let ibc_submsg: SubMsg<_> = SubMsg::new(ibc_msg);
            let ibc_submsg_2: SubMsg<_> = SubMsg::new(ibc_msg_2);

            Ok(
                Response::new()
                    .add_attribute("action", "transfer_funds")
                    // .add_messages(vec![ibc_msg, ibc_msg_2])
                    .add_submessage(ibc_submsg)
                    .add_submessage(ibc_submsg_2), // this way we can get the replies later & ensure it was sucessful. Do we need that though for main hub?
            )
        },
        ExecuteMsg::CreateVault { vault } => execute_create_vault(deps, env, info, vault),
        ExecuteMsg::DisableVault { vault_id } => execute_disable_vault(deps, env, info, vault_id)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&queries::query_config(deps)?),

        QueryMsg::GetFunds { address } => to_binary(&queries::get_funds(deps, address)?),
        
        QueryMsg::GetVaults {} => to_binary(&queries::get_vaults(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

fn execute_create_vault(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault: Vault
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.to_string() != config.admin.to_string() {
        Err(ContractError::Unauthorized {  })
    } else {
        let id: Uint128 = VAULTS_COUNTER.may_load(deps.storage)?.unwrap_or_default() + Uint128::from(1u128);
        VAULTS_COUNTER.save(deps.storage, &id)?;
        VAULTS.save(deps.storage, id.u128(), &vault).unwrap();
        Ok(Response::new())
    }
}

fn execute_disable_vault(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_id: u128
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin.to_string() {
        Err(ContractError::Unauthorized {  })
    } else {
        match VAULTS.may_load(deps.storage, vault_id)? {
            Some(mut vault) => {
                vault.is_active = false;
                VAULTS.save(deps.storage, vault_id, &vault)?;
                Ok(Response::new())
            },
            None => Err(ContractError::VaultDoesntExist {})
        }
    }
}