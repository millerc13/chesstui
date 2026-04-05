#!/bin/bash
set -a
source "$(dirname "$0")/.env"
set +a
cargo run -- server --bind 0.0.0.0:7600
