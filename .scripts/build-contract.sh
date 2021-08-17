#!/bin/bash
#
# Write the original and optimized sizes of a Wasm file
# to a CSV file.
#
# Usage: `./build-contract.sh <path_to_contract> <path_to_output_file>`

set -eux

CONTRACT=$(basename $1)
SIZE_OUT=$(cargo contract build --manifest-path $1/Cargo.toml --output-json)
ORIGINAL_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.original_size')
OPTIMIZED_SIZE=$(echo $SIZE_OUT | jq '.optimization_result.optimized_size')
echo "${CONTRACT}, ${ORIGINAL_SIZE}, ${OPTIMIZED_SIZE}" >> $2.csv
