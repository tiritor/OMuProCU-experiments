# Orchestrator Experiments


This repository contains experiments used for the evaluation of the orchestrator pipeline step duration.

It is part of the papers [```Low Impact Tenant Code Updates on Multi-tenant Programmable Switches```]() and ```Resilient Multi-Tenant Network Programmability for Adaptive Service Placement```.

## Disclaimer

> This repositories contains code which was using proprietary hardware and software. 
> Due to their license, these code parts were removed and must be added again, or triggered manually to achieve the experiment setup used in the proposed paper!


## Prerequisites

- Root is necessary for the switch warm up.
- iPerf3 must be installed on both hosts.
- You need rust installed on the host for the self-implemented speedtest tool using an example UDP application layer.
- Also, the Experiments used proprietary hardware and software for the switch initialization and communication. Due to their license, this is not part of this repository! If you want to have the same experience these steps must be implemented again!

## Usage

### General requirements

Add the missing code parts and parameters to this repository. 

### Python requirements

For this setup, you need also a proper python environment like a virtual environment with OMuProCU-utils package installed. 

```
python3 -m venv .venv
source .venv/bin/activate
pip3 install ../OMuProCU-utils
pip3 install -r requirements.txt # primarly required for the postprocessing.
```

### Experiment for the paper [`Low Impact Tenant Code Updates on Multi-tenant Programmable Switches`](https://ieeexplore.ieee.org/abstract/document/10327866)

#### Starting the experiment

To start the experience after implementing the missing steps again, you can enter

```
./meta-run-experiment.sh
```
which will start a tmux session where experiment is done.

#### Evaluation

After a successful experiment run, the following command can be executed in the virtual environment to generate plots and evaluation files:

```
python3 evaluation/timemeasurement_postprocessing.py
```

### Experiment for the paper `Resilient Multi-Tenant Network Programmability for Adaptive Service Placement`

For the first experiment, the following steps must be done:

#### Prerequisites

- VXLAN must be configured correctly (```./enable_vxlan_h1_pinger.sh```)
- [Pinger](https://github.com/tiritor/MD-OMuProCU/blob/main/mdtdc-files/MD-TDC-Ping.yaml) must be deployed correctly

#### Starting the first experiment

To start the experiment, the following command must be executed in the virtual environment:

```
python3 pinger-test.py
```

or 

```
pinger-experiment.sh
```


which will do all steps accordingly. 

#### Evaluation of the first experiment

After a successful experiment run, the following command can be executed in the virtual environment which generates plots and evaluation files:

```
python3 evaluation/ping-results-evaluation.py
```

#### Starting the second experiment

As second experiment for this paper, the following steps must be done.

#### Prerequisites

- VXLAN must be configured correctly (```./enable_vxlan_h1_pinger.sh```)
- [Pinger](https://github.com/tiritor/MD-OMuProCU/blob/main/mdtdc-files/MD-TDC-Ping.yaml) must be deployed correctly
- Rust must be installed on the host

#### Building the speedtest tool

To build the speedtest tool, the following command must be executed:

```
cd speedtest
cargo build --release --bin UDPClient
# If you want to run the server on a host 
cargo build --release --bin UDPServer
```

#### Configuration of the speedtest tool

The speedtest tool must be configured with the correct IP addresses of the hosts. This can be done by editing the [`speedtest/client_config.yaml`](speedtest/client_config.yaml) file.
The client also supports an experiment mode, which can be enabled by setting the `experiment_mode` parameter to `true`. In this mode, the client will send a fixed number of packets to the server and measure the time it takes to send and receive the packets.
If you are running the experiment on a host using the speedtest server, the server configuration must be edited in the [`speedtest/server_config.yaml`](speedtest/server_config.yaml) file.

#### Starting the second experiment

If you need to run the server on a host, the following command must be executed to start the speedtest server:

```
cargo run --release --bin UDPServer
```

To start the experiment, the following command must be executed to start the speedtest tool:

```
cargo run --release --bin UDPClient
```

#### Evaluation of the second experiment

After a successful experiment run, the following command can be executed in the virtual environment which generates evaluation files:

```
python3 speedtest/evaluation.py
```

