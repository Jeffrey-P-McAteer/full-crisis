# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///


# The plan here is to cross-compile a PE32 32-bit .exe binary,
# then pack a dist/* dir to be loadable from a web browser using
# https://www.boxedwine.org as a runtime.
# If we need web-specific changes such as no animations for web,
# use #[cfg(target_pointer_width = "64")] or some such similar compile-time option.

import os
import shutil
import urllib.request
import zipfile
import sys
import json
import subprocess
import io

def build_www_folder(exe_path, www_folder_path):
    if not os.path.exists(exe_path):
        print(f'Error: Provided .exe file ({exe_path}) does not exist.')
        return

    www_dir = os.path.abspath(www_folder_path)
    os.makedirs(www_dir, exist_ok=True)

    with open(os.path.join(www_dir, 'index.html'), 'w') as fd:
      fd.write('''
<!-- TODO -->
''')

    print(f'Done! Open {os.path.join(www_dir, "index.html")} in a browser to run {os.path.basename(exe_path)}')

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print('Usage: python build-for-web.py ./path/to/game.exe ./path/to/www')
        sys.exit(1)

    exe_file_path = sys.argv[1]
    www_folder_path = sys.argv[2]
    build_www_folder(exe_file_path, www_folder_path)



