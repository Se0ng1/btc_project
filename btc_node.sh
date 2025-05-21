#!/usr/bin/env bash

set -euo pipefail

sudo apt update
sudo apt install -y \
    libevent-dev \
    libsqlite3-dev \
    autoconf \
    automake \
    libtool \
    pkg-config \
    build-essential \
    bison \
    curl \
    wget \
    unzip

BOOST_VER=1_83_0
BOOST_DIR=boost$BOOST_VER

if [ ! -d "$BOOST_DIR" ]; then
    wget -c https://sourceforge.net/projects/boost/files/boost/1.83.0/boost_1_83_0.tar.bz2
    tar -xvf boost_1_83_0.tar.bz2
    pushd $BOOST_DIR
    ./bootstrap.sh
    sudo ./b2 install
    popd
fi

rm -rf bitcoin-core-cat
git clone --depth 1 --branch dont-success-cat https://github.com/rot13maxi/bitcoin.git bitcoin-core-cat

pushd bitcoin-core-cat
./autogen.sh
./configure --enable-wallet --with-sqlite --with-incompatible-bdb --without-gui --without-tests --disable-bench
make -j"$(nproc)"
popd

mkdir -p ./bit_data

./bitcoin-core-cat/src/bitcoind \
    -regtest \
    -timeout=15000 \
    -daemon \
    -server=1 \
    -txindex=1 \
    -rpcuser=user \
    -rpcpassword=password \
    -datadir=./bit_data &

sleep 3


if ./bitcoin-core-cat/src/bitcoin-cli \
    -regtest \
    -rpcuser=user \
    -rpcpassword=password \
    decodescript 01aa01bb7e | grep -q "OP_CAT"; then
    echo "OP_CAT is supported by this node"
else
    echo "OP_CAT is NOT supported by this node"
    exit 1
fi

# 종료
./bitcoin-core-cat/src/bitcoin-cli \
    -regtest \
    -rpcuser=user \
    -rpcpassword=password \
    stop
