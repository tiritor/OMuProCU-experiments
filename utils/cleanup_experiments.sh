#!/bin/bash

killall ping
ps aux | grep "python -u /scripts/ping.py" | awk '{print $2}' | head -n -1 | xargs kill -9
ps aux | grep "iperf" | awk '{print $2}' | head -n -1 | xargs kill -9
