#!/bin/bash

set -e

JUNOD_NODE="https://rpc.uni.junonetwork.io:443" 
KEY="testwasm"
KEY_ADDR="juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"
GAS="--gas-adjustment 1.1 --gas 1250000 --gas-prices=0.025ujunox"
CHAIN_ID="--chain-id uni-5"

EXECUTE_GAS="--gas-adjustment 1.1 --gas 500000 --gas-prices=0.025ujunox"

YIELD_CONTRACT=$(junod tx wasm store artifacts/yield_optimizer.wasm --from testwasm -y --broadcast-mode sync --output json $GAS $CHAIN_ID --node $JUNOD_NODE | jq -r '.txhash') && echo $YIELD_CONTRACT
CODE_ID_YIELD=$(junod query tx $YIELD_CONTRACT --output json | jq -r '.logs[0].events[-1].attributes[0].value') && echo $CODE_ID_YIELD # 329
YIELD_TX_INIT=$(junod tx wasm instantiate "$CODE_ID_YIELD" '{"contract_admin": "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"}' --label "42-hackwasm" $EXECUTE_GAS $CHAIN_ID --broadcast-mode sync --output json -y --admin $KEY_ADDR --from $KEY | jq -r '.txhash') && echo $YIELD_TX_INIT
ADDR_YIELD=$(junod query tx $YIELD_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Contract Address for Yield: $ADDR_YIELD"
# export ADDR_YIELD=juno1cr3n7hjv739fnk0a2jh57qx9np3pn9kawt6nslkzjwn3r2fs5e4qfcqzqa