#!/bin/bash
trap "kill 0" EXIT
RUST_LOG="infr_solver=info,infr_layout=info,infr_server=info" cargo run -p infr_server &
python -i test_util.py
