#!/bin/bash

set -eux

# TODO: Use `--output-json` when that gets merged
SIZE_OUT=""
SIZE_OUT=$(cargo contract build --manifest-path $1/Cargo.toml  | grep "Original" | cut -d " " -f 4,6)
echo "$1, ${SIZE_OUT}" >> output.csv #${BRANCH}.csv
