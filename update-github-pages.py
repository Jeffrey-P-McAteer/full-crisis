# /// script
# requires-python = ">=3.12"
# dependencies = [
#
# ]
# ///

import os
import sys
import subprocess
import tempfile
import shutil
import webbrowser
import time
import datetime
import getpass
import socket
import tomllib
import zlib
import textwrap

# AI-generated utility
def get_last_commit_sha_message(repo_path="."):
    git_dir = os.path.join(repo_path, ".git")

    # Read the HEAD file to find the current branch
    with open(os.path.join(git_dir, "HEAD"), "r") as f:
        ref_line = f.readline().strip()

    if ref_line.startswith("ref: "):
        ref_path = os.path.join(git_dir, ref_line[5:])
        with open(ref_path, "r") as f:
            commit_hash = f.readline().strip()
    else:
        # Detached HEAD (ref_line contains the commit hash directly)
        commit_hash = ref_line

    # Get the object file for the commit
    obj_path = os.path.join(git_dir, "objects", commit_hash[:2], commit_hash[2:])
    with open(obj_path, "rb") as f:
        compressed_data = f.read()

    decompressed_data = zlib.decompress(compressed_data)

    # Convert to string
    commit_data = decompressed_data.decode()

    # Find the commit message (starts after two newlines)
    message_index = commit_data.find("\n\n")
    if message_index != -1:
        return (commit_hash, commit_data[message_index+2:].strip())
    else:
        return (commit_hash, "(No commit message found)")

def wrap_text(text, width=76):
    return textwrap.fill(text, width=width, break_long_words=False, break_on_hyphens=False)


git_repo = os.path.dirname(__file__)
os.chdir(git_repo)

last_commit_sha, last_commit_msg = get_last_commit_sha_message(git_repo)

git_remote_origin_url = subprocess.check_output(['git', 'remote', 'get-url', 'origin']).decode('utf-8').strip()

open_preview = any('preview' in arg for arg in sys.argv)
noninteractive = any('noninteractive' in arg for arg in sys.argv)

version = '0.0.0'
if os.path.exists('Cargo.toml'):
  with open('Cargo.toml', 'rb') as fd:
      data = tomllib.load(fd)
      version = data["package"]["version"]

# Printed above download links in monospace
build_message = ' '.join([
  'Version', version, 'built at', datetime.datetime.now().strftime('%Y-%m-%d %H:%M'),
  'by', getpass.getuser(),
  'on', socket.gethostname(),

  '\nfrom commit', last_commit_sha, 'with the message:',

  '\n'+wrap_text(last_commit_msg)
])

