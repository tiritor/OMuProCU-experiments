#!/bin/bash

ITERATION=${2:-0}    
DATE=$1
DURATION=${3:-180}
INTERVAL=${4:-0.5}
CLEANUP_DELAY=15
BITRATE="100M"
PRE_WINDOW=0
STREAMS=1
SWITCH_ADDRESS="<Switch IP Address>"

sleep 3


ssh 192.168.76.226 -t nohup "iperf3 -c $DESTINATION_ADDRESS -b $BITRATE -R -P $STREAMS -J -i $INTERVAL -t $(( DURATION + PRE_WINDOW )) 2>&1 > ./iperf3-c-$DESTINATION_ADDRESS-$ITERATION-$DATE.json" </dev/null &

source .venv/bin/activate
python3 ./experiments/experiment-setup-1.py -d $DURATION -p $PRE_WINDOW

echo "Waiting $CLEANUP_DELAY secs until end"

sleep $CLEANUP_DELAY

./utils/cleanup_experiments.sh

exit 0
