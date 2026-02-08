#!/bin/bash
trap "kill 0" EXIT
cargo run -p infr_server &
cargo run -p infr_main
