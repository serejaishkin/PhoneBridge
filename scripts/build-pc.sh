#!/bin/bash
set -e
cd "$(dirname "$0")/../pc"
cargo build --release
echo "Binary: target/release/phonebridge"
