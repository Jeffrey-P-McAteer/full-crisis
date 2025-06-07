# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "zstandard",
# ]
# ///

# This builds an Arch Linux operating system in a folder (defaults to ./target/docker-on-arch)
# and installs python, cargo, docker, and cross-rs on it.

import os
import sys
import subprocess
import shutil
import time
import urllib.request
import random
import html.parser
import tarfile
import io
import traceback
import threading

import zstandard

if os.getuid() != 0:
  print('Not running as root, re-executing self using sudo...')
  sys.exit(subprocess.run(['sudo', '--preserve-env', sys.executable] + sys.argv).returncode)

class ParseAllLinks(html.parser.HTMLParser):
  def __init__(self, *args, **kwargs):
    super().__init__(*args, **kwargs)
    self.all_links = []

  def handle_starttag(self, tag, attrs):
    #print(f'handle_starttag({tag}, {attrs})')
    if tag.casefold() == 'a'.casefold():
      for k,v in attrs:
        if k.casefold() == 'href'.casefold():
          self.all_links.append(v)

  def handle_endtag(self, tag):
    #print(f'handle_endtag({tag})')
    pass

  def handle_data(self, data):
    #print(f'handle_data({data})')
    pass

def http_get_str(url):
  with urllib.request.urlopen(url, timeout=9) as response:
    reply_txt = response.read()
    if not isinstance(reply_txt, str):
      reply_txt = reply_txt.decode('utf-8')

    # And there it is, our very own little quirks mode to make directory-trees easier to parse.
    reply_txt = reply_txt.replace('<hr>', '').replace('</hr>', '')

    return reply_txt

def http_get_links(url):
  parser = ParseAllLinks()
  parser.feed(http_get_str(url))
  return parser.all_links

def get_random_mirror():
  mirror_links = http_get_links('https://archlinux.org/download/')
  return random.choice([
    link for link in mirror_links if 'archlinux/iso'.casefold() in link.casefold() and ('.com'.casefold() in link.casefold() or '.net'.casefold() in link.casefold())
  ])

def empty_dir(dir_name):
  if os.path.isdir(dir_name):
    for dirent in os.listdir(dir_name):
      dirent_path = os.path.join(dir_name, dirent)
      if os.path.isdir(dirent_path):
        shutil.rmtree(dirent_path)
      else:
        os.remove(dirent_path)

def remove_root_pw_from_etc_passwd():
  etc_passwd_file = os.path.join(CONTAINER_ROOT, 'etc', 'passwd')
  with open(etc_passwd_file, 'rb') as fd:
    etc_passwd_lines = fd.read().decode('utf-8').splitlines(keepends=False)
  if len(etc_passwd_lines) > 0 and not etc_passwd_lines[0].startswith('root::'):
    root_line_tokens = etc_passwd_lines[0].split(':')
    root_line_tokens[1] = ''
    etc_passwd_lines[0] = ':'.join(root_line_tokens)

  for i in range(0, len(etc_passwd_lines)):
    if etc_passwd_lines[i].startswith('nobody') and not etc_passwd_lines[i].startswith('nobody::'):
      nobody_line_tokens = etc_passwd_lines[i].split(':')
      nobody_line_tokens[1] = ''
      etc_passwd_lines[i] = ':'.join(nobody_line_tokens)

  with open(etc_passwd_file, 'wb') as fd:
    fd.write('\n'.join(etc_passwd_lines).encode('utf-8'))

def write_initial_mirrorlist():
  commented_mirrorlist = http_get_str('https://archlinux.org/mirrorlist/?country=US&protocol=http&protocol=https&ip_version=4')
  lines = commented_mirrorlist.splitlines(keepends=False)
  for i in range(0, len(lines)):
    if lines[i].startswith('#Server') and random.choice([True, False, False, False]):
      lines[i] = lines[i].replace('#Server', 'Server')
  container_mirrorlist_file = os.path.join(CONTAINER_ROOT, 'etc', 'pacman.d', 'mirrorlist')
  with open(container_mirrorlist_file, 'wb') as fd:
    fd.write(('\n'.join(lines)+'\n').encode('utf-8'))


