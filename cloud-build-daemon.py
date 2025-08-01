# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "diskcache",
#   "platformdirs",
# ]
# ///

# This script is designed specifically for the azure-sidekick local cloud
# environment. Tweaks will be necessary to make it work on any other cloud.

# Misc config
CACHE_EXPIRE_S = 24 * 60 * 60
MAX_BUILD_FAILURES_PER_SHA = 4
BUILD_DIR = '/opt/fc-cloud-build-daemon-build-dir/full-crisis'
BUILD_USER_NAME = 'user'
GITUB_TOKEN_FILE = '/opt/fc-cloud-build-daemon-build-dir-github-token.txt'

import os
import sys
import subprocess
import time
import urllib.request
import json
import traceback
import threading

# Shared constants w/ lcloud-compile-app
REPO_DIR = BUILD_DIR.rstrip('/').rstrip('\\')

if 'install' in sys.argv:
  print(f'Creating {BUILD_DIR}')
  os.makedirs(BUILD_DIR, exist_ok=True)
  subprocess.run([
    'chown', f'{BUILD_USER_NAME}:{BUILD_USER_NAME}', BUILD_DIR,
  ], check=True)

  service_name = 'full-crisis-cloud-build-daemon.service'
  service_file = f'/etc/systemd/system/{service_name}'
  print(f'Installing to {service_file} - make sure {__file__} will always exist!')

  with open(service_file, 'w') as fd:
    fd.write(f'''
[Unit]
Description=Full Crisis Cloud Builder and Website Updater
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=2
User={BUILD_USER_NAME}
ExecStart=/usr/bin/uv run {__file__}
RuntimeMaxSec=900m
StandardError=journal
StandardOutput=journal
StandardInput=null
TimeoutStopSec=4

[Install]
WantedBy=multi-user.target
'''.strip())
  subprocess.run([
    'systemctl', 'enable', '--now', service_name
  ], check=True)
  sys.exit(0)

if not os.path.exists(BUILD_DIR):
  print(f'Please run "sudo python {__file__} install" first!')
  sys.exit(1)

if not os.path.exists(GITUB_TOKEN_FILE):
  print(f'Missing github token which must be placed at {GITUB_TOKEN_FILE}')
  sys.exit(1)

import diskcache
import platformdirs

# This allows us to save data across daemon restarts; we do try to
# ensure it isn't cached _forever_ though
cache = diskcache.Cache(platformdirs.user_cache_dir('full-crisis-cloud-build-daemon'))

def lcache(key, expensive_call, expire=CACHE_EXPIRE_S):
    global cache
    value = cache.get(key, None)
    if value is None:
        value = expensive_call()
    cache.set(key, value, expire=expire)
    return value

def get_latest_commit_sha():
  url = f"https://api.github.com/repos/Jeffrey-P-McAteer/full-crisis/commits/master"
  try:
    with urllib.request.urlopen(url) as response:
      if response.status != 200:
        raise Exception(f"GitHub API returned status {response.status}")
      data = json.load(response)
      return str(data["sha"]).strip().casefold()
  except:
      traceback.print_exc()
      return None

def get_sha_build_failures(sha):
  global cache
  return cache.get(f'build-fails-{sha}', 0)

def increase_sha_build_failures(sha):
  global cache
  cache.set(
    f'build-fails-{sha}',
    get_sha_build_failures(sha) + 1,
    expire=CACHE_EXPIRE_S
  )

def zero_sha_build_failures(sha):
  global cache
  cache.set(
    f'build-fails-{sha}',
    0,
    expire=CACHE_EXPIRE_S
  )


def get_sha_build_success(sha):
  global cache
  return cache.get(f'build-succcess-{sha}', False)

def set_sha_build_success(sha):
  global cache
  cache.set(
    f'build-succcess-{sha}',
    True,
    expire=CACHE_EXPIRE_S
  )


