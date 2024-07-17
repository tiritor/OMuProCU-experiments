# Orchestrator Experiments


This repository contains experiments used for the evaluation of the orchestrator pipeline step duration.

It is part of the papers [```Low Impact Tenant Code Updates on Multi-tenant Programmable Switches```]() and ```Resilient Multi-Tenant Code Updates for Adaptive Network State Changes```.

## Disclaimer

> This repositories contains code which was using proprietary hardware and software. 
> Due to their license, these code parts were removed and must be added again, or triggered manually to achieve the experiment setup used in the proposed paper!


## Prerequisites

- Root is necessary for the switch warm up.
- iPerf3 must be installed on both hosts.
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

### Experiment for the paper `Resilient Multi-Tenant Code Updates for Adaptive Network State Changes`

#### Prerequisites

- VXLAN must be configured correctly (```./enable_vxlan_h1_pinger.sh```)

#### Starting the experiment

To start the experiment, the following command must be executed in the virtual environment:

```
python3 pinger-test.py
```

or 

```
pinger-experiment.sh
```


which will do all steps accordingly. 

#### Evaluation

After a successful experiment run, the following command can be executed in the virtual environment which generates plots and evaluation files:

```
python3 evaluation/ping-results-evaluation.py
```