use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Deps};
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
    pub funds: Vec<Coin>
}

pub const CONFIG: Item<Config> = Item::new("config");

// address.to_string() => Funds array
pub const FUNDS: Map<String, Funds> = Map::new("funds");


