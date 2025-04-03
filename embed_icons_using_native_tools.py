# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///

import os
import sys
import subprocess
import shutil
import time

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
    print(f'Found Resource Hacker at {resource_hacker_exe}')

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
    print(f'resource_hacker_exes = {resource_hacker_exes}')
else:
  print(f'resource_hacker_folders = {resource_hacker_folders}')


