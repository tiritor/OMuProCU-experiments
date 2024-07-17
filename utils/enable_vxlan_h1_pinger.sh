#!/bin/bash

PINGER_HOST="<PINGER_HOST_IP>"
HOST1="<HOST1_IP>"

sudo ip link add vxlan0 \
    type vxlan id 42 \
    dstport 4789 \
    remote $PINGER_HOST \
    local $HOST1 \
    dev ens160
sudo ip addr add 192.168.42.1/24 dev vxlan0
sudo ip link set vxlan0 up
