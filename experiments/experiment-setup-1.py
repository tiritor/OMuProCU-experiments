
# This experiment deploys two Tenant CNFs which should be deployed in a schedule time window.

import time
import argparse

from orchestrator_utils.logger.logger import init_logger
from orchestrator_utils.orchestrator_client import OrchestratorClient


class DurationAction(argparse.Action):

    def __call__(self, parser, namespace, values, option_string=None):
        if values < 30:
            parser.error("Minimum duration for an experiment should be 30 or higher")
            # raise argparse.ArgumentError("Minimum bandwidth is 12")

        setattr(namespace, self.dest, values)

parser = argparse.ArgumentParser(description='Experiment with time window scheduling')
parser.add_argument('--duration', '-d', default=60, action=DurationAction, type=int, required= False,
                    help='duration of an experiment in seconds (default: 60 seconds) (NOTE: must be larger than 60 seconds!)')
parser.add_argument('--schedule_time_window_size', '-s', default=10, type=int, required=False,
                    help='Schedule Time Window Size in an experiment in seconds (default: 10 seconds)')
parser.add_argument('--pre_window_size', '-p', default=25, type=int, required=False,
                    help='Schedule Time Window Size in an experiment in seconds (default: 10 seconds)')

args = parser.parse_args()


ORCHESTRATOR_ADDRESS = "192.168.73.192:49055"
SCHEDULE_TIME_WINDOW_SIZE = args.schedule_time_window_size # in secs
EXPERIMENT_DURATION = args.duration # in secs
EXPERIMENT_CONFIGURATION_LOCATION = "./experiment-files"

logger = init_logger("OrchestratorClient")

logger.info("Connecting to Orchestrator Client")
client = OrchestratorClient(ORCHESTRATOR_ADDRESS)

logger.info("Restart Scheduler Loop to improve time window scheduling points.")
logger.info(client.restart_reconfig_scheduler_loop())

logger.info("Waiting some time until experiments run a moment (ca. {} secs)".format(args.pre_window_size + 5))

time.sleep(args.pre_window_size + 5)

logger.info(client.create(EXPERIMENT_CONFIGURATION_LOCATION + "/TDC.yaml"))
logger.info(client.create(EXPERIMENT_CONFIGURATION_LOCATION + "/TDC-2.yaml"))

logger.info("Waiting until experiments ({} secs) end".format(EXPERIMENT_DURATION + 5))
time.sleep(EXPERIMENT_DURATION + 5)

logger.info("Cleanup after experiment")
logger.info(client.cleanup())

logger.info("Disconnect from Client")
client.close()
