use std::vec;

use cosmwasm_std::{Decimal, Deps, Env, StdResult};

use crate::denom_utils::denom_to_string;
use crate::wasmswap_msg::QueryMsg as WasmSwapQueryMsg;
use crate::{msg::ConfigResponse, wasmswap_msg::InfoResponse};
use cw20::{BalanceResponse, Cw20QueryMsg};

use crate::state::{Funds, Position, CONFIG, FUNDS};

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
    let funds = FUNDS.load(deps.storage, address)?;
    Ok(funds)
}
// Get Positions
pub fn get_positions(deps: Deps, env: Env) -> StdResult<Vec<Position>> {
    //TODO: Load vaults from storage
    let mut positions: Vec<Position> = vec![];

    let pools = vec![(
        "juno1j4ezvp80mnn75hlngak35n6wwzemxqjxdncdxc5n9dfw6s0q080qyhh9zl",
        "juno1hqkjtyk9lhykua80dxacqufzawcu55flug8j675xfpaxplselrasvkjmjl",
    )];
    pools.iter().for_each(|p| {
        // Query LP Token Balance
        let pool_addr = deps.api.addr_validate(p.0).unwrap();
        let lp_token_addr = deps.api.addr_validate(p.1).unwrap();
        let lp_token_balance: BalanceResponse = deps
            .querier
            .query_wasm_smart(
                lp_token_addr,
                &Cw20QueryMsg::Balance {
                    address: env.contract.address.to_string(),
                },
            )
            .unwrap();

        //Query Pool Info
        let pool_info: InfoResponse = deps
            .querier
            .query_wasm_smart(pool_addr.clone(), &WasmSwapQueryMsg::Info {})
            .unwrap();

        // Calculate our Share of the Pool
        let pool_share = Decimal::from_ratio(lp_token_balance.balance, pool_info.lp_token_supply);

        // Populate positions
        positions.push(Position {
            pool_share,
            pool_token1_balance: pool_info.token1_reserve,
            pool_token2_balance: pool_info.token2_reserve,
            token1_denom: denom_to_string(&pool_info.token1_denom),
            token2_denom: denom_to_string(&pool_info.token2_denom),
        });
    });

    Ok(positions)
}

pub fn get_pools(deps: Deps, env: Env) -> StdResult<Vec<InfoResponse>> {
    let pools_addrs = vec!["juno1j4ezvp80mnn75hlngak35n6wwzemxqjxdncdxc5n9dfw6s0q080qyhh9zl"];
    let mut pools: Vec<InfoResponse> = vec![];
    pools_addrs.iter().for_each(|p| {
        let pool_info: InfoResponse = deps
            .querier
            .query_wasm_smart(p.to_string(), &WasmSwapQueryMsg::Info {})
            .unwrap();
        pools.push(pool_info);
    });
    Ok(pools)
}
