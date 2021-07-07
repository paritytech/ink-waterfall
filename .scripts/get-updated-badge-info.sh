#!/bin/bash

echo "collecting stats for badge"

echo "pipeline created at " ${CI_PIPELINE_CREATED_AT}

TS_CREATED=`date +%s -d ${CI_PIPELINE_CREATED_AT}`
echo created $TS_CREATED

TS_NOW=`date +%s`
echo now $TS_NOW

DIFF=$(( ($TS_NOW - $TS_CREATED) / 60 ))
echo diff $DIFF

echo "{\"duration\":\"$DIFF mins\"}" > badge.json
