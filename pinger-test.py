import time
from scapy.all import *
import os
import subprocess

experiments_list = []

experiments_count = 30
iterations = 60
interval = 1 # in secs
# Specify the host to ping
host = "192.168.42.42"
host_mac = "AA:AA:AA:AA:AA:AA"
packet_sr_verbosity = 0

MOC_SHELL_EXECUTABLE = "python3 moc_shell -c"
MOC_SHELL_INITIAL_RULE_COMMAND = '"rules provider update s3 ipv4_host 10.100.0.200 101"'
MOC_SHELL_UPDATED_RULE_COMMAND = '"rules provider update s3 ipv4_host 10.100.0.200 100"'
MD_OMUPROCU_PATH = "~/working_space/md-omuprocu"
PYTHON_VENV = "source ~/working_space/md-omuprocu/.venv/bin/activate"

ssh_user = "tiritor"
ssh_host = "sde-sw2"
ssh_init_cmd = f"cd {MD_OMUPROCU_PATH} && {PYTHON_VENV} && {MOC_SHELL_EXECUTABLE} {MOC_SHELL_INITIAL_RULE_COMMAND}"
ssh_update_cmd = f"cd {MD_OMUPROCU_PATH} && {PYTHON_VENV} && {MOC_SHELL_EXECUTABLE} {MOC_SHELL_UPDATED_RULE_COMMAND}"

csv = open("ping-results.csv", "w")
header = ""
for e in range(experiments_count):
    header += f"experiment_{e+1},"
header = header[:-1]
csv.write(f"{header}\n")
csv.flush()


def ping(host):
    # Generate 40 bytes of random payload
    random_payload = os.urandom(40)
    random_timestamp = os.urandom(16) # We cannot use the timestamp field in the ICMP header, so we use the payload instead

    # Create ICMP packet
    # packet = Ether(dst="AA:AA:AA:AA:AA:AA")/IP(dst=host)/ICMP(type=8, code=0, id=0x1234, seq=1, ts_ori=current_timestamp) / random_payload
    packet = Ether(dst=host_mac)/IP(dst=host)/ICMP(type=8, code=0, id=0x1234, seq=1) / random_timestamp / random_payload

    # Send packet and capture response
    reply = srp1(packet, iface="vxlan0", timeout=1, verbose=packet_sr_verbosity)

    if reply:
        # Check if response packet include ICMP layer
        if ICMP in reply:    
                # Check if the response packet is an ICMP echo reply
                if reply[ICMP].type == 0:
                    # Calculate latency
                    latency = reply.time - packet.sent_time
                    micros = latency * 1000000
                    print(f"Latency: {micros} µs ({latency} seconds)")
                    return micros
                else:
                    print("No response")
        else:
            print("No corresponding response")
    else:
        print("No response")

print("Experiment duration: ~ {} secs ( ~ {} min) (each experiment iteration durates {} secs)".format((iterations * interval * experiments_count), (iterations * interval * experiments_count) / 60, iterations * interval))

for e in range(experiments_count):
    print(f"---Experiment {e+1}---")
    print("Initializing the rules")
    print (f"ssh {ssh_user}@{ssh_host} '{ssh_init_cmd}'")
    stdout, stderr = subprocess.Popen(f"ssh {ssh_user}@{ssh_host} '{ssh_init_cmd}'", shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).communicate()
    print(stdout.decode())
    print(stderr.decode())
    experiments_list.append([])
    for i in range(iterations):
        # Call the ping function
        latency = ping(host)
        if i == iterations/2:
            print("Updating the rules")
            print(f"ssh {ssh_user}@{ssh_host} '{ssh_update_cmd}'")
            stdout, stderr = subprocess.Popen(f"ssh {ssh_user}@{ssh_host} '{ssh_update_cmd}'", shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).communicate()
            print(stdout.decode())
            print(stderr.decode())
        experiments_list[e].append(latency)
        time.sleep(interval)
# write the results to a csv file (each ping iteration as a row in the csv file)
for i in range(iterations):
    row = ""
    for e in range(experiments_count):
        row += f"{experiments_list[e][i]}" + ","
    csv.write(row[:-1] + "\n")
#csv.write(f"{time.time()}, {latency}\n")

csv.flush()
csv.close()