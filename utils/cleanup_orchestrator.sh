#!/bin/bash

ps aux | grep "python3 orchestrator.py" | awk '{print $2}' | head -n -1 | xargs kill -2

echo "Killing orchestrator if needed"
sleep 15
ps aux | grep "python3 orchestrator.py" | awk '{print $2}' | head -n -1 | xargs kill -9