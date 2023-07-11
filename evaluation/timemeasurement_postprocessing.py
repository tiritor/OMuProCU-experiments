#!/usr/bin/python3

from enum import Enum
from matplotlib import pyplot as plt

import pandas as pd

class TimeScales(Enum):
    """
    docstring
    """
    SECONDS = 10 ** 0 
    MILLISECONDS = 10 ** 3
    MICROSECONDS = 10 ** 6
    NANOSECONDS = 10 ** 9

def convert_timescale(data, current_timescale : int, new_timescale : int):
    """
    Convert data from a given timescale to another one. 
    """
    timescale = current_timescale / new_timescale if current_timescale < new_timescale else new_timescale / current_timescale
    return data * timescale

dev_init_modes = [
    0,
    1
]

protocols = [
    "tcp",
    "udp"
]

orchestrator_timemeasurement_path = "./evaluation/raw_data/"
orchestrator_timemeasurement_file = "orchestrator_timemeasurement-{}-{}.csv"
orchestrator_timemeasurement_timescale = TimeScales.NANOSECONDS

evaluation_timescale = TimeScales.MILLISECONDS

orchestrator_measured_steps = [
    # "deploymentTime",
    "validationTime",
    "scheduledTime",
    "processingTime",
    "nosPreprocessTime",
    "nosUpdateTime",
    "inPreprocessTime",
    "inCompileTime",
    "inUpdateTime",
    "inPostProcessTime",
    "tifPreprocessTime",
    "tifUpdateTime",
    "tifPostUpdateTime"
]

orchestrator_prettified_names = {
    "validationTime" : "Validation",
    "scheduledTime" : "Schedule",
    "processingTime" : "Processing",
    "nosPreprocessTime" : "NOS\n Preprocessing",
    "nosUpdateTime" : "NOS\n Update",
    "inPreprocessTime" : "INC\n Preprocess",
    "inCompileTime" : "TIF\n Compile",
    "inUpdateTime" : "INC\n Update",
    "inPostProcessTime" : "INC\n Postprocessing",
    "tifPreprocessTime" : "TIF\n Preprocessing",
    "tifUpdateTime" : "TIF\n Update",
    "tifPostUpdateTime": "TIF\n Post-Update"
}

orchestrator_measured_steps_colors = {
    "deploymentTime": "#AAAAAA",
    "validationTime": "#8604dd",
    "scheduledTime": "#AAAAAA",
    "processingTime": "#AAAAAA",
    "nosPreprocessTime": "#10b541",
    "nosUpdateTime": "#10b541",
    # "inPreprocessTime": "#0000FF",
    "inPreprocessTime": "#fcae28",
    "inCompileTime": "#0000FF",
    "inUpdateTime": "#fcae28",
    "inPostProcessTime": "#fcae28",
    "tifPreprocessTime": "#ff9000",
    "tifUpdateTime": "#b51010",
    "tifPostUpdateTime": "#b51010",
}

unnecessary_steps = [
    "tifPreprocessTime",
    "inPostProcessTime",
    "inUpdateTime",
    "nosPreprocessTime",
]

collection = {}

