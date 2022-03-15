#!/bin/bash

# Extracts the gas usage from the `ink-waterfall` log.
#
# Usage:
#   ./extract-gas-usage.sh <ink-example-name>

set -eu
set -o pipefail

EXAMPLE=$(basename $1)

# We need to account for the case when an example cannot be found in the log.
# This can be the case when e.g. somebody adds a new ink! example in an ink! PR,
# but the `ink-waterfall` not yet containing any test for this new example.
COUNT=$(grep --count "example: $EXAMPLE, " /tmp/waterfall.log) || true
if [ "$COUNT" -eq "0" ]; then
  echo "$EXAMPLE, 0"
  exit 0;
fi

USAGE=$(cat /tmp/waterfall.log |
  grep "example: $EXAMPLE, " |
  # This next line is an unfortunate hack that we need to have
  # for the moment. The reason is that there might be a retry within
  # a test, due to the UI to being responsive in time. This retry
  # then results in another "estimated gas for transaction" line
  # in the log.
  #
  # Until we have a frontend with RPC for proper checking of the
  # consumed gas we need to use this hack to ensure that the gas
  # usage per example is the same, independent of how many retries
  # there were.
  #
  # The `sed` command removes everything before the first space, i.e.
  # the `[2021-12-01T14:13:42Z INFO` stuff before `INFO` -- this way
  # it's possible to uniq-ify the lines to ensure that no duplicate
  # "estimated gas for transaction" from some retry is counted.
  # This kind of skews the true gas costs, but since this whole
  # endeavor is mostly about having a rough idea now, this should
  # be fine.
  sed 's/[^ ]*//' | sort | uniq |
  egrep --only-matching "estimated gas for transaction is [0-9]*" |
  egrep --only-matching "[0-9]*" |
  awk '{s+=$1} END {print s}')

echo "$EXAMPLE, $USAGE"