CONTAINER_ROOT = os.environ.get('CONTAINER_ROOT', os.path.join(os.path.dirname(__file__), 'target', 'docker-on-arch'))
print(f'CONTAINER_ROOT={CONTAINER_ROOT}')

if (not 'CONTAINER_ROOT' in os.environ or os.path.exists(os.path.dirname(CONTAINER_ROOT)) ) and not os.path.exists(CONTAINER_ROOT):
  # Something is mounted but no child folder exists
  os.makedirs(CONTAINER_ROOT, exist_ok=True)
  subprocess.run([
    'sudo', 'chown', '1000:1000', os.path.join(os.path.dirname(__file__), 'target')
  ])

# Step 1: Has the install completed?
download_extract_os_complete_flag = os.path.join(CONTAINER_ROOT, 'install-complete.txt')
if not os.path.exists(download_extract_os_complete_flag):
  for _try_num in range(0, 6):
    try:

      empty_dir(CONTAINER_ROOT)

      a_mirror_url = get_random_mirror()
      print(f'Selected mirror {a_mirror_url}')
      mirror_file_links = http_get_links(a_mirror_url)
      best_mirror_tarball = next(link for link in mirror_file_links if 'archlinux-bootstrap'.casefold() in link.casefold() and not '.sig'.casefold() in link.casefold())
      if not best_mirror_tarball.casefold().startswith('http'.casefold()):
        best_mirror_tarball = os.path.join(a_mirror_url, best_mirror_tarball)
      print(f'Downloading {best_mirror_tarball}')

      with urllib.request.urlopen(best_mirror_tarball, timeout=45) as response:
        print(f'Decompressing...')
        dctx = zstandard.ZstdDecompressor()
        tar_bio = io.BytesIO()
        oneway_reader = dctx.stream_reader(response)
        #tar_bio.write(oneway_reader.read())
        while True:
          chunk = oneway_reader.read(256 * 1024)
          if not chunk:
            break
          tar_bio.write(chunk)
          print('.', end='', flush=True)
        print('')
        tar_bio.seek(0)

        print(f'Extracting {int(tar_bio.getbuffer().nbytes / 1000000)}mb to {CONTAINER_ROOT}')
        with tarfile.open(fileobj=tar_bio, mode='r:') as tar:
          tar.extractall(path=CONTAINER_ROOT, numeric_owner=True, filter='tar')

        # If the sub-directory root.x86_64 exists, move all children up one directory
        # then delete it
        root_x86_64_folder = os.path.join(CONTAINER_ROOT, 'root.x86_64')
        if os.path.exists(root_x86_64_folder) and os.path.isdir(root_x86_64_folder):
          for dirent in os.listdir(root_x86_64_folder):
            dirent_path = os.path.join(root_x86_64_folder, dirent)
            if os.path.isdir(dirent_path):
              shutil.copytree(dirent_path, os.path.join(CONTAINER_ROOT, dirent), symlinks=True, ignore_dangling_symlinks=True)
            else:
              shutil.copy(dirent_path, os.path.join(CONTAINER_ROOT, dirent), follow_symlinks=True)

        if os.path.exists(root_x86_64_folder) and os.path.isdir(root_x86_64_folder):
          shutil.rmtree(root_x86_64_folder)

      print(f'Successfully extracted data to {CONTAINER_ROOT}')
      break
    except:
      if 'KeyboardInterrupt' in traceback.format_exc():
        raise
      traceback.print_exc()
  else:
    sys.exit(1)

  # We also edit /etc/passwd to make sure the first line begins with
  #  root::
  # Which allows us to login as root w/o password
  remove_root_pw_from_etc_passwd()

  write_initial_mirrorlist()

  with open(download_extract_os_complete_flag, 'w') as fd:
    fd.write(f'Extracted data from {best_mirror_tarball}')

def run_in_container(*cmd):
  cmd_txt = ' '.join(x for x in cmd if not x is None)
  cmd_list = [x for x in cmd if not x is None]
  print(f'>>> {cmd_txt}')
  r = subprocess.run([
    'systemd-run',
      '--machine', 'docker-on-arch',
      '--pipe', '--pty', '--quiet',
  ] + cmd_list)
  return (cmd_txt, r.returncode)

