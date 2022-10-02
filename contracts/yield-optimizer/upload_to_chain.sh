#!/bin/bash

set -e

JUNOD_NODE="https://rpc.uni.junonetwork.io:443" 
KEY="testwasm"
KEY_ADDR="juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"
GAS="--gas-adjustment 1.1 --gas 1500000 --gas-prices=0.025ujunox"
CHAIN_ID="--chain-id uni-5"

EXECUTE_GAS="--gas-adjustment 1.1 --gas 500000 --gas-prices=0.025ujunox"

YIELD_CONTRACT=$(junod tx wasm store artifacts/yield_optimizer.wasm --from testwasm -y --broadcast-mode sync --output json $GAS $CHAIN_ID --node $JUNOD_NODE | jq -r '.txhash') && echo $YIELD_CONTRACT
sleep 7
CODE_ID_YIELD=$(junod query tx $YIELD_CONTRACT --output json | jq -r '.logs[0].events[-1].attributes[0].value') && echo $CODE_ID_YIELD # 329
YIELD_TX_INIT=$(junod tx wasm instantiate "$CODE_ID_YIELD" '{"contract_admin": "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"}' --label "42-hackwasm" $EXECUTE_GAS $CHAIN_ID --broadcast-mode sync --output json -y --admin $KEY_ADDR --from $KEY | jq -r '.txhash') && echo $YIELD_TX_INIT
sleep 7
ADDR_YIELD=$(junod query tx $YIELD_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Contract Address for Yield: $ADDR_YIELD"
# export ADDR_YIELD=juno140zrfkpjnxgeq6w9l9av3hxdgvpulqekgmsqeenjg5je6syuzz2ss9xmqx


# migrate
# junod tx wasm migrate $ADDR_YIELD $CODE_ID_YIELD '{"migrate_msg":{}}' --gas-prices="0.025ujunox" -y --from $KEY --chain-id uni-5

# ! test adding funds to the contract
junod tx wasm execute $ADDR_YIELD '{"add_funds":{}}' --from $KEY -y --broadcast-mode sync --output json $EXECUTE_GAS $CHAIN_ID --node $JUNOD_NODE --amount 1ujunox,2uusdcx

junod tx wasm execute $ADDR_YIELD '{"withdraw_funds":{"denom": "uusdcx", "amount": "1"}}' --from $KEY -y --broadcast-mode sync --output json $EXECUTE_GAS $CHAIN_ID --node $JUNOD_NODE

junod tx wasm execute $ADDR_YIELD '{"create_vault": {"vault": {"is_active": true, "chain": "juno", "dex": "junoswap", "pool_contract_address": "addr2...", "lp_token_contract_address": "addr1..", "reward_tokens": [{"native": "utest"}], "token1": { "native": "utoken1" }, "token2": { "cw20": "addr1.." }, "total_shares": "0"}}}' --from $KEY -y --broadcast-mode sync --output json $EXECUTE_GAS $CHAIN_ID --node $JUNOD_NODE

junod tx wasm execute $ADDR_YIELD '{"disable_vault": {"vault_id": 1}}' --from $KEY -y --broadcast-mode sync --output json $EXECUTE_GAS $CHAIN_ID --node $JUNOD_NODE

# ! query funds of a user
#junod query wasm contract-state smart $ADDR_YIELD '{"get_funds":{"address": "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"}}' --node $JUNOD_NODE

# ! query vaults created
junod query wasm contract-state smart $ADDR_YIELD '{"get_vaults":{}}' --node $JUNOD_NODE