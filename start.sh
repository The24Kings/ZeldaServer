#!/bin/sh

mkdir -p logs

while true
do
    stdbuf -oL ./target/release/ImprovedLurk --port $1 2>&1 | tee -a ./logs/serverlog_`date +%m_%d_%y_%s`.log
    echo CRASH at `date` | tee -a ./logs/lurk_crash.log
    sleep 120
done
