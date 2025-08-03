# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "tomlkit",
# ]
# ///

import os
import sys
import tomllib
import subprocess

import tomlkit

repo_dir = os.path.dirname(__file__)

with open(os.path.join(repo_dir, 'Cargo.toml'), 'rb') as fd:
  workspace = tomllib.load(fd)

if len(sys.argv) > 1:
  # Print version data prior to update
  subprocess.run([sys.executable, __file__])

  version = sys.argv[1].casefold()
  if len(version.split('.')) != 3:
    print(f'Error, version {version} does not follow semver!')
    sys.exit(1)

  for member in workspace.get('workspace', dict()).get('members', list()):
    with open(os.path.join(repo_dir, member, 'Cargo.toml'), 'r', encoding='utf-8') as fd:
      toml_data = tomlkit.parse(fd.read())

    print(f'Applying version {version} to {member}')
    toml_data['package']['version'] = version

    with open(os.path.join(repo_dir, member, 'Cargo.toml'), 'w', encoding='utf-8') as fd:
      fd.write(tomlkit.dumps(toml_data))

else:
  for member in workspace.get('workspace', dict()).get('members', list()):
    version = '0.0.0'
    if os.path.exists(os.path.join(repo_dir, member, 'Cargo.toml')):
      with open(os.path.join(repo_dir, member, 'Cargo.toml'), 'rb') as fd:
          data = tomllib.load(fd)
          version = data["package"]["version"]

    print(f'{member} - v{version}')

