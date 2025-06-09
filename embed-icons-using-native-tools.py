# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "Pillow"
# ]
# ///

import os
import sys
import subprocess
import shutil
import time
import pathlib
import plistlib
import tempfile

import PIL
import PIL.Image

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

repo_dir = os.path.dirname(__file__)
ico_file = os.path.join(repo_dir, 'icon', 'full-crisis-icon.ico')
png_file = os.path.join(repo_dir, 'icon', 'full-crisis-icon.png')

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
    print(f'[ Win ] Found Resource Hacker at {resource_hacker_exe}')

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
    print(f'[ Win ] resource_hacker_exes = {resource_hacker_exes}')
else:
  print(f'[ Win ] resource_hacker_folders = {resource_hacker_folders}')

print('[ Win ] Done!')

# Now do mac icons + assemble a .app file under target/Mac

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



if shutil.which('iconutil'):
  print(f'Found iconutil at {shutil.which("iconutil")}, building mac app...')

  mac_targets = ['x86_64-apple-darwin', 'aarch64-apple-darwin']
  for target in mac_targets:
    if not os.path.exists(os.path.join(repo_dir, 'target', target, 'release', 'full-crisis')) or len(os.environ.get('REBUILD', '')) > 0:
      subprocess.run([
        'cargo', 'build', '--release', f'--target={target}'
      ], check=True, cwd=repo_dir)

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
      print(f'Creating {dmg_file_path}')
      create_dmg_bundle(dmg_file_path, app_dir_file, background_png)

  print(f'Copy back to main machine with:')
  for target in mac_targets:
    print(f'   rsync -aP jeffrey@169.254.100.10:full-crisis/target/{target} ./target/')
  print()
  print('[ Mac ] Done!')

else:
  print('[ Mac ] Skipping because iconutil is not installed')





