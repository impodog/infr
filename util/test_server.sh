#!/bin/bash
trap "kill 0" EXIT
cargo build -p infr_server
RUST_LOG="infr_solver=info,infr_layout=info,infr_server=info" ./target/debug/infr_server &
python -i util/test_util.py