if __name__ == '__main__':
  start_time = time.time()

  with open(GITUB_TOKEN_FILE, 'r') as fd:
    github_token = fd.read().strip()

  # Explicitly tell us not to sleep after a build (it's the default, but just in case)
  os.environ['GUEST_SUSPEND_AFTER_BUILD'] = 'f'
  # Shared constant w/ lcloud-compile-all rsync command
  repo_dir_name = os.path.basename(REPO_DIR)

  last_seen_commit_hash = get_latest_commit_sha()
  while last_seen_commit_hash is None:
    print(f'Retrying get_latest_commit_sha()')
    time.sleep(1.0)
    last_seen_commit_hash = get_latest_commit_sha()

  # At start-up we perform some long-term maitenence tasks which generally only need to happen once.
  root_ca_crt = os.path.join(REPO_DIR, 'rootca', 'rootca.crt')
  if not os.path.exists(root_ca_crt):
    proc = subprocess.Popen(
        ['uv', 'run', 'rootca-mgmt.py'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True  # ensures string I/O instead of bytes
    )
    time.sleep(0.5)
    while True:
      try:
        stdout, stderr = proc.communicate('y\n')
        print(stdout)
        print(stderr)
      except:
        break
    try:
      proc.wait()
    except:
      pass

  # We also clear any memory of build failures if the whole daemon re-starts
  zero_sha_build_failures(last_seen_commit_hash)

  while True:
    runtime_s = time.time() - start_time

    if runtime_s < 60.0:
      sleep_s = 8
    elif runtime_s < 120.0:
      sleep_s = 20
    else:
      sleep_s = 120

    # Check to see if new commit exists, if so trigger build
    current_commit_hash = get_latest_commit_sha()
    if current_commit_hash is None or get_sha_build_success(current_commit_hash) or get_sha_build_failures(current_commit_hash) >= MAX_BUILD_FAILURES_PER_SHA:
      if current_commit_hash is None:
        print(f'We are offline, cannot read current_commit_hash={current_commit_hash}', flush=True)
      if get_sha_build_failures(current_commit_hash) >= MAX_BUILD_FAILURES_PER_SHA:
        print(f'Failed to build {current_commit_hash} {MAX_BUILD_FAILURES_PER_SHA} times, giving up.', flush=True)
      if get_sha_build_success(current_commit_hash):
        print(f'We have already built {current_commit_hash}.', flush=True)
      time.sleep(sleep_s)
      continue

    try:
      print(f'= = = = = = = Building {current_commit_hash} = = = = = = =')

      # Run a build, only updating last_seen_commit_hash if we finish successfully. Otherwise, try again!
      if not os.path.exists(os.path.join(BUILD_DIR, '.git')):
        subprocess.run([
          'git', 'clone', f'https://{github_token}@github.com/Jeffrey-P-McAteer/full-crisis.git', BUILD_DIR,
        ], check=True)

      subprocess.run([
        'git', 'fetch', 'origin',
      ], check=True, cwd=BUILD_DIR)

      subprocess.run([
        'git', 'reset', '--hard', 'origin/master',
      ], check=True, cwd=BUILD_DIR)

      subprocess.run(['sync'], check=False)

      # We now have the most recent changes locally, copy code to shared VM dir
      subprocess.run([
        'rsync',
          '-az', '--info=progress2', '--exclude=target/docker-on-arch/', '--exclude=.git/', '--exclude=target/',
          f'{REPO_DIR}',
          f'/mnt/nfs/shared-vm-dir/', # "/" at end will ensure /mnt/nfs/shared-vm-dir/full-crisis is created if not exists
      ], check=True)

      # Tell VMs to do builds, we use multiple threads b/c this is primarially I/O bound.

      def run_cloud_build_t():
        subprocess.run([
          'uv', 'run', 'lcloud-compile-all.py', 'cloud' # Does win + mac VMs
        ], check=True, cwd=BUILD_DIR)

      def run_host_linux_build_t():
        subprocess.run([
          'uv', 'run', 'lcloud-compile-all.py', 'host-linux' # Does non-vm linux build
        ], check=True, cwd=BUILD_DIR)

      def run_host_wasm32_build_t():
        subprocess.run([
          'uv', 'run', 'lcloud-compile-all.py', 'host-wasm32' # Does non-vm linux build
        ], check=True, cwd=BUILD_DIR)

      build_threads = [
        threading.Thread(target=run_cloud_build_t,      args=()),
        threading.Thread(target=run_host_linux_build_t, args=()),
        threading.Thread(target=run_host_wasm32_build_t, args=()),
      ]

      for t in build_threads:
        t.start()

      for t in build_threads:
        t.join()

      subprocess.run(['sync'], check=False)

      # Copy build files back "to us" (we use the same shared disk as the regular lcloud-compile-all.py use from a laptop)
      subprocess.run([
        'rsync',
          '-az', '--info=progress2', '--exclude=.git/',
          f'/mnt/nfs/shared-vm-dir/{repo_dir_name}/target/.',
          f'{BUILD_DIR}/target',
      ], check=True)

      subprocess.run(['sync'], check=False)

      subprocess.run([
        'uv', 'run', 'update-github-pages.py', 'noninteractive', # updates webpage w/ new build data
      ], check=True, cwd=BUILD_DIR)

      last_seen_commit_hash = current_commit_hash
      zero_sha_build_failures(current_commit_hash)
      set_sha_build_success(current_commit_hash)
    except:
      traceback.print_exc()
      increase_sha_build_failures(current_commit_hash)






