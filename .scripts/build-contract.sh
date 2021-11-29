#!/bin/bash
#
# Write to the stdout the original and optimized sizes of a Wasm file
# as a CSV format line.
#
# Usage: `./build-contract.sh <path_to_contract>`

set -eux

CONTRACT=$(basename $1)
cargo +nightly contract build --release --manifest-path $1/Cargo.toml --output-json > /tmp/size_$CONTRACT
SIZE_OUT=$(cat /tmp/size_$CONTRACT)
ORIGINAL_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.original_size')
OPTIMIZED_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.optimized_size')

echo -n "${CONTRACT}, ${ORIGINAL_SIZE}, ${OPTIMIZED_SIZE}"
