#!/bin/bash

echo "collecting stats for badges"

commits=`git rev-list --all --count`
echo "{\"commits\":\"$commits\"}" > badges.json
