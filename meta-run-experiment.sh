#!/bin/bash

TMUX_SESSION_NAME=meta-experiment-session
TMUX_MAIN_WINDOW_NAME=main

tmux new-session -A -s $TMUX_SESSION_NAME \; rename-window $TMUX_MAIN_WINDOW_NAME \; send -t $TMUX_SESSION_NAME:$TMUX_MAIN_WINDOW_NAME "./run-experiment-helper.sh" ENTER

