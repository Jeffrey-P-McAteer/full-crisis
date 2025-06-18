# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///

import os
import sys
import subprocess
import time
import datetime
import shutil

# jeffry-isms; he cloned + installed osslsigncode to /opt/osslsigncode/build
if not shutil.which('osslsigncode') and os.path.exists('/opt/osslsigncode/build') and not '/opt/osslsigncode/build' in os.environ.get('PATH', ''):
  os.environ['PATH'] = os.environ.get('PATH', '')+':/opt/osslsigncode/build'


git_repo = os.path.dirname(__file__)
os.chdir(git_repo)

build_timestamp = 'Built at '+datetime.datetime.now().strftime('%Y-%m-%d %H:%M')

rootca_folder = os.path.join(git_repo, 'rootca')
if not os.path.exists(rootca_folder):
  print(f'FATAL: {rootca_folder} does not exist, please run "uv run rootca_mgmt.py" first!')
  sys.exit(0)

rootca_priv_key_file = os.path.join(rootca_folder, 'rootca_key.key')
rootca_cert_file = os.path.join(rootca_folder, 'rootca.crt')

windows_release_exe = os.path.join(git_repo, 'target', 'x86_64-pc-windows-gnu', 'release', 'full-crisis.exe')
if os.path.exists(windows_release_exe):
  if shutil.which('osslsigncode'):
    signed_exe = windows_release_exe+'.signed.exe'
    subprocess.run([
      'osslsigncode', 'sign', '-certs', rootca_cert_file,
        '-key', rootca_priv_key_file, # '-pass', '<key-password>',
        '-n', 'Full Crisis',
        '-i', 'https://full-crisis.jmcateer.com/',
        '-in', windows_release_exe,
        '-out', signed_exe,
    ], check=True)

    # Now copy the signed binary over the original
    shutil.copyfile(signed_exe, windows_release_exe)
    os.remove(signed_exe)

  else:
    print(f'WARNING: Cannot sign {windows_release_exe} because the program "osslsigncode" is missing! Go install it if you want windows signed binaries.')
else:
  print(f'{windows_release_exe} does not exist, not signing...')



