#[cfg(test)]
// Example I like & am using:
// https://github.com/osmosis-labs/cw-usdc/blob/main/contracts/cw-usdc/src/contract_tests.rs
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
#[allow(unused_imports)]
use cosmwasm_std::{coins, from_binary, Deps, MessageInfo, Uint128};
use cosmwasm_std::DepsMut;

use crate::contract;
use crate::msg::{InstantiateMsg, ConfigResponse, QueryMsg,};
// use crate::state::Config;

const NAME: &str = "crates.io:yield-optimizer";
const VERSION: &str = "0.1.0";
const DENOM: &str = "utoken";
const ADMIN_STR_ADDR: &str = "admin";


#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let contract_admin = initialize_contract(deps.as_mut());

    // could also call the query directly from the queries file itself. But wouldn't test the messages
    let res: ConfigResponse = from_binary(        
        &contract::query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
    ).unwrap();

    assert_eq!(res.name, NAME.to_string());
    assert_eq!(res.version, VERSION);
    assert_eq!(res.admin, contract_admin);
}


// ==== TEST HELPERS ====
fn initialize_contract(deps: DepsMut) -> String {
    let msg = InstantiateMsg {
        contract_admin: Some(ADMIN_STR_ADDR.to_string()),
    };

    let info = mock_info("creator", &coins(1000000, &DENOM.to_string()));
    contract::instantiate(deps, mock_env(), info, msg.clone()).unwrap();

    // (msg.denom, msg.fee_receive_address, msg.platform_fee)
    msg.contract_admin.unwrap()
}