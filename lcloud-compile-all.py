# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "paramiko",
#     "scp",
# ]
# ///

import os
import sys
import subprocess
import json
import socket

STAGES = ['host', 'cloud', 'guest-win11', 'guest-macos']
SELF_FILE_NAME = os.path.basename(__file__) # we can safely assume this is identical across all systems and is used when building file paths to next stage

if len(sys.argv) < 2:
  print(f'''
Usage: uv run cloud-compile-all.py {"|".join(STAGES)}

This script copies itself to targets and runs stages on those machines;
 - host is your machine
 - cloud is the machine running VMS, which you have SSH access to using PKI
 - guest-* are the guest machines, which have hardcoded credentials in this script.
'''.strip())
  sys.exit(1)

stage = sys.argv[1]
if not (stage in STAGES):
  print(f'Unknown stage "{stage}", exiting')
  sys.exit(1)

####################
# Host stage data
####################
host_host_ip = '169.254.10.10'

host_cloud_ip = '169.254.100.20'
host_cloud_user = 'user'
host_cloud_key = '/j/ident/azure_sidekick'

####################
# Cloud stage data
####################
cloud_dhcp_lease_file = '/var/lib/libvirt/dnsmasq/virbr0.status'

####################
# Guest stage data
####################

####################
# Utility functions
####################

def setup_host_ip_space():
  eth_dev = os.environ.get('ETH_DEV', '')
  if len(eth_dev) < 1:
    eth_dev = subprocess.check_output(['sh', '-c', "ip a | grep ': enp' | tail -1 | cut -d':' -f2 | tr -d '[:space:]'"]).decode('utf-8').strip()
  #print(f'eth_dev = {eth_dev}')
  ip_addr_out = subprocess.check_output(['sh', '-c', 'ip address']).decode('utf-8').strip()
  if not host_host_ip.casefold() in ip_addr_out.casefold():
    subprocess.run([
      'sudo', 'ip', 'address', 'add', f'{host_host_ip}/16', 'broadcast', '+', 'dev', eth_dev
    ], check=True)



def get_ip_for_vm_hostname(vm_hostname):
    if not os.path.exists(cloud_dhcp_lease_file):
        raise FileNotFoundError(f"Lease file not found: {cloud_dhcp_lease_file}")
    with open(cloud_dhcp_lease_file, "r") as f:
        leases = json.load(f)
    for entry in leases:
        if entry.get('hostname', '').casefold() == vm_hostname.casefold():
            return entry.get('ip-address', None)
    return None


####################
# Stage Logic
####################

def host():
  setup_host_ip_space()
  user_at_host = f'{host_cloud_user}@{host_cloud_ip}'
  # Copy self to cloud
  subprocess.run([
    'scp', '-i', host_cloud_key,
      __file__,
      f'{user_at_host}:/tmp/{SELF_FILE_NAME}'
  ],check=True)
  # Execute self on cloud at stage "cloud"
  subprocess.run([
    'ssh', '-i', host_cloud_key,
      f'{user_at_host}', 'uv', 'run', f'/tmp/{SELF_FILE_NAME}', 'cloud'
  ],check=True)

def cloud():
  print(f'Running "cloud" stage on {socket.gethostname()}')


def guest_win11():
  pass

def guest_macos():
  pass


# Call the stage function
stage = stage.replace('-', '_').lower()
globals()[stage]()


