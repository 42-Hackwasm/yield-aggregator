#!/bin/bash

set -e

JUNOD_NODE="https://rpc.uni.junonetwork.io:443" 
KEY="testwasm"
KEY_ADDR="juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"
GAS="--gas-adjustment 1.1 --gas 2000000 --gas-prices=0.035ujunox"
CHAIN_ID="--chain-id uni-5"

EXECUTE_GAS="--gas-adjustment 1.1 --gas 500000 --gas-prices=0.025ujunox"

YIELD_CONTRACT=$(junod tx wasm store artifacts/yield_optimizer.wasm --from testwasm -y --broadcast-mode sync --output json $GAS $CHAIN_ID --node $JUNOD_NODE --keyring-backend test | jq -r '.txhash') && echo $YIELD_CONTRACT
sleep 8
CODE_ID_YIELD=$(junod query tx $YIELD_CONTRACT --output json | jq -r '.logs[0].events[-1].attributes[0].value') && echo "Code ID: $CODE_ID_YIELD" # 329
sleep 8
YIELD_TX_INIT=$(junod tx wasm instantiate "$CODE_ID_YIELD" '{"contract_admin": "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"}' --label "42-hackwasm" $EXECUTE_GAS $CHAIN_ID --broadcast-mode sync --output json -y --admin $KEY_ADDR --from $KEY --keyring-backend test | jq -r '.txhash') && echo $YIELD_TX_INIT
sleep 8
ADDR_YIELD=$(junod query tx $YIELD_TX_INIT --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "Contract Address for Yield: $ADDR_YIELD"
# export ADDR_YIELD=juno1h0knfsptrhq24ejpf5v9jje48hv24zpuxe385hmzj5ck286346lspyykl8
# DEPOSIT
sleep 8
RESULT=$(junod tx wasm execute $ADDR_YIELD '{"deposit":{"pool_addr":"juno1j4ezvp80mnn75hlngak35n6wwzemxqjxdncdxc5n9dfw6s0q080qyhh9zl", "token1_amount":"10000", "token2_amount":"0"}}' -y --from testwasm -b block --gas-adjustment 1.1 --gas 1550000 --gas-prices=0.025ujunox --keyring-backend test --amount 10000ujunox) && echo $RESULT