use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, Deps, Uint128};
use cw_storage_plus::{Item, Map};

use crate::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr, // DAO / Multisig
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Funds {
    pub funds: Vec<Coin>,
}

pub const CONFIG: Item<Config> = Item::new("config");

// address.to_string() => Funds array
pub const FUNDS: Map<String, Funds> = Map::new("funds");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Deposit {
    pub token1_amount: Uint128,
    pub token1_denom: Denom,
    pub token2_amount: Uint128,
    pub token2_denom: Denom,
    pub pool_addr: Addr,
    pub swapped_token: String,
}

pub const BALANCES: Map<String, Uint128> = Map::new("balances");
pub const DEPOSIT: Item<Deposit> = Item::new("deposit");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub pool_share: Decimal,
    pub pool_token1_balance: Uint128,
    pub pool_token2_balance: Uint128,
    pub token1_denom: String,
    pub token2_denom: String,
}
