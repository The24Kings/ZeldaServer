#!/bin/sh

# Function to check if a string is a valid port number
check_port() {
    # Check if it's a non-negative integer
    if ! echo "$1" | grep -qE '^[0-9]+$'; then
        return 1 # Not a number
    fi  

    # Check if it's within the valid port range (1-65535)
    if [ "$1" -lt 1 ] || [ "$1" -gt 65535 ]; then
        return 1 # Out of range
    fi

    return 0 # Valid port
}

# Function to check if the verbosity level is valid
check_verbose() {
    if [ -z "$1" ]; then
        return 0 # Default to valid if nothing is passed
    fi

    if echo "$1" | grep -qE '^-(v{1,4})$'; then
        return 0
    fi

    return 1
}

if [ -n "$1" ]; then
    if ! check_port "$1"; then
        echo "Error: '$1' is not a valid port number. Please provide a port between 1 and 65535."
        exit 1
    fi
fi

if [ -n "$2" ]; then
    if ! check_verbose "$2"; then
        echo "Error: '$2' is not a valid verbosity flag. Please use -v, -vv, -vvv, or -vvvv."
        exit 1
    fi
fi

mkdir -p logs

cd `pwd`
cargo build --release

while true
do
    stdbuf -oL ./target/release/ZeldaServer --port ${1:-8080} ${2:--vv} 2>&1 | tee -a ./logs/${1:-8080}_serverlog_`date +%m_%d_%y_%s`.log
    echo CRASH at `date` | tee -a ./logs/${1:-8080}_lurk_crash.log
    sleep 120
done
