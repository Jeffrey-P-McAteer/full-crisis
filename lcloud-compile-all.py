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
import time
import threading

paramiko = None
try:
  import paramiko
except Exception as ex:
  ignorable = os.name == 'nt' or sys.platform == 'darwin'
  if not ignorable:
    raise ex

STAGES = ['host', 'cloud', 'guest-win11', 'guest-macos']
SELF_FILE_NAME = os.path.basename(__file__) # we can safely assume this is identical across all systems and is used when building file paths to next stage

stage = None
if len(sys.argv) > 1:
  stage = sys.argv[1]
elif socket.gethostname().casefold() == 'azure-angel'.casefold():
  stage = 'host' # it's jeff's laptop

if stage is None:
  print(f'''
Usage: uv run cloud-compile-all.py {"|".join(STAGES)}

This script copies itself to targets and runs stages on those machines;
 - host is your machine
 - cloud is the machine running VMS, which you have SSH access to using PKI
 - guest-* are the guest machines, which have hardcoded credentials in this script.
'''.strip())
  sys.exit(1)

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

def paramiko_stream_cmd(channel, command):
  print(f'Running command in VM: {command}')

  channel.exec_command(command)

  # Stream stdout
  while True:
      if channel.recv_ready():
          output = channel.recv(1024).decode()
          print(output, end="", flush=True)  # already has newline

      if channel.recv_stderr_ready():
          error = channel.recv_stderr(1024).decode()
          print(error, end="", flush=True)  # already has newline

      if channel.exit_status_ready():
          break

      time.sleep(0.1)  # avoid busy wait

  return channel.recv_exit_status()

def stream_output(stream, label):
  if len(label) > 0:
      for line in stream:
          print(f"[{label}] {line}", end="")  # line already includes newline
  else:
      for line in stream:
          print(f"{line}", end="")  # line already includes newline

def run_streaming_command(cmd):
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1  # Line-buffered
    )

    # Start threads to read stdout and stderr
    stdout_thread = threading.Thread(target=stream_output, args=(process.stdout, ""))
    stderr_thread = threading.Thread(target=stream_output, args=(process.stderr, ""))

    stdout_thread.start()
    stderr_thread.start()

    # Wait for both threads to finish
    stdout_thread.join()
    stderr_thread.join()

    process.wait()
    return process.returncode

####################
# Stage Logic
####################

def host():
  print(f'[ host ] Running "host" stage on {socket.gethostname()}', flush=True)
  setup_host_ip_space()
  user_at_host = f'{host_cloud_user}@{host_cloud_ip}'
  # Copy project directory to cloud's /mnt/nfs/shared-vm-dir, which is shared to VMs
  repo_dir = os.path.dirname(__file__).rstrip('/').rstrip('\\')
  repo_dir_name = os.path.basename(repo_dir)
  subprocess.run([
    'rsync',
      '-az', '--info=progress2', '-e', f'ssh -i "{host_cloud_key}"', '--exclude=target/docker-on-arch/', '--exclude=.git/',
      f'{repo_dir}',
      f'{user_at_host}:/mnt/nfs/shared-vm-dir/', # "/" at end will ensure /mnt/nfs/shared-vm-dir/full-crisis is created if not exists
  ],check=True)
  # Copy self to cloud
  subprocess.run([
    'scp', '-i', host_cloud_key,
      __file__,
      f'{user_at_host}:/tmp/{SELF_FILE_NAME}'
  ],check=True,bufsize=1,text=True)

  # Execute self on cloud at stage "cloud"
  run_streaming_command([
    'ssh', '-i', host_cloud_key,
      f'{user_at_host}', 'uv', 'run', f'/tmp/{SELF_FILE_NAME}', 'cloud'
  ])

  # Copy built files back to local machine
  print(f'[ host ] Copying built files back...')
  subprocess.run([
    'rsync',
      '-az', '--info=progress2', '-e', f'ssh -i "{host_cloud_key}"', '--exclude=.git/',
      f'{user_at_host}:/mnt/nfs/shared-vm-dir/{repo_dir_name}/.',
      f'{repo_dir}',
  ],check=True,bufsize=1,text=True)
  # Remove self just to be clean
  subprocess.run([
    'ssh', '-i', host_cloud_key,
      f'{user_at_host}', 'rm', f'/tmp/{SELF_FILE_NAME}'
  ],check=True,bufsize=1,text=True)
  print(f'[ host ] Done!')

def cloud():
  print(f'[ cloud ] Running "cloud" stage on {socket.gethostname()}', flush=True)
  # Spin up the external drive early and asyncronously
  ignored_proc = subprocess.Popen(['ls', '/mnt/nfs'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
  ignored_proc = subprocess.Popen(['sudo', 'cpupower', 'frequency-set', '-g', 'performance'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

  win11_vm_ip = get_ip_for_vm_hostname('Builder-Win11')
  if win11_vm_ip is not None:
    print(f'[ cloud ] Running a build in Builder-Win11 at {win11_vm_ip}')
    # The windows 11 machine Z:\ drive is the same as the cloud's /mnt/nfs/shared-vm-dir, so we just remote in & run the build
    # and can be sure /mnt/nfs/shared-vm-dir/full-crisis will contain build results, no rsync needed.
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(hostname=win11_vm_ip, username='jeffrey', password='Passw0rd!', timeout=10)

    transport = client.get_transport()
    channel = transport.open_session()
    paramiko_stream_cmd(channel, f'uv run Z:\\full-crisis\\lcloud-compile-all.py guest-win11')

  else:
    print(f'WARNING: Builder-Win11 is not running! Run with: virsh start Builder-Win11')
  macos_vm_ip = get_ip_for_vm_hostname('Builder-MacOS')
  if macos_vm_ip is not None:
    print(f'[ cloud ] Running a build in Builder-MacOS at {macos_vm_ip}')
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(hostname=macos_vm_ip, username='jeffrey', password='Passw0rd!', timeout=10)

    transport = client.get_transport()
    channel = transport.open_session()
    paramiko_stream_cmd(channel, f'uv run /Volumes/nfs/full-crisis/lcloud-compile-all.py guest-macos')
    # ^^ TODO update that command for the osx location of the nfs share

  else:
    print(f'WARNING: Builder-MacOS is not running! Run with: virsh start Builder-MacOS')
  print(f'[ cloud ] Done!')


def guest_win11():
  print(f'[ guest-win11 ] Running "guest-win11" stage on {socket.gethostname()}', flush=True)
  for target in ['x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc', ]: # 'i686-pc-windows-gnu', 'i686-pc-windows-msvc']:
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=f'Z:\\full-crisis', check=False)
    subprocess.run([
      'cargo', 'build', f'--target={target}'
    ], cwd=f'Z:\\full-crisis', check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=f'Z:\\full-crisis', check=True)
  print(f'[ guest-win11 ] Done!', flush=True)

def guest_macos():
  print(f'[ guest-macos ] Running "guest-macos" stage on {socket.gethostname()}', flush=True)
  for target in ['x86_64-apple-darwin', 'aarch64-apple-darwin']:
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=f'Z:\\full-crisis', check=False)
    subprocess.run([
      'cargo', 'build', f'--target={target}'
    ], cwd=f'Z:\\full-crisis', check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=f'Z:\\full-crisis', check=True)
  print(f'[ guest-macos ] Done!', flush=True)


# Call the stage function
stage = stage.replace('-', '_').lower()
globals()[stage]()


