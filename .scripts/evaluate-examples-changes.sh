#!/bin/bash

# Evaluates size changes of the ink! examples and posts the results
# as a comment on GitHub.
#
# Usage:
#   ./evaluate-examples-size-changes.sh \
#     <path_to_size_baseline.csv> <path_to_size_change.csv> \
#     <path_to_gas_usage_baseline.csv> <path_to_gas_usage_change.csv> \
#     <github_url_to_comments_of_pr>

set -eu
set -o pipefail

PR_COMMENTS_URL=$1
BASELINE_FILE=$2
COMPARISON_FILE=$3

GAS_BASELINE_FILE=$4
GAS_COMPARISON_FILE=$5

echo "$BASELINE_FILE will be compared to $COMPARISON_FILE"

echo "BASELINE_FILE:"
cat $BASELINE_FILE

echo "COMPARISON_FILE:"
cat $COMPARISON_FILE

echo "$GAS_BASELINE_FILE will be compared to $GAS_COMPARISON_FILE"

echo "GAS_BASELINE_FILE:"
cat $GAS_BASELINE_FILE

echo "GAS_COMPARISON_FILE:"
cat $GAS_COMPARISON_FILE

echo " ,Δ Optimized Size,Δ Used Gas,Total Optimized Size, Total Used Gas" | tee diffs.csv
csv-comparator $BASELINE_FILE $COMPARISON_FILE $GAS_BASELINE_FILE $GAS_COMPARISON_FILE  | \
  # Remove 0.00 for display purposes
  sed 's/,0.00 K,/,,/g' |
  sed 's/,+0.00 K,/,,/g' |
  sed 's/,-0.00 K,/,,/g' |
  sed 's/,0,/,,/g' |
  sed 's/,0$/,/g' |
  sort |
  tee --append diffs.csv

csv2md --pretty < diffs.csv | tee diffs.md

echo "diff:"
cat diffs.csv | tail -n+2

if cat diffs.csv | tail -n+2 | grep -v ",,,,,"; then
  DID_SIZE_CHANGE="true"
else
  DID_SIZE_CHANGE="false"
fi

echo "did size change? " $DID_SIZE_CHANGE

cat diffs.md | \
  # Align the column texts right.
  sed 's/---|/---:|/g' | \
  # Align first column left.
  sed --regexp-extended 's/(-+)\:/:\1/' | \
  # Replace `\n` so that it works properly when submitted to the GitHub API.
  sed 's/$/\\n/g' | \
  tr -d '\n' | \
  tee diffs-processed.md
COMMENT=$(cat diffs-processed.md)

if [ "$DID_SIZE_CHANGE" == "false" ]; then
  echo "No size changes observed"
  COMMENT="_No size changes were observed._"
fi

# If there is already a comment by the user `paritytech-cicd-pr` in the ink! PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
POSSIBLY_COMMENT_URL=$(curl --silent $PR_COMMENTS_URL | \
  jq -r ".[] | select(.user.login == \"paritytech-cicd-pr\") | .url" | \
  head -n1
)
echo $POSSIBLY_COMMENT_URL

VERB="POST"
if [ ! -z "$POSSIBLY_COMMENT_URL" ]; then
   VERB="PATCH";
   PR_COMMENTS_URL="$POSSIBLY_COMMENT_URL"
fi

echo $VERB
echo $PR_COMMENTS_URL

INK_MASTER_HEAD=$(curl -s "https://api.github.com/repos/paritytech/ink/commits/master" | jq -r .sha)
HEAD_IN_BRANCH=$(cd ./ink/ && git log | grep -q $INK_MASTER_HEAD; echo $?)

MASTER_AHEAD=""
if [ "$HEAD_IN_BRANCH" == "1" ]; then
  echo "ink! master is ahead"
  MASTER_AHEAD="⚠️ **The ink! \`master\` is ahead of your branch, this might skew the comparison data below.**\\n\\n"
fi

UPDATED=$(TZ='Europe/Berlin' date)
CC_VERSION=$(cargo-contract --version | egrep --only-matching "cargo-contract .*-x86" | sed -s 's/-x86//')
curl -X ${VERB} ${PR_COMMENTS_URL} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_PR_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{ \
\"body\": \"## 🦑 📈 ink! Example Contracts ‒ Changes Report 📉 🦑\\n \
${MASTER_AHEAD}These are the results when building the \`examples/*\` contracts from this branch with \`$CC_VERSION\` and comparing them to ink! \`master\`: \\n\\n\
${COMMENT}\n\n[Link to the run](https://gitlab.parity.io/parity/ink-waterfall/-/pipelines/${CI_PIPELINE_ID}) | Last update: ${UPDATED}\" \
    }"
