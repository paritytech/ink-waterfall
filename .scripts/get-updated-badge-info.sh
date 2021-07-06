#!/bin/bash

echo "collecting stats for badge"

echo "pipeline created at " ${CI_PIPELINE_CREATED_AT}

date

commits=`git rev-list --all --count`
echo "{\"commits\":\"$commits\"}" > badge.json
