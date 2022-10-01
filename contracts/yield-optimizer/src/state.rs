use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Deps, Uint128};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub denom: String,
    pub token_contract_address: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vault {
    pub is_active: bool,
    pub chain: String,
    pub dex: String,
    pub lp_token_contract_address: String,
    pub earned_token_name: String,
    pub earned_token_address: String,
    pub token1: Asset,
    pub token2: Asset,
    pub total_shares: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserPosition {
    pub shares: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

// address.to_string() => Funds array
pub const FUNDS: Map<String, Funds> = Map::new("funds");
// vault id, vault
pub const VAULTS: Map<u128, Vault> = Map::new("vaults");
pub const VAULTS_COUNTER: Item<Uint128> = Item::new("vaults_counter");
// user address, vault id, position
pub const POSITIONS: Map<(String, Uint128), UserPosition> = Map::new("positions");



