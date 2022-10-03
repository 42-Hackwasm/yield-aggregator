use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_denom::Asset;
use cw_storage_plus::{Item, Map};

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[cw_serde]
pub struct Config {}

pub const CONFIG: Item<Config> = Item::new("config");

// address => Funds array
pub const COLLATERAL: Map<&Addr, Vec<Asset>> = Map::new("f");
pub const BORROWS: Map<&Addr, Vec<Asset>> = Map::new("b");

// // vault id, vault
// pub const VAULTS: Map<u128, Vault> = Map::new("vaults");
// pub const VAULTS_COUNTER: Item<Uint128> = Item::new("vaults_counter");
// // user address, vault id, position
// pub const POSITIONS: Map<(String, Uint128), UserPosition> = Map::new("positions");
