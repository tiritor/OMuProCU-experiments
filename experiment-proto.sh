#!/bin/bash

DATE=`date +"%Y-%m-%d_%T"`
PROTO=${1:-'tcp'}
ITERATIONS=${2:-10}
DURATION=${3:-80}
INTERVAL=${4:-0.5}
ORCHESTRATOR_INIT_DEV_MODE=${5:-0}

SWITCH_ADDRESS="<Switch IP Address>"
SWITCH_WARMUP_DELAY=15

ORCHESTRATOR_TIF_APPLY_TIME=60
TMUX_SESSION_NAME=meta-experiment-session
TMUX_MAIN_WINDOW_NAME=main
TMUX_ORCHESTRATOR_WINDOW_NAME=orchestrator
TMUX_SWITCH_WINDOW_NAME=switch-software

function clean_up_timemeasurement_files () {
	# $1 --> Hostname
	echo "Cleanup time measurement files @ $1"
	ssh $1 "rm -f $HOME/time_measurement-*.csv"
}

function collect_data_from_host () {
	# $1 --> Hostname
	rsync --progress $1:"$HOME/time_measurement-*.csv" $HOME/experiments/$EXPERIMENT_DATA_DIR/
	retval=$?
	if [ $retval -eq 0 ]
	then
		clean_up_timemeasurement_files $1
	fi
}

tmux has -t $TMUX_SESSION_NAME:$TMUX_SWITCH_WINDOW_NAME
SWITCH_WINDOW=$?
if (( $SWITCH_WINDOW != 1 )); then 
    tmux kill-window -t $TMUX_SESSION_NAME:$TMUX_SWITCH_WINDOW_WINDOW_NAME
fi

tmux has -t $TMUX_SESSION_NAME:$TMUX_ORCHESTRATOR_WINDOW_NAME
ORCHESTRATOR=$?
if (( $ORCHESTRATOR != 1 )); then 
    tmux kill-window -t $TMUX_SESSION_NAME:$TMUX_ORCHESTRATOR_WINDOW_NAME
fi

### At this point, the switch software initialization was done. 
### Due to the license of the used propretary software, these scripts are removed.
### The implementation must be added again to get the original evaluation setup.
echo "Starting Switch Software @ $SWITCH_ADDRESS"
tmux new-window -n "$TMUX_SWITCH_WINDOW_NAME" 
tmux send -t $TMUX_SWITCH_WINDOW_NAME "ssh $SWITCH_ADDRESS -t './start-sal.sh'" ENTER
sleep 1
tmux split-window -t "$TMUX_SWITCH_WINDOW_NAME" -h 
sleep 1
tmux send "ssh $SWITCH_ADDRESS -t './tail-sal-logs.sh'" ENTER
tmux split-window -t "$TMUX_SWITCH_WINDOW_NAME" -h 
sleep 1
tmux send "ssh $SWITCH_ADDRESS -t './init-switch.sh'" ENTER
tmux select-layout even-vertical
tmux select-window -t "$TMUX_MAIN_WINDOW_NAME"

echo "Waiting some time ($SWITCH_WARMUP_DELAY secs) until switch is ready."
sleep $SWITCH_WARMUP_DELAY

echo "Starting Orchestrator @ $SWITCH_ADDRESS"
tmux new-window -n "$TMUX_ORCHESTRATOR_WINDOW_NAME"
sleep 1
tmux send -t "$TMUX_ORCHESTRATOR_WINDOW_NAME" "ssh $SWITCH_ADDRESS -t './orchestrator/start-orchestrator.sh $ORCHESTRATOR_INIT_DEV_MODE $PROTO'" ENTER \; select-window -t "$TMUX_MAIN_WINDOW_NAME"


echo "Waiting some time ($ORCHESTRATOR_TIF_APPLY_TIME secs) until inital TIF is applied."
sleep $ORCHESTRATOR_TIF_APPLY_TIME

echo "Experiment started"
echo "Details: \n\t Protocol:\t $PROTO\n\t Date: $DATE\n\t Iterations: $ITERATIONS\n"

for ((i = 1; i <= ITERATIONS; i++))
do
    echo "Running Experiment ID $i ($PROTO)"
    ./experiment-latency-$PROTO.sh $DATE $i $DURATION $INTERVAL
    sleep 2
done

echo "Stopping Orchestrator @ $SWITCH_ADDRESS"
ssh tiritor@$SWITCH_ADDRESS -t "ps aux | grep "python3 orchestrator.py" | awk '{print $2}' | head -n -1 | xargs kill -2"
sleep $ORCHESTRATOR_TIF_APPLY_TIME
tmux kill-window -t $TMUX_ORCHESTRATOR_WINDOW_NAME

echo "Stopping Switch Software @ $SWITCH_ADDRESS"
tmux kill-window -t $TMUX_SWITCH_WINDOW_NAME

collect_data_from_host $SWITCH_ADDRESS

exit 0
