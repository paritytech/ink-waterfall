#!/bin/bash
#
# Write to the stdout the original and optimized sizes of a Wasm file
# as a CSV format line.
#
# Usage: `./build-contract.sh <path_to_contract>`

set -eux

CONTRACT=$(basename $1)
SIZE_OUT=$(cargo +nightly contract build --release --manifest-path $1/Cargo.toml --output-json) || exit $?
ORIGINAL_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.original_size')
OPTIMIZED_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.optimized_size')

echo -n "${CONTRACT}, ${ORIGINAL_SIZE}, ${OPTIMIZED_SIZE}"
