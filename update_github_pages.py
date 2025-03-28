
import os
import sys
import subprocess
import tempfile
import shutil

git_repo = os.path.dirname(__file__)
os.chdir(git_repo)

git_remote_origin_url = subprocess.check_output(['git', 'remote', 'get-url', 'origin']).decode('utf-8').strip()

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
    <h1>Full-Crisis</h1>
    <p>
      TODO better documentation!
      <br>
      See <a href="https://github.com/Jeffrey-P-McAteer/full-crisis">the code</a>!
    </p>
  </body>
</html>
'''.strip())

  with open(os.path.join(f'{td}', 'style.css'), 'w') as fd:
    fd.write(f'''

'''.strip())

  subprocess.run([
    'git', 'add', '-A', '.'
  ], check=True, cwd=f'{td}')

  subprocess.run([
    'git', 'commit', '-m', 'Automated update from update_github_pages.py'
  ], check=True, cwd=f'{td}')

  subprocess.run([
    'git', 'push', '-f',
  ], check=True, cwd=f'{td}')


