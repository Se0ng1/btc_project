#!/usr/bin/env bash
set -euo pipefail

mkdir -p ./bit_data

./bitcoin-core-cat/src/bitcoind \
    -regtest \
    -timeout=15000 \
    -daemon \
    -server=1 \
    -txindex=1 \
    -rpcuser=user \
    -rpcpassword=password \
    -datadir=./bit_data

sleep 5