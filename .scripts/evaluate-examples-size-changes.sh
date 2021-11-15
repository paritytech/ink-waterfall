#!/bin/bash

# Evaluates size changes of the ink! examples.

set -eux
echo ${TRGR_REF}
echo "$REDIS_SIZES_KEY will be compared to $REDIS_SIZES_KEY_MASTER"

# Deserialize comparison data
redis-cli -u $GITLAB_REDIS_URI --raw lrange $REDIS_SIZES_KEY 0 -1 | sort | tee $REDIS_SIZES_KEY.csv
redis-cli -u $GITLAB_REDIS_URI --raw lrange $REDIS_SIZES_KEY_MASTER 0 -1 | sort | tee $REDIS_SIZES_KEY_MASTER.csv

csv-comparator $REDIS_SIZES_KEY_MASTER.csv $REDIS_SIZES_KEY.csv | \
  sort | \
  awk -F"," '{printf "`%s`,%.2f kb,%.2f kb\n", $1, $2, $3}' | \
  sed --regexp-extended 's/^([0-9])/,+\1/g' | \
  sed --regexp-extended 's/,([0-9])/,+\1/g' | \
  tee pure-contract-size-diff.csv

# Append the original optimized size (i.e. not the delta) to the end of each line
cat $REDIS_SIZES_KEY.csv | \
  sort | uniq | \
  egrep --only-matching ', [0-9]+\.[0-9]+$' | \
  awk -F", " '{printf ",%.2f kb\n", $2}' | \
  tee total-optimized-size.csv

paste -d "" pure-contract-size-diff.csv total-optimized-size.csv | tee combined.csv

echo " ,Δ Original Size,Δ Optimized Size,Total Optimized Size" | tee contract-size-diff.csv
cat combined.csv | sed 's/+0.00 kb//g' | tee --append contract-size-diff.csv
csv2md --pretty < contract-size-diff.csv | tee contract-size-diff.md

# Replace `\n` so that it works propely when submitted to the GitHub API.
# Also align the table text right.
cat contract-size-diff.md | \
  sed 's/---|/---:|/g' | \
  sed --regexp-extended 's/(-+)\:/:\1/' | \
  sed 's/$/\\n/g' | \
  tr -d '\n' | \
  tee contract-size-diff-nl.md
COMMENT=$(cat contract-size-diff-nl.md)

# If there is already a comment by the user `paritytech-ci` in the ink! PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
COMMENT_URL=$(curl --silent https://api.github.com/repos/paritytech/ink/issues/${TRGR_REF}/comments | \
  jq -r ".[] | select(.user.login == \"paritytech-ci\") | .url" | \
  head -n1
)

echo $COMMENT_URL
VERB="PATCH"

if [ -z "$COMMENT_URL" ]; then
   VERB="POST";
   COMMENT_URL="https://api.github.com/repos/paritytech/ink/issues/${TRGR_REF}/comments";
fi
echo $VERB
echo $COMMENT_URL
UPDATED=$(TZ='Europe/Berlin' date)
curl -X "${VERB}" "${COMMENT_URL}" \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{
        \"body\": \"## 🦑 📈 Example Contracts ‒ Size Change Report 📉 🦑\\n
        ${COMMENT}\n\n[Link to the run](https://gitlab.parity.io/parity/ink-waterfall/-/pipelines/${CI_PIPELINE_ID}) | Last update: ${UPDATED}\"
    }"