# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///

# This script is designed specifically for the azure-sidekick local cloud
# environment. Tweaks will be necessary to make it work on any other cloud.

import os
import sys
import subprocess
import time
import urllib.request
import json

if 'install' in sys.argv:
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
RestartSec=1
User=user
ExecStart=/usr/bin/python {__file__}
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


def get_latest_commit_sha():
  url = f"https://api.github.com/repos/Jeffrey-P-McAteer/full-crisis/commits/master"
  try:
    with urllib.request.urlopen(url) as response:
      if response.status != 200:
        raise Exception(f"GitHub API returned status {response.status}")
      data = json.load(response)
      return data["sha"]
  except urllib.error.URLError as e:
      print(f"Failed to fetch latest commit: {e}", file=sys.stderr)
      return None


if __name__ == '__main__':
  start_time = time.time()

  last_seen_commit_hash = get_latest_commit_sha()

  while True:
    runtime_s = time.time() - start_time

    if runtime_s < 60.0:
      sleep_s = 5
    elif runtime_s < 120.0:
      sleep_s = 15
    else:
      sleep_s = 60

    # Check to see if new commit exists, if so trigger build
    current_commit_hash = get_latest_commit_sha()
    print(f'current_commit_hash = {current_commit_hash} last_seen_commit_hash = {last_seen_commit_hash}')

    time.sleep(sleep_s)






