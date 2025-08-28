#!/bin/sh

mkdir -p logs

while true
do
    stdbuf -oL ./target/release/ZeldaServer --port ${1:-8080} ${2:--vv} 2>&1 | tee -a ./logs/${1:-8080}_serverlog_`date +%m_%d_%y_%s`.log
    echo CRASH at `date` | tee -a ./logs/lurk_crash.log
    sleep 120
done
