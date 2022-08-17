#!/bin/bash
#
# Write to the stdout the original and optimized sizes of a Wasm file
# as a CSV format line.
#
# Usage: `./build-contract.sh <path_to_contract>`

set -eux
set -o pipefail

CONTRACT=$(basename $1)
SIZE_OUT=$(RUST_LOG="" cargo +nightly contract build --release --manifest-path $1/Cargo.toml --output-json 2> /tmp/err) || exit $?
OPTIMIZED_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.optimized_size')

echo -n "${CONTRACT}, ${OPTIMIZED_SIZE}"