for mode in dev_init_modes:
    for protocol in protocols:
        avg_values = {}
        df = None
        with open(orchestrator_timemeasurement_path + orchestrator_timemeasurement_file.format(mode, protocol)) as f:
            df = pd.read_csv(f)

        # Transform nanoseconds to milliseconds
        for index in orchestrator_measured_steps:
            df[index] = convert_timescale(df[index], orchestrator_timemeasurement_timescale.value, evaluation_timescale.value)

        actions = df["action"].unique()

        for action in actions:
            df_avg_values = df.loc[df["action"] == action]
            df_avg_values.drop("tenantId", axis=1, inplace=True)
            df_avg_values.drop("submissionId", axis=1, inplace=True)

            # Remove deploymentTime (since it is not measured at the moment)
            df_avg_values.drop("deploymentTime", axis=1, inplace=True)
            df_avg_values = df_avg_values.mean()
            mean_deployment_time = df_avg_values.sum()
            print("PROTO: {}, DEV_INIT_MODE: {}, ACTION: {}".format(protocol, mode, action))
            print(df_avg_values)
            print("MEAN DEPLOYMENT TIME: " + str(mean_deployment_time) + " ms (" + str(mean_deployment_time / TimeScales.MILLISECONDS.value) + " secs)")

            tifTime = df_avg_values["tifUpdateTime"] + df_avg_values["tifPostUpdateTime"]
            if mode not in collection.keys():
                collection[mode] = {}
            if protocol in collection.keys():
                collection[mode][protocol] = {}
            collection[mode][protocol] =  {"mean_deployment": mean_deployment_time /TimeScales.MILLISECONDS.value, "tif_time_total": tifTime / TimeScales.MILLISECONDS.value, "tif_update_time": (df_avg_values["tifUpdateTime"]) / TimeScales.MILLISECONDS.value, "tif_postprocess_time": + df_avg_values["tifPostUpdateTime"] / TimeScales.MILLISECONDS.value}
            print("TIF_UPDATE_TIME (including pre-/post-processing steps): " + str(tifTime) + " ms (" + str(tifTime / TimeScales.MILLISECONDS.value) + " secs)")
            copied_orchestrator_measured_steps = orchestrator_measured_steps.copy()
            avg_values = df_avg_values[copied_orchestrator_measured_steps].to_dict()
            copied_avg_values = avg_values.copy()

            # Remove processingTime (since it is a meta duration value)
            copied_avg_values.pop("processingTime")
            copied_orchestrator_measured_steps.remove("processingTime")
            plt.barh(copied_orchestrator_measured_steps, [copied_avg_values[col] for col in copied_orchestrator_measured_steps], color=[orchestrator_measured_steps_colors[col] for col in  copied_orchestrator_measured_steps])
            plt.xticks(rotation=45)
            plt.tight_layout()
            plt.savefig("test-{}-wo-processingTime.pdf".format(action))
            plt.close()

            # Remove inCompileTime (since it is is a meta duration value)
            copied_avg_values.pop("inCompileTime")
            copied_orchestrator_measured_steps.remove("inCompileTime")
            plt.barh(copied_orchestrator_measured_steps, [copied_avg_values[col] for col in copied_orchestrator_measured_steps], color=[orchestrator_measured_steps_colors[col] for col in  copied_orchestrator_measured_steps])
            plt.xticks(rotation=45)
            plt.suptitle("Orchestrator Step Timemeasurements (avg)")
            plt.title("Without Processing and inCompile Time")
            plt.tight_layout()
            plt.savefig("test-{}-wo-pro-incompileTime.pdf".format(action))
            plt.close()

            # Remove scheduledTime (since it depends on the used infrastructure and its philosophy)
            copied_avg_values.pop("scheduledTime")
            copied_orchestrator_measured_steps.remove("scheduledTime")
            copied_avg_values["tifUpdateTime"] += copied_avg_values["tifPostUpdateTime"]
            copied_orchestrator_measured_steps.remove("tifPostUpdateTime")
            for step in unnecessary_steps: 
                copied_avg_values.pop(step)
                copied_orchestrator_measured_steps.remove(step)
            # Prettify x-axis names
            orchestrator_measured_steps_names = []
            for step in copied_orchestrator_measured_steps:
                orchestrator_measured_steps_names.append(orchestrator_prettified_names[step])
            fig, ax = plt.subplots()
            ax.bar(orchestrator_measured_steps_names, [copied_avg_values[col] for col in copied_orchestrator_measured_steps], color=[orchestrator_measured_steps_colors[col] for col in  copied_orchestrator_measured_steps])
            ax.set_ylim(0,1600)
            plt.yticks(rotation=45)
            # Set the width and height
            fig.set_figwidth(5.5)
            fig.set_figheight(3)
            ax.set_xticks(orchestrator_measured_steps_names)
            ax.set_ylabel("Time in ms", rotation=90)

            fig.set_tight_layout("rect")
            fig.savefig("test-{}-{}-{}-influenceable-steps-minimized.pdf".format(action, mode, protocol))
            plt.close()

import pprint
pprint.pprint(collection)