def run_in_container_silent(*cmd):
  cmd_txt = ' '.join(x for x in cmd if not x is None)
  cmd_list = [x for x in cmd if not x is None]
  # print(f'>>> {cmd_txt}')
  r = subprocess.run([
    'systemd-run',
      '--machine', 'docker-on-arch',
      '--pipe', '--pty', '--quiet',
  ] + cmd_list, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
  return (cmd_txt, r.returncode)

def die_ifn_0(cmd_and_code):
  cmd = 'UNKNOWN'
  code = cmd_and_code
  if isinstance(cmd_and_code, tuple) and len(cmd_and_code) > 0:
    cmd = cmd_and_code[0]
    code = cmd_and_code[1]
  if code != 0:
    if len(os.environ.get('HANG_ON_FAIL', '')) > 0:
      print('HANGING, use machinectl shell to login. Press ctrl+c to kill container.')
      try:
        time.sleep(9999999)
      except:
        pass
    subprocess.run(['machinectl', 'stop', 'docker-on-arch'])
    print(f'Exited because code {code} returned from the command "{cmd}"')
    sys.exit(1)

def di0(cmd_and_code):
  die_ifn_0(cmd_and_code)

def flag_passed(flag_name):
  flag_dir = os.path.join(CONTAINER_ROOT, 'flags')
  if not os.path.exists(flag_dir):
    os.makedirs(flag_dir, exist_ok=True)
  flag_file = os.path.join(flag_dir, flag_name)
  return os.path.exists(flag_file)

def pass_flag(flag_name):
  flag_dir = os.path.join(CONTAINER_ROOT, 'flags')
  if not os.path.exists(flag_dir):
    os.makedirs(flag_dir, exist_ok=True)
  flag_file = os.path.join(flag_dir, flag_name)
  with open(flag_file, 'w') as fd:
    fd.write(flag_name)

def setup_container_async():

  # every 250ms ask if container is up before continuing w/ setup commands
  while run_in_container_silent('true')[1] != 0:
    print('+', end='', flush=True)
    time.sleep(0.25)
  print()

  # Also ask if network is available
  while run_in_container_silent('ping', '-c', '1', '1.1.1.1')[1] != 0:
    print('*', end='', flush=True)
    time.sleep(0.25)
  print()

  di0(run_in_container('sysctl', '-w', 'net.ipv6.conf.all.disable_ipv6=1'))
  di0(run_in_container('sysctl', '-w', 'net.ipv6.conf.default.disable_ipv6=1'))
  di0(run_in_container('sysctl', '-w', 'net.ipv6.conf.lo.disable_ipv6=1'))
  run_in_container('bash', '-c', 'rm /etc/resolv.conf ; ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf')

  if not flag_passed('pacman-key-setup'):
    di0(run_in_container('pacman-key', '--init'))
    di0(run_in_container('pacman-key', '--populate', 'archlinux'))
    di0(run_in_container('pacman', '-Syu', '--noconfirm')) # Sync & upgrade all
    pass_flag('pacman-key-setup')
  elif random.choice([True, False, False, False, False, False, False, False, False]):
    time.sleep(1.25)
    print(f'Running regular maitenence package sync + upgrade')
    time.sleep(1.25)
    run_in_container('pacman', '-Syu', '--noconfirm') # Sync & upgrade all

  if not flag_passed('pacman-install-base-packages'):
    di0(run_in_container('pacman', '-S', '--noconfirm', 'base-devel', 'git', 'vim', 'sudo'))
    pass_flag('pacman-install-base-packages')

  # Setup 'nobody' as an admin because sure why not it already exists
  if not flag_passed('setup-nobody-user'):
    di0(run_in_container('usermod', '--shell', '/usr/bin/bash', 'nobody'))
    di0(run_in_container('usermod', '--expiredate=', 'nobody'))
    di0(run_in_container('sed', '-i', 's/nobody::65534:65534/nobody::1000:1000/g', '/etc/passwd'))
    di0(run_in_container('sed', '-i', 's/nobody:x:65534:/nobody:x:1000:/g', '/etc/group'))
    os.makedirs(os.path.join(CONTAINER_ROOT, 'home', 'nobody'), exist_ok=True)
    di0(run_in_container('chown', '-R', 'nobody:nobody', '/home/nobody'))
    di0(run_in_container('usermod', '-d', '/home/nobody', 'nobody'))
    pass_flag('setup-nobody-user')

  nobody_sudoers_file = os.path.join(CONTAINER_ROOT, 'etc', 'sudoers.d', 'nobody')
  if not os.path.exists(nobody_sudoers_file):
    with open(nobody_sudoers_file, 'w') as fd:
      fd.write('''
nobody ALL=(ALL) NOPASSWD: ALL
Defaults:nobody timestamp_timeout=9000
Defaults:nobody !tty_tickets
'''.strip())

  def run_nobody_shell(shellcode):
    if isinstance(shellcode, list):
      shellcode = ' '.join(x for x in shellcode if not (x is None))
    return run_in_container('sudo', '-i', '-u', 'nobody', 'bash', '-c', shellcode)


  if not os.path.exists(os.path.join(CONTAINER_ROOT, 'opt', 'yay')):
    di0(run_in_container('git', 'clone', 'https://aur.archlinux.org/yay.git', '/opt/yay'))

  if not flag_passed('install-yay'):
    di0(run_in_container('chown', '-R', 'nobody:nobody', '/opt/yay'))
    di0(run_nobody_shell('cd /opt/yay && makepkg -si'))
    pass_flag('install-yay')

  # yay -S <aur-pkg-name> is now available; ensure we run as sudo-nobody!
  if not flag_passed('install-docker'):
    di0(run_nobody_shell('yay -S --noconfirm docker'))
    di0(run_in_container('systemctl', 'enable', '--now', 'docker.service'))
    di0(run_nobody_shell('sudo mkdir /etc/docker'))
    di0(run_nobody_shell('''echo '{"bridge": "none","no-new-privileges": true}' | sudo tee /etc/docker/daemon.json'''))
    pass_flag('install-docker')

  if not flag_passed('install-rust'):
    di0(run_nobody_shell('yay -S --noconfirm rustup'))
    di0(run_nobody_shell('rustup default stable'))
    di0(run_nobody_shell('cargo install cross --git https://github.com/cross-rs/cross'))
    pass_flag('install-rust')

  if not flag_passed('install-mingw-w64-binutils'):
    di0(run_nobody_shell('yay -S --noconfirm mingw-w64-binutils'))
    pass_flag('install-mingw-w64-binutils')

  di0(run_nobody_shell('ls -alh /full-crisis'))
  di0(run_nobody_shell('sudo chmod a+rw /var/run/docker.sock')) # Yeah yeah docker group this, docker group that. Our security boundary is the host.

  # # # # #
  #
  #    Compilation
  #
  # # # # #

  time.sleep(1)
  print(f'Container is running, about to cross-compile for all targets')

  for host_triple in ['x86_64-pc-windows-gnu', 'i686-pc-windows-gnu', 'x86_64-unknown-linux-gnu', 'aarch64-apple-darwin']:
    di0(run_nobody_shell(f'cd /full-crisis && /home/nobody/.cargo/bin/cross build --target {host_triple}'))
    di0(run_nobody_shell(f'cd /full-crisis && /home/nobody/.cargo/bin/cross build --release --target {host_triple}'))


  # # # # #
  #
  #    Compilation
  #
  # # # # #

  time.sleep(1)
  run_in_container('shutdown', 'now')



bg_t = threading.Thread(target=setup_container_async, args=())
bg_t.start()

nspawn_env = dict(os.environ)
nspawn_env['SYSTEMD_SECCOMP'] = '0'

r = subprocess.run([
  'systemd-nspawn',
    '--boot',
    '--machine', 'docker-on-arch',
    '--bind', f'{os.path.abspath(os.path.dirname(__file__))}:/full-crisis',
    '--capability=all',
    '--network-veth',
    '--system-call-filter=@keyring bpf',
    '--system-call-filter=add_key bpf keyctl',
    '--directory', CONTAINER_ROOT,
], env=nspawn_env)

sys.exit(r.returncode)

