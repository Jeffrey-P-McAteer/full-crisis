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
import traceback
import shutil

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
windows_workdir = 'Z:\\full-crisis'
macos_workdir = '/Volumes/nfs/shared-vm-dir/full-crisis'

repo_dir = os.path.dirname(__file__).rstrip('/').rstrip('\\')
ico_file = os.path.join(repo_dir, 'icon', 'full-crisis-icon.ico')
png_file = os.path.join(repo_dir, 'icon', 'full-crisis-icon.png')


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


def rreplace(s, old, new, occurrence=1):
  li = s.rsplit(old, occurrence)
  return new.join(li)


def find_name_under(dir_name, file_name, max_recursion=8):
  found_files = []
  if max_recursion > 0 and os.path.exists(dir_name) and os.path.isdir(dir_name):
    try:
      for dirent in os.listdir(dir_name):
        dirent_path = os.path.join(dir_name, dirent)
        if dirent.casefold() == file_name.casefold():
          found_files.append( dirent_path )
        if os.path.isdir(dirent_path):
          found_files += find_name_under(dirent_path, file_name, max_recursion=max_recursion-1)
    except PermissionError:
      print(f'Skipping {dir_name} because PermissionError')

  return found_files

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

  vm_threads = []

  win11_vm_ip = get_ip_for_vm_hostname('Builder-Win11')
  if win11_vm_ip is not None:
    try:
      print(f'[ cloud ] Running a build in Builder-Win11 at {win11_vm_ip}')
      # The windows 11 machine Z:\ drive is the same as the cloud's /mnt/nfs/shared-vm-dir, so we just remote in & run the build
      # and can be sure /mnt/nfs/shared-vm-dir/full-crisis will contain build results, no rsync needed.
      client = paramiko.SSHClient()
      client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
      client.connect(hostname=win11_vm_ip, username='jeffrey', password='Passw0rd!', timeout=10)

      transport = client.get_transport()
      channel = transport.open_session()
      #paramiko_stream_cmd(channel, f'uv run \"{windows_workdir}\\lcloud-compile-all.py\" guest-win11')
      win_t = threading.Thread(target=paramiko_stream_cmd, args=(
        channel, f'uv run \"{windows_workdir}\\lcloud-compile-all.py\" guest-win11'
      ))
      win_t.start()
      vm_threads.append(win_t)
    except:
      traceback.print_exc()

  else:
    print(f'WARNING: Builder-Win11 is not running! Run with: virsh start Builder-Win11')
  macos_vm_ip = get_ip_for_vm_hostname('Builder-MacOS')
  if macos_vm_ip is not None:
    try:
      print(f'[ cloud ] Running a build in Builder-MacOS at {macos_vm_ip}')
      client = paramiko.SSHClient()
      client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
      client.connect(hostname=macos_vm_ip, username='jeffrey', password='Passw0rd!', timeout=10)

      transport = client.get_transport()
      channel = transport.open_session()
      #paramiko_stream_cmd(channel, f'/usr/local/bin/uv run \"{macos_workdir}/lcloud-compile-all.py\" guest-macos')
      mac_t = threading.Thread(target=paramiko_stream_cmd, args=(
        channel, f'/usr/local/bin/uv run \"{macos_workdir}/lcloud-compile-all.py\" guest-macos'
      ))
      mac_t.start()
      vm_threads.append(mac_t)
    except:
      traceback.print_exc()
  else:
    print(f'WARNING: Builder-MacOS is not running! Run with: virsh start Builder-MacOS')

  for t in vm_threads:
    t.join()

  ignored_proc = subprocess.Popen(['sudo', 'cpupower', 'frequency-set', '-g', 'powersave'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

  print(f'[ cloud ] Done!')


def guest_win11():
  print(f'[ guest-win11 ] Running "guest-win11" stage on {socket.gethostname()}', flush=True)
  # Step 1: Compile all .exe binaries
  for target in ['x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc', ]: # 'i686-pc-windows-gnu', 'i686-pc-windows-msvc']:
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=windows_workdir, check=False)
    subprocess.run([
      'cargo', 'build', f'--target={target}'
    ], cwd=windows_workdir, check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=windows_workdir, check=True)

  # Step 2: Add in icons!
  resource_hacker_folders = find_name_under(r'C:\Program Files', 'Resource Hacker', max_recursion=1)
  resource_hacker_folders += find_name_under(r'C:\Program Files (x86)', 'Resource Hacker', max_recursion=1)

  # This allows an "Anywhere" install to be picked up from a configured PATH env var
  path_rh = shutil.which('ResourceHacker.exe')
  if not (path_rh is None) and os.path.exists(path_rh):
    resource_hacker_folders = [ os.path.dirname(path_rh) ] + resource_hacker_folders

  if len(resource_hacker_folders) > 0:
    resource_hacker_exes = find_name_under(resource_hacker_folders[0], 'ResourceHacker.exe')
    if len(resource_hacker_exes) > 0:
      resource_hacker_exe = resource_hacker_exes[0]
      print(f'[ guest-win11 ] Found Resource Hacker at {resource_hacker_exe}')

      full_crisis_exes = find_name_under(os.path.join(repo_dir, 'target', 'x86_64-pc-windows-gnu'), 'full-crisis.exe', max_recursion=2)
      full_crisis_exes += find_name_under(os.path.join(repo_dir, 'target', 'x86_64-pc-windows-msvc'), 'full-crisis.exe', max_recursion=2)
      full_crisis_exes = list(set(full_crisis_exes))

      for full_crisis_exe in full_crisis_exes:
        full_crisis_exe_with_icon = full_crisis_exe+'.icon.exe'
        subprocess.run([
          resource_hacker_exe,
            '-open', full_crisis_exe,
            '-save', full_crisis_exe_with_icon,
            '-action', 'addskip',
            '-res', ico_file,
            '-mask', 'ICONGROUP,MAINICON,'
        ], check=True)

        if os.path.exists(full_crisis_exe_with_icon):
          os.remove(full_crisis_exe)
          shutil.copyfile(full_crisis_exe_with_icon, full_crisis_exe)
          os.remove(full_crisis_exe_with_icon)
          print(f'Added icon {ico_file} to {full_crisis_exe}')
        else:
          print(f'WARNING: {full_crisis_exe_with_icon} does not exist!')

    else:
      print(f'[ guest-win11 ] resource_hacker_exes = {resource_hacker_exes}')
  else:
    print(f'[ guest-win11 ] resource_hacker_folders = {resource_hacker_folders}')

  print(f'[ guest-win11 ] Done!', flush=True)

def guest_macos():
  print(f'[ guest-macos ] Running "guest-macos" stage on {socket.gethostname()}', flush=True)
  for target in ['x86_64-apple-darwin', 'aarch64-apple-darwin']:
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=macos_workdir, check=False)
    subprocess.run([
      'cargo', 'build', f'--target={target}'
    ], cwd=macos_workdir, check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=macos_workdir, check=True)
  print(f'[ guest-macos ] Done!', flush=True)


# Call the stage function
stage = stage.replace('-', '_').lower()
globals()[stage]()


