use cosmwasm_std::{from_slice, Addr, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128};

// use cosmwasm_std::{to_binary, IbcMsg, IbcTimeout, IbcTimeoutBlock};

use cw20::Cw20ReceiveMsg;
use cw_denom::Asset;

use crate::error::ContractError;
use crate::msg::ReceiveMsg;
use crate::state::{BORROWS, COLLATERAL};

pub fn collaterize(deps: DepsMut, user: &Addr, coin: Asset) -> Result<Response, ContractError> {
    if coin.amount.is_zero() {
        return Err(ContractError::NoFundsSend {});
    }
    let mut funds = COLLATERAL.may_load(deps.storage, user)?.unwrap_or_default();

    let mut found = false;
    for i in 0..funds.len() {
        if funds[i].denom == coin.denom {
            funds[i].amount += coin.amount;
            found = true;
        }
    }
    if !found {
        funds.push(coin.clone());
    }

    COLLATERAL.save(deps.storage, user, &funds)?;
    Ok(Response::new())
}

/// native transfer for Bank Denoms
pub fn borrow(
    deps: DepsMut,
    user: &Addr,
    denom: cw_denom::CheckedDenom,
    amount: Uint128,
) -> Result<Response, ContractError> {
    BORROWS.update(deps.storage, user, |b| -> Result<_, ContractError> {
        if let Some(mut b) = b {
            let mut found = false;
            for i in 0..b.len() {
                if b[i].denom == denom {
                    b[i].amount += amount;
                    found = true;
                }
            }
            if !found {
                b.push(Asset {
                    denom: denom.clone(),
                    amount,
                });
            }
            return Ok(b);
        }
        Ok(vec![Asset {
            denom: denom.clone(),
            amount,
        }])
    })?;
    let msg = denom.get_transfer_to_message(user, amount)?;
    Ok(Response::new().add_submessage(SubMsg::new(msg)))
}

/// handler for cw20 *Send messages.
pub fn cw20_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // info.sender is the address of the cw20 contract (that re-sent this message).
    // wrapper.sender is the address of the user that requested the cw20 contract to send this.
    // This cannot be fully trusted (the cw20 contract can fake it), so only use it for actions
    // in the address's favor (like paying/bonding tokens, not withdrawls)
    let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
    match msg {
        ReceiveMsg::Collaterize {} => {
            let user = &deps.api.addr_validate(&wrapper.sender)?;
            collaterize(
                deps,
                user,
                Asset::new_cw20_checked(info.sender, wrapper.amount),
            )
        }
    }
}
