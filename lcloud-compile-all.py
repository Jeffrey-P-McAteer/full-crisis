# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "paramiko",
#     "scp",
#     "Pillow",
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
import tempfile
import shutil
import pathlib
import plistlib
import getpass

paramiko = None
try:
  import paramiko
except Exception as ex:
  ignorable = os.name == 'nt' or sys.platform == 'darwin'
  if not ignorable:
    raise ex

try:
  import PIL
  import PIL.Image
except:
  traceback.print_exc()

print(f'Running from {sys.executable} {sys.version}')

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

guest_compile_debug = False

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

def paramiko_stream_cmd(prefix, channel, command):
  print(f'Running command in VM: {command}')

  channel.exec_command(command)

  # Stream stdout
  while True:
      if channel.recv_ready():
          output = channel.recv(1024).decode()
          print(prefix+output, end="", flush=True)  # already has newline

      if channel.recv_stderr_ready():
          error = channel.recv_stderr(1024).decode()
          print(prefix+error, end="", flush=True)  # already has newline

      if channel.exit_status_ready():
          break

      time.sleep(0.1)  # avoid busy wait

  return channel.recv_exit_status()

def stream_output(stream, label):
  if len(label) > 0:
      for line in stream:
          print(f"{label}{line}", end="")  # line already includes newline
  else:
      for line in stream:
          print(f"{line}", end="")  # line already includes newline

def run_streaming_command(cmd, label):
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1  # Line-buffered
    )

    # Start threads to read stdout and stderr
    stdout_thread = threading.Thread(target=stream_output, args=(process.stdout, label))
    stderr_thread = threading.Thread(target=stream_output, args=(process.stderr, label))

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
        if os.path.isdir(dirent_path) and not dirent.casefold() == 'docker-on-arch'.casefold(): # I get one special-case OK? it'd be annoying to take in a list of these.
          found_files += find_name_under(dirent_path, file_name, max_recursion=max_recursion-1)
    except PermissionError:
      print(f'Skipping {dir_name} because PermissionError')

  return found_files

def create_icns(icon_png_path, output_icns_path):
    icon_iconset_path = str(output_icns_path).replace('.icns', '.iconset')
    iconset_dir = pathlib.Path(icon_iconset_path)
    iconset_dir.mkdir(exist_ok=True)

    sizes = [
        (16, False), (16, True),
        (32, False), (32, True),
        (128, False), (128, True),
        (256, False), (256, True),
        (512, False), (512, True),
    ]
    with PIL.Image.open(icon_png_path) as img:
        for size, is_2x in sizes:
            icon_size = size * (2 if is_2x else 1)
            name = f"icon_{size}x{size}{'@2x' if is_2x else ''}.png"
            resized = img.resize((icon_size, icon_size), PIL.Image.LANCZOS)
            resized.save(iconset_dir / name)

    subprocess.run([shutil.which('iconutil'), "-c", "icns", iconset_dir], check=True)
    shutil.move(os.path.join(os.path.dirname(output_icns_path), 'Icon.icns'), output_icns_path)
    shutil.rmtree(iconset_dir)

def create_info_plist(app_name, executable_name, icon_file):
    plist = {
        "CFBundleName": app_name,
        "CFBundleDisplayName": app_name,
        "CFBundleExecutable": executable_name,
        "CFBundleIdentifier": f"com.jmcateer.{app_name.lower()}",
        "CFBundleVersion": "1.0",
        "CFBundlePackageType": "APPL",
        "CFBundleSignature": "????",
        "CFBundleIconFile": icon_file,
        "LSMinimumSystemVersion": "10.10"
    }
    return plist

def build_app_bundle(folder_to_build_in, app_name, binary_path, icon_png):
    app_dir = pathlib.Path(os.path.join(folder_to_build_in, f"{app_name}.app"))
    contents = app_dir / "Contents"
    macos = contents / "MacOS"
    resources = contents / "Resources"

    if os.path.exists(app_dir):
      shutil.rmtree(app_dir)

    for path in [app_dir, contents, macos, resources]:
        path.mkdir(parents=True, exist_ok=True)

    # Copy binary
    shutil.copy(binary_path, macos / app_name)
    os.chmod(macos / app_name, 0o755)

    # Convert and copy icon
    icns_path = resources / "Icon.icns"
    create_icns(icon_png, icns_path)

    # Write Info.plist
    plist = create_info_plist(app_name, app_name, "Icon.icns")
    with open(contents / "Info.plist", "wb") as f:
        plistlib.dump(plist, f)

    print(f"{app_name}.app created successfully at {folder_to_build_in}")

    return app_dir

