# sudo npm i -g @confio/relayer@main

# cd to script home dir
cd "$(dirname "$0")"

JUNOD_NODE="https://rpc.uni.junonetwork.io:443" # testnet RPC
# OSMO_NODE="https://rpc-test.osmosis.zone:443" # testnet RPC
OSMO_NODE="https://osmosis-rpc.polkachu.com:443" # mainnet RPC

# ibc-setup init --home ./relayer # only needed to start it, ensure you remove all commas "," given they need to be numbers. CTRL+SHIFT+H , -> ""


# ibc-setup init --src osmosis --dest juno --home ./relayer/
# update mnemonic in there to be the test one we use for both chains
# test peanut elevator motor proud globe obtain gasp sad balance nature ladder

# ibc-setup keys list --home ./relayer/

# ibc-setup ics20 -v --home ./relayer/

# echo "Starting relayer in the background, kill with 'killall ibc-relayer'"
# ibc-relayer start -v --poll 6 --home ./relayer/ &

# # test sending a token from OSMO -> JUNO





# GO version
# rly config init --memo "hackwasm 42 relayer" --home ./go-relay/

rly chains add osmosis --home ./go-relay/
rly chains add juno --home ./go-relay/
# rly chains add --file uni-5.json --home ./go-relay/
# rly chains add --file ./go-relay/config/osmosis-1.json --home ./go-relay/


# juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4
# osmo10c3slrqx3369mfsr9670au22zvq082ja8mh8gm
rly keys restore juno testwasm "test peanut elevator motor proud globe obtain gasp sad balance nature ladder" --home ./go-relay/
# rly keys restore uni-5 testwasm "test peanut elevator motor proud globe obtain gasp sad balance nature ladder" --home ./go-relay/
rly keys restore osmosis testwasm "test peanut elevator motor proud globe obtain gasp sad balance nature ladder" --home ./go-relay/
# rly keys delete osmo-test-4 relayer --home ./go-relay/

# getting osmosis tokens on junoswap
# rly paths new osmosis uni-5 mosmo_juno --home ./go-relay/
rly paths new osmosis juno mosmos_to_mjuno --home ./go-relay/

rly tx connect mosmo_juno --src-port transfer --dst-port transfer --order unordered --version ics20-1 --home ./go-relay/


rly start osmo_juno --home ./go-relay/

# rly transact channel osmo_juno --src-port transfer --dst-port transfer --order unordered --version ics20-1 --home ./go-relay/



junod tx ibc-transfer transfer transfer channel-0 osmo10c3slrqx3369mfsr9670au22zvq082ja8mh8gm 2ujunox --from juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4 --node $JUNOD_NODE --chain-id uni-5 --fees 50ujunox --packet-timeout-height 0-0
osmosisd tx ibc-transfer transfer transfer channel-0 juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4 18uosmo --from osmo10c3slrqx3369mfsr9670au22zvq082ja8mh8gm --node $OSMO_NODE --chain-id osmosis-1 --packet-timeout-height 0-0