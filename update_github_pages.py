
import os
import sys
import subprocess
import tempfile
import shutil
import webbrowser
import time
import datetime

git_repo = os.path.dirname(__file__)
os.chdir(git_repo)

git_remote_origin_url = subprocess.check_output(['git', 'remote', 'get-url', 'origin']).decode('utf-8').strip()

open_preview = any('preview' in arg for arg in sys.argv)

build_timestamp = 'Built at '+datetime.datetime.now().strftime('%Y-%m-%d %H:%M')

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

  # Built artifacts
  shutil.copy(
    os.path.join(git_repo, 'target', 'x86_64-pc-windows-gnu', 'release', 'full-crisis.exe'),
    os.path.join(f'{td}', 'full-crisis.x86_64-pc-windows-gnu.exe')
  )
  shutil.copy(
    os.path.join(git_repo, 'target', 'x86_64-unknown-linux-gnu', 'release', 'full-crisis'),
    os.path.join(f'{td}', 'full-crisis.x86_64-unknown-linux-gnu')
  )

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
      <header></header>
      <p>
        Join historic disasters as emergency response personnel and hone your crisis-solving skills while saving the world!
      </p>
      <br/>
      <p>
        While you're there, tell the developer to get his butt in gear and make sure this project gets halfway done!
      </p>
      <br/>

      <header>Download <img src="full-crisis-icon.transparent.128.png" style="border-radius:6pt;"/> </header>
      <pre>{build_timestamp}</pre>
      <a class="dl win" href="full-crisis.x86_64-pc-windows-gnu.exe">Windows x64</a>
      <a class="dl mac" href="javascript:alert('todo cross-compile MacOS x64')">MacOS x64 (todo)</a>
      <a class="dl mac" href="javascript:alert('todo cross-compile MacOS ARM')">MacOS ARM (todo)</a>
      <a class="dl linux" href="full-crisis.x86_64-unknown-linux-gnu">Linux x64</a>
      <a class="dl noapp" href="javascript:alert('todo compile for WASM')">Play on the Web</a>

      <header>Code</header>
      <p>
        See <a href="https://github.com/Jeffrey-P-McAteer/full-crisis">the code</a>!
      </p>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      <br/>
      </p>
    </main>
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
  width: 600pt;
  position: absolute;
  top: 60pt;
  left: 0;
  right: 0;
  margin: auto;
  border-radius: 12pt;
  box-shadow: #3E124A 2pt 2pt 24pt;
  padding: 6pt 16pt;

  padding-top: 320pt;
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

@media only screen and (max-width: 620pt) {
  main {
    width: 94vw !important;
    top: 0pt !important;
    border-radius: 0pt !important;
    padding-top: 260pt !important;
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
    'git', 'commit', '-m', 'Automated update from update_github_pages.py'
  ], check=True, cwd=f'{td}')

  subprocess.run([
    'git', 'push', '-f',
  ], check=True, cwd=f'{td}')


