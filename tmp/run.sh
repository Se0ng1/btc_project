#!/usr/bin/env bash
set -euo pipefail

docker build -t btc_node_test .
docker run --rm btc_node_test