def get_size_recursive(start_path):
    total_size = 0
    for dirpath, dirnames, filenames in os.walk(start_path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            # skip if it is symbolic link
            if not os.path.islink(fp):
                total_size += os.path.getsize(fp)
    return total_size

def get_temp_filepath(suffix=""):
    # Create a temporary file just to get a valid name, then delete it
    with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
        temp_path = tmp.name
    os.unlink(temp_path)  # Now it's safe for a 3rd-party tool to create it
    return temp_path

def create_dmg_bundle(dmg_file_path, app_dir_file, background_png):
  if os.path.exists(dmg_file_path):
    os.remove(dmg_file_path)

  with tempfile.TemporaryDirectory(suffix='full-crisis-staging') as staging_dir:
    print(f'staging_dir = {staging_dir}')
    if not isinstance(staging_dir, pathlib.Path):
      staging_dir = pathlib.Path(staging_dir)
    shutil.copytree(app_dir_file, staging_dir / os.path.basename(app_dir_file))
    # Create symbolic link to /Applications
    applications_link = staging_dir / "Applications"
    applications_link.symlink_to("/Applications")
    (staging_dir / ".background").mkdir()
    shutil.copy(background_png, staging_dir / ".background" / "background.png")

    expected_size_mb = int(get_size_recursive(staging_dir) / 1_000_000.0) + 20
    volume_name = os.path.basename(app_dir_file).replace('.app', '')

    temp_dmg = get_temp_filepath('.dmg')
    try:
      subprocess.run([
          "hdiutil", "create",
          "-volname", volume_name,
          "-srcfolder", str(staging_dir),
          "-fs", "HFS+",
          "-fsargs", "-c c=64,a=16,e=16",
          "-format", "UDRW",
          "-size", f'{expected_size_mb}m',
          temp_dmg
      ], check=True)

      mount_result = subprocess.run(["hdiutil", "attach", temp_dmg, "-readwrite"], capture_output=True, text=True, check=True)
      device_line = next((line for line in mount_result.stdout.splitlines() if "/Volumes/" in line), None)
      volume_path = device_line.split("\t")[-1] if device_line else None

      try:
        apple_script = f'''
tell application "Finder"
    tell disk "{volume_name}"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {{100, 100, 580, 410}}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 128
        set background picture of viewOptions to file ".background:background.png"

        set position of item "{os.path.basename(app_dir_file)}" to {90, 130}
        set position of item "Applications" to {380, 130}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
'''
        subprocess.run([
            "osascript", "-e", apple_script
        ], check=True)

        ds_store_path = pathlib.Path(volume_path) / '.DS_Store'
        for _ in range(20):
          if ds_store_path.exists():
              break
          print(f'Waiting for {ds_store_path} to be created...')
          time.sleep(0.75)


      finally:
          subprocess.run(["hdiutil", "detach", volume_path], check=True)

      # Now convert to a read-only dmg file
      subprocess.run([
        "hdiutil", "convert", temp_dmg,
        "-format", "UDZO",
        "-imagekey", "zlib-level=9",
        "-o", dmg_file_path
      ], check=True)

    finally:
      if os.path.exists(temp_dmg):
        os.remove(temp_dmg)

def print_duration(begin_s, msg_f):
  duration_s = time.time() - begin_s
  minutes = int(duration_s / 60.0)
  seconds = duration_s - (minutes * 60.0)
  if minutes > 0:
    duration_string = f'{minutes}m {seconds:.1f}s'
  else:
    duration_string = f'{duration_s:.1f}s'
  print(msg_f.format(duration_string), flush=True)

def delete_target_binary(base_path, target_name):
  potentials = [
    os.path.join(base_path, 'target', target_name, 'debug', 'full-crisis.exe'),
    os.path.join(base_path, 'target', target_name, 'release', 'full-crisis.exe'),
    os.path.join(base_path, 'target', target_name, 'debug', 'full-crisis'),
    os.path.join(base_path, 'target', target_name, 'release', 'full-crisis'),
  ]
  for p in potentials:
    if os.path.exists(p):
      os.remove(p)

####################
# Stage Logic
####################

def host():
  print(f'[ host ] Running "host" stage on {socket.gethostname()}', flush=True)
  begin_s = time.time()
  setup_host_ip_space()
  user_at_host = f'{host_cloud_user}@{host_cloud_ip}'
  # Copy project directory to cloud's /mnt/nfs/shared-vm-dir, which is shared to VMs
  repo_dir = os.path.dirname(__file__).rstrip('/').rstrip('\\')
  repo_dir_name = os.path.basename(repo_dir)
  for target_dirent in os.listdir(os.path.join(repo_dir, 'target')):
    delete_target_binary(repo_dir, target_dirent)
  subprocess.run([
    'rsync',
      '-az', '--info=progress2', '-e', f'ssh -i "{host_cloud_key}"', '--exclude=target/docker-on-arch/', '--exclude=.git/', '--exclude=target/',
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
  # run_streaming_command([
  #   'ssh', '-i', host_cloud_key,
  #     f'{user_at_host}', 'uv', 'run', f'/tmp/{SELF_FILE_NAME}', 'cloud'
  # ])

  threads = []
  cloud_t = threading.Thread(target=run_streaming_command, args=([
    'ssh', '-i', host_cloud_key,
      f'{user_at_host}', 'uv', 'run', f'/tmp/{SELF_FILE_NAME}', 'cloud'
  ], '[ cloud ] ',))
  cloud_t.start()
  threads.append(cloud_t)

  host_linux_t = threading.Thread(target=host_linux, args=())
  host_linux_t.start()
  threads.append(host_linux_t)

  for t in threads:
    t.join()

  # Copy built files back to local machine
  print(f'[ host ] Copying built files back...')
  subprocess.run([
    'rsync',
      '-az', '--info=progress2', '-e', f'ssh -i "{host_cloud_key}"', '--exclude=.git/',
      f'{user_at_host}:/mnt/nfs/shared-vm-dir/{repo_dir_name}/target/.',
      f'{repo_dir}/target',
  ],check=True,bufsize=1,text=True)
  # Remove self just to be clean
  subprocess.run([
    'ssh', '-i', host_cloud_key,
      f'{user_at_host}', 'rm', f'/tmp/{SELF_FILE_NAME}'
  ],check=True,bufsize=1,text=True)
  print_duration(begin_s, '[ host ] took {}')
  # Also print timestamps of all artifacts to double-check build time; if one is old
  # that indicates build failed and we did not propogate the error across a VM
  artifact_names_checkers = [
    ('full-crisis', lambda x: os.path.isfile(x)),
    ('full-crisis.exe', lambda x: os.path.isfile(x)),
    ('Full-Crisis.app', lambda x: os.path.isdir(x)),
  ]
  for artifact_name, checker_fn in artifact_names_checkers:
    for found_path in find_name_under(os.path.join(repo_dir, 'target'), artifact_name, max_recursion=12):
      if checker_fn(found_path):
        age_s = time.time() - os.path.getmtime(found_path)
        age_m = int(age_s / 60.0)
        age_s = age_s - (age_m * 60.0)
        if age_m > 60:
          print(f'{found_path} is very old!')
        else:
          print(f'{age_m}m {age_s:.1f}s old - {found_path}')


  print(f'[ host ] Done!')

def host_linux():
  begin_s = time.time()

  rust_flags = '-C target-cpu=x86-64-v3' # Intel Haswell (2013), AMD Excavator (2015), Zen (2017+), so safe to say anything built within 2018+ has these features.
  if 'RUSTFLAGS' in os.environ:
    print(f'warning: overriding your custom $RUSTFLAGS="{os.environ.get("RUSTFLAGS", "")}" with "{rust_flags}"')
  os.environ['RUSTFLAGS'] = rust_flags

  linux_targets = ['x86_64-unknown-linux-gnu']
  linux_workdir = os.path.dirname(__file__)
  for target in linux_targets:
    delete_target_binary(linux_workdir, target)
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=linux_workdir, check=False)
    if guest_compile_debug:
      subprocess.run([
        'cargo', 'build', f'--target={target}'
      ], cwd=linux_workdir, check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=linux_workdir, check=True)
  print_duration(begin_s, '[ host-linux ] took {}')

def cloud():
  print(f'[ cloud ] Running "cloud" stage on {socket.gethostname()}', flush=True)
  begin_s = time.time()
  # Spin up the external drive early and asyncronously
  ignored_proc = subprocess.Popen([
    'ls', '/mnt/nfs'
  ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
  ignored_proc = subprocess.Popen([
    'sudo', 'cpupower', 'frequency-set', '-g', 'performance'
  ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
  ignored_proc = subprocess.Popen([
    'sudo', 'find', '/var/lib/libvirt/qemu/ram', '-name', 'pc.ram', '-print', '-exec', 'vmtouch', '-vt', '{}', ';'
  ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

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
        '[ guest-win11 ] ', channel, f'uv run \"{windows_workdir}\\lcloud-compile-all.py\" guest-win11'
      ))
      win_t.start()
      vm_threads.append(win_t)
    except:
      traceback.print_exc()

  else:
    print(f'WARNING: Builder-Win11 is not running! We are booting it with: virsh start Builder-Win11')
    subprocess.run(['sudo', 'virsh', 'start', 'Builder-Win11'], check=False)

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
        '[ guest-macos ] ', channel, f'sudo \"$HOME/mount-nfs.sh\" ; sleep 0.5 ; /usr/local/bin/uv run \"{macos_workdir}/lcloud-compile-all.py\" guest-macos'
      ))
      mac_t.start()
      vm_threads.append(mac_t)
    except:
      traceback.print_exc()
  else:
    print(f'WARNING: Builder-MacOS is not running! We are booting it with: virsh start Builder-MacOS')
    subprocess.run(['sudo', 'virsh', 'start', 'Builder-MacOS'], check=False)


  for t in vm_threads:
    t.join()

  ignored_proc = subprocess.Popen(['sudo', 'cpupower', 'frequency-set', '-g', 'powersave'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

  print_duration(begin_s, '[ cloud ] took {}')

  print(f'[ cloud ] Done!')


def guest_win11():
  print(f'[ guest-win11 ] Running "guest-win11" stage on {socket.gethostname()}', flush=True)
  # Step 1: Compile all .exe binaries
  begin_s = time.time()
  for target in ['x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc', ]: # 'i686-pc-windows-gnu', 'i686-pc-windows-msvc']:
    delete_target_binary(windows_workdir, target)
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=windows_workdir, check=False)
    if guest_compile_debug:
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

        for i in range(0, 5):
          if os.path.exists(full_crisis_exe_with_icon):
            break
          time.sleep(0.25)

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

  # TODO sign binaries!

  print_duration(begin_s, '[ guest-win11 ] took {}')
  print(f'[ guest-win11 ] Done!', flush=True)

def guest_macos():
  print(f'[ guest-macos ] Running "guest-macos" stage on {socket.gethostname()}', flush=True)
  begin_s = time.time()
  # Step 0: we re-mount the NFS share because it commonly shows OLD file contents!
  subprocess.run(['sudo', 'umount', '/Volumes/nfs'], check=False)
  subprocess.run(['sudo', 'mkdir', '-p', '/Volumes/nfs'], check=False)
  subprocess.run(['sudo', 'chown', f'{getpass.getuser()}:staff', '/Volumes/nfs'], check=False)
  subprocess.run(['sudo', os.path.join(os.environ.get('HOME', ''), 'mount-nfs.sh')], check=False)
  for _ in range(0, 10):
    if not os.path.exists(macos_workdir):
      time.sleep(0.2)

  # Step 1: Compile for all targets
  mac_targets = ['x86_64-apple-darwin', 'aarch64-apple-darwin']
  for target in mac_targets:
    delete_target_binary(macos_workdir, target)
    subprocess.run([
      'rustup', 'target', 'add', f'{target}'
    ], cwd=macos_workdir, check=False)
    if guest_compile_debug:
      subprocess.run([
        'cargo', 'build', f'--target={target}'
      ], cwd=macos_workdir, check=True)
    subprocess.run([
      'cargo', 'build', '--release', f'--target={target}'
    ], cwd=macos_workdir, check=True)

  # Step 2: Build a .app file for each target!
  if shutil.which('iconutil'):
    print(f'[ guest-macos ] Found iconutil at {shutil.which("iconutil")}, building mac .app files')
    for target in mac_targets:
      if os.path.exists(os.path.join(repo_dir, 'target', target, 'release', 'full-crisis')):
        app_dir_file = build_app_bundle(
          os.path.join(repo_dir, 'target', target, 'release'),
          'Full-Crisis',
          os.path.join(repo_dir, 'target', target, 'release', 'full-crisis'),
          png_file
        )
        # Now package app_dir_file into a .dmg file
        dmg_file_path = rreplace(str(app_dir_file), '.app', '.dmg')
        background_png = os.path.join(repo_dir, 'icon', 'mac-dmg-background-image.png')
        print(f'[ guest-macos ] Creating {dmg_file_path}')
        create_dmg_bundle(dmg_file_path, app_dir_file, background_png)
  else:
    print(f'[ guest-macos ] iconutil does not exist, cannot build .app files!')

  print_duration(begin_s, '[ guest-macos ] took {}')
  print(f'[ guest-macos ] Done!', flush=True)


# Call the stage function
stage = stage.replace('-', '_').lower()
globals()[stage]()


