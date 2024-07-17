#!/bin/bash

echo Setting static ARP entries

sudo arp -s 10.100.0.200 AA:AA:AA:AA:AA:AA
sudo arp -s 192.168.42.42 AA:AA:AA:AA:AA:AA

echo "Starting pinger experiment"
sudo python3 pinger-test.py 

echo "Experiment finished"
