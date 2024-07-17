#!/bin/bash

PROTOS=("tcp" "udp") # PROTOCOLS
DEV_INIT_MODES=("0" "1")
ITERATIONS=10 # Swap Iterations
DURATION=75 # iPerf duration
INTERVAL=0.5 # iPerf export interval

LEN_DEV_INIT_MODES=${#DEV_INIT_MODES[*]}
LEN_PROTOS=${#PROTOS[*]}
SWITCH_WARMUP_DELAY=15 # Estimated switch warmup/initialization delay in our setup
TIF_APPLY_TIME=60 # Estimated TIF deployment time in our setup
TIME_IPERF_RUN_CHECK=5 # Waiting time until iPerf is running some time before starting the submission process

echo "Starting Experiments with $ITERATIONS iterations and an additational duration of $DURATION seconds and export interval of $INTERVAL seconds."


TIME_ONE_EXPERIMENT=$(( DURATION + TIME_IPERF_RUN_CHECK + SWITCH_WARMUP_DELAY + TIF_APPLY_TIME * 2 ))
TIME_ALL_EXPERIMENTS=$(( TIME_ONE_EXPERIMENT * LEN_DEV_INIT_MODES * LEN_PROTOS * EXPERIMENT_ITERATIONS ))

echo "One experiment will ~ $TIME_ONE_EXPERIMENT secs ($(( TIME_ONE_EXPERIMENT / 60 )))"
echo "All experiments will take ~ $TIME_ALL_EXPERIMENTS secs ($(( TIME_ALL_EXPERIMENTS / 60 )) mins)"

for proto in "${PROTOS[@]}"
do
    for dev_init_mode in "${DEV_INIT_MODES[@]}"
    do
        echo "Running Experiment for DEV_INIT_MODE: $dev_init_mode, PROTOCOL: $proto"
        ./experiment-proto.sh $proto $ITERATIONS $DURATION $INTERVAL $dev_init_mode
    done
done

./utils/cleanup_orchestrator.sh

exit 0
