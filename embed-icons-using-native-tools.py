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

def create_dmg_bundle(dmg_file_path, app_dir_file):
  with tempfile.TemporaryDirectory(suffix='full-crisis-staging') as staging_dir:
    print(f'staging_dir = {staging_dir}')


if shutil.which('iconutil'):
  print(f'Found iconutil at {shutil.which("iconutil")}, building mac app...')

  mac_targets = ['x86_64-apple-darwin', 'aarch64-apple-darwin']
  for target in mac_targets:
    if not os.path.exists(os.path.join(repo_dir, 'target', target, 'release', 'full-crisis')):
      subprocess.run([
        'cargo', 'build', '--release', f'{target}'
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
      print(f'Creating {dmg_file_path}')
      create_dmg_bundle(dmg_file_path, app_dir_file)


  print('[ Mac ] Done!')
else:
  print('[ Mac ] Skipping because iconutil is not installed')