with tempfile.TemporaryDirectory(prefix='full-crisis-github-pages') as td:
  print(f'Building pages for {git_repo} at {td}')

  subprocess.run([
    'git', 'clone', '-b', 'github-pages-content', git_remote_origin_url, f'{td}'
  ], check=True)

  oldest_commit_hash = subprocess.check_output(['git', 'rev-list', '--max-parents=0', 'HEAD'], cwd=f'{td}').decode('utf-8').strip()

  # Reset all commits + delete files other than `.git*`
  subprocess.run([
    'git', 'reset', '--hard', f'{oldest_commit_hash}'
  ], check=True, cwd=f'{td}')

  for dirent in os.listdir(f'{td}'):
    if dirent.startswith('.git'):
      continue
    dirent_path = os.path.join(f'{td}', dirent)
    if os.path.isdir(dirent_path):
      shutil.rmtree(dirent_path)
    else:
      os.remove(dirent_path)

  # Begin logic to build pages!
  shutil.copy(
    os.path.join(git_repo, 'icon', 'full-crisis-icon.transparent.128.png'),
    os.path.join(f'{td}', 'full-crisis-icon.transparent.128.png')
  )

  shutil.copy(
    os.path.join(git_repo, 'icon', 'full-crisis-splash.transparent.png'),
    os.path.join(f'{td}', 'full-crisis-splash.transparent.png')
  )

  shutil.copy(
    os.path.join(git_repo, 'images-thirdparty', 'windows_icon.png'),
    os.path.join(f'{td}', 'windows_icon.png')
  )

  shutil.copy(
    os.path.join(git_repo, 'images-thirdparty', 'linux_icon.png'),
    os.path.join(f'{td}', 'linux_icon.png')
  )

  shutil.copy(
    os.path.join(git_repo, 'images-thirdparty', 'macos_icon.png'),
    os.path.join(f'{td}', 'macos_icon.png')
  )

  shutil.copy(
    os.path.join(git_repo, 'images-thirdparty', 'web_globe_icon.png'),
    os.path.join(f'{td}', 'web_globe_icon.png')
  )

  root_ca_crt = os.path.join(git_repo, 'rootca', 'rootca.crt')
  if not os.path.exists(root_ca_crt):
    raise Exception(f'Please create {root_ca_crt} first! Run "uv run rootca-mgmt.py"')

  shutil.copy(
    root_ca_crt,
    os.path.join(f'{td}', 'rootca.crt')
  )

  # Built artifacts
  shutil.copy(
    os.path.join(git_repo, 'target', 'x86_64-pc-windows-gnu', 'release', 'full-crisis.exe'),
    os.path.join(f'{td}', 'full-crisis.x86_64-pc-windows-gnu.exe')
  )
  shutil.copy(
    os.path.join(git_repo, 'target', 'i686-pc-windows-gnu', 'release', 'full-crisis.exe'),
    os.path.join(f'{td}', 'full-crisis.i686-pc-windows-gnu.exe')
  )
  shutil.copy(
    os.path.join(git_repo, 'target', 'x86_64-apple-darwin', 'release', 'Full-Crisis.dmg'),
    os.path.join(f'{td}', 'Full-Crisis.x86_64-apple-darwin.dmg')
  )
  shutil.copy(
    os.path.join(git_repo, 'target', 'aarch64-apple-darwin', 'release', 'Full-Crisis.dmg'),
    os.path.join(f'{td}', 'Full-Crisis.aarch64-apple-darwin.dmg')
  )
  shutil.copy(
    os.path.join(git_repo, 'target', 'x86_64-unknown-linux-gnu', 'release', 'full-crisis'),
    os.path.join(f'{td}', 'full-crisis.x86_64-unknown-linux-gnu')
  )

  # Write files like f'{td}/loc-graph.png' for inclusion in HTML below
  subprocess.run([
    'uv', 'run', os.path.join(git_repo, 'record-kpis.py'), f'{td}'
  ], check=True)

  with open(os.path.join(f'{td}', 'index.html'), 'w') as fd:
    fd.write(f'''
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <title>Full-Crisis</title>
    <link rel="stylesheet" href="style.css">
  </head>
  <body>
    <main>
      <br/>
      <p>
        Join historic disasters as emergency response personnel and hone your crisis-solving skills while saving the world!
      </p>
      <br/>
      <p>
        While you're there, tell the developer to get his butt in gear and make sure this project gets halfway done!
      </p>
      <br/>

      <header>Download <img src="full-crisis-icon.transparent.128.png" style="border-radius:6pt;"/> </header>
      <pre id="build-message">{build_message}</pre>
      <a class="dl win mobile-light-bordered-text" href="full-crisis.x86_64-pc-windows-gnu.exe">Windows x64</a>
      <a class="dl win mobile-light-bordered-text" href="full-crisis.i686-pc-windows-gnu.exe">Windows i686</a>
      <a class="dl mac mobile-light-bordered-text" href="Full-Crisis.x86_64-apple-darwin.dmg">MacOS x64</a>
      <a class="dl mac mobile-light-bordered-text" href="Full-Crisis.aarch64-apple-darwin.dmg">MacOS ARM</a>
      <a class="dl linux mobile-light-bordered-text" href="full-crisis.x86_64-unknown-linux-gnu">Linux x64</a>
      <!--<a class="dl noapp mobile-light-bordered-text" href="javascript:alert('todo compile for WASM')">Play on the Web</a>-->

      <header>Code</header>
      <p>
        See <a href="https://github.com/Jeffrey-P-McAteer/full-crisis">the code</a>!
      </p>
      <br/>
      <br/>
      <header>Metrics</header>
      <a href="loc-graph.png"><img class="kpi-chart" src="loc-graph.png"></a>
      <br/>
      <br/>
      <br/>
      <header>Development RootCA</header>
      <p>
        During development windows x64 binaries will be signed using a self-signed certificate;
        in order to trust this system you will have to manually install this certificate as a Root Certificate Authority
        on your machine.
        <br/>
        <span class="warning">THIS IS AN INSECURE DECISION!</span>
        <br/>
        The development Root Certificate Authority may be <a href="rootca.crt">downloaded here</a>.
      </p>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      </p>
    </main>
    <script>
/* Half-AI-Authored utility which scales the build message so it remains on screen when viewed with a phone */
const pre = document.getElementById('build-message');
const wrapper = pre.parentElement;
const container = wrapper.parentElement;

function scalePre() {{
  // Reset scale to 1 to get actual natural width
  pre.style.transform = "scale(1)";

  // Measure full natural width of the <pre>
  const contentWidth = pre.scrollWidth;
  const containerWidth = container.clientWidth - 22;

  // Calculate scale factor
  const scale = Math.min(1, containerWidth / contentWidth);

  // Apply scale
  pre.style.transform = `scale(${{scale}})`;

}}

window.addEventListener('load', scalePre);
window.addEventListener('resize', scalePre);
    </script>
  </body>
</html>
'''.strip())

  with open(os.path.join(f'{td}', 'style.css'), 'w') as fd:
    fd.write('''
/* Credit to MaisieMartin's wonderful selection of hues from https://www.colourlovers.com/palette/269490/This_is_an_Emergency */
html, body {
  background: #7f8e9e;
}
header {
  font: bold 2.4em Arial, sans-serif;
  color: #61B09B;
  -webkit-text-stroke-width: 1.5pt;
  -webkit-text-stroke-color: #3E124A;
}
main {
  font-family: Arial, sans-serif;
  font-size: 1.8em;
  color: #3E124A;
  width: 740pt;
  position: absolute;
  top: 60pt;
  left: 0;
  right: 0;
  margin: auto;
  border-radius: 12pt;
  box-shadow: #3E124A 2pt 2pt 24pt;
  padding: 6pt 16pt;

  padding-top: 440pt;
  background-image:    url(full-crisis-splash.transparent.png), linear-gradient(180deg, rgba(255,106,0,1) 0%, rgba(255,106,0,1) 22%, rgba(245,244,220,1) 66%, rgba(245,244,220,1) 100%);
  background-size:     contain;
  background-repeat:   no-repeat;
  background-position: center top;
}

a.dl {
  color: inherit;
  text-decoration: inherit;
  font-size: 1.12em;
  padding: 28pt 12pt;
  padding-left: 94pt;
  margin: 8pt 16pt;
  border: 3pt solid #4D4B17;
  background-color: #4D4B17;
  border-radius: 8pt;
  display: inline-block;
  background-repeat: no-repeat;
  background-position: left;
  background-size: contain;
  background-origin: padding-box;
  transition: transform .2s;
}
a.dl:hover {
  transform: scale(1.06);
  border-color: #3E124A;
}
a.win {
  background-image: url("windows_icon.png");
  background-color: #4687b0;
  border-color: #4687b0;
}
a.mac {
  background-image: url("macos_icon.png");
  background-color: #ff9b9b;
  border-color: #ff9b9b;
}
a.linux {
  background-image: url("linux_icon.png");
  background-color: #8D8B57;
  border-color: #8D8B57;
}
a.noapp {
  background-image: url("web_globe_icon.png");
  background-color: #ffa700;
  border-color: #ffa700;
}

img.kpi-chart {
  width: 300pt;
  padding: 0;
  display: inline-block;
}
img.kpi-chart:hover {
  transform: scale(1.06);
}

pre#build-message {
  transform-origin: left top;
}

.warning {
  /* colors from https://www.colourlovers.com/palette/2727/Hurricane_Warning! */
  font-weight: bold;
  color: #FF3030;
  -webkit-text-stroke-width: 1pt;
  -webkit-text-stroke-color: #680000;
  padding: 1pt 8pt;
  border-radius: 2pt;
  border: 1pt solid black;
  background: #FFFF7F
}

.background-bordered-text {
  -webkit-text-stroke-width: 4pt;
  -webkit-text-stroke-color: rgba(255,106,0,1); /* match top of gradient <main> gradient */
  paint-order: stroke fill;
}

@media only screen and (max-width: 630pt) {
  main {
    width: 94vw !important;
    top: 0pt !important;
    border-radius: 0pt !important;
    padding: 2pt 4pt !important;
    padding-top: 240pt !important;
  }
  .mobile-light-bordered-text {
    -webkit-text-stroke-width: 1pt;
    -webkit-text-stroke-color: #E0E0E0;
    paint-order: stroke fill;
  }
}
'''.strip())


  if open_preview:
    print('open_preview set, opening browser and waiting for ctrl+c...')
    webbrowser.open(os.path.join(f'{td}', 'index.html'))
    while not 'y'.casefold() in input('continue?').casefold():
      time.sleep(0.25)

    sys.exit(1)

  # We also need to tell Microsoft some bookkeeping
  with open(os.path.join(f'{td}', 'CNAME'), 'w') as fd:
    fd.write('full-crisis.jmcateer.com')

  subprocess.run([
    'git', 'add', '-A', '.'
  ], check=True, cwd=f'{td}')

  subprocess.run([
    'git', 'commit', '-m', 'Automated update from update-github-pages.py'
  ], check=True, cwd=f'{td}')

  subprocess.run([
    'git', 'push', '-f',
  ], check=True, cwd=f'{td}')


