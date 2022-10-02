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

// use std::env::current_dir;
// use std::fs::create_dir_all;

// use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

// use yield_optimizer::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, };

// fn main() {
//     let mut out_dir = current_dir().unwrap();
//     out_dir.push("schema");
//     create_dir_all(&out_dir).unwrap();
//     remove_schemas(&out_dir).unwrap();

//     export_schema(&schema_for!(InstantiateMsg), &out_dir);
//     export_schema(&schema_for!(ExecuteMsg), &out_dir);
//     export_schema(&schema_for!(QueryMsg), &out_dir);
//     export_schema(&schema_for!(MigrateMsg), &out_dir);
//     export_schema(&schema_for!(Config), &out_dir);
// }
