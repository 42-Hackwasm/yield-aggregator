use cosmwasm_schema::write_api;

use yield_optimizer::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
// use yield_optimizer::state::Config;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        migrate: MigrateMsg,
    }
}
