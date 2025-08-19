# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "matplotlib",
#     "python-dateutil",
# ]
# ///

import os
import sys
import subprocess
import time
import datetime
import importlib
import traceback
import shutil
import tempfile
import json
import getpass
import socket
import ftplib
import webbrowser
import re
import threading
import collections
import random

import dateutil
import matplotlib
import matplotlib.pyplot
import matplotlib.dates

def read_json(filename):
  try:
    with open(filename, 'r') as fd:
      return json.load(fd)
  except:
    traceback.print_exc()
    return {}

def save_json(filename, data):
  with open(filename, 'w') as fd:
    json.dump(data, fd)

def linecount_file(file_path):
  with open(file_path, 'r') as fd:
    return len([x for x in fd.read().splitlines() if len(x.strip()) > 1])

def count_sloc_for_languages(folder):
  languages_sloc = collections.defaultdict(int)

  for dirent in os.listdir(folder):
    if dirent.casefold() == 'target'.casefold() or dirent.casefold() == 'playable-crises'.casefold():
      continue # Ignore this/these

    dirent_path = os.path.join(folder, dirent)
    if os.path.isdir(dirent_path):
      for k,v in count_sloc_for_languages(dirent_path).items():
        languages_sloc[k] += v
    else:
      if dirent_path.casefold().endswith('.rs'.casefold()):
        languages_sloc['rust'] += linecount_file(dirent_path)
      elif dirent_path.casefold().endswith('.py'.casefold()):
        languages_sloc['python'] += linecount_file(dirent_path)
      elif dirent_path.casefold().endswith('.toml'.casefold()):
        languages_sloc['toml'] += linecount_file(dirent_path)

  return languages_sloc

def count_sloc_for_folder(folder, extensions):
  folder_sloc = 0
  for dirent in os.listdir(folder):
    dirent_path = os.path.join(folder, dirent)
    if os.path.isdir(dirent_path):
      folder_sloc += count_sloc_for_folder(dirent_path, extensions)
    else:
      if any( dirent_path.casefold().endswith(ext.casefold()) for ext in extensions ):
        folder_sloc += linecount_file(dirent_path)
  return folder_sloc


repo_dir = os.path.dirname(__file__)
kpis_json_file = os.path.join(repo_dir, 'kpis.json')
repo_kpis = read_json(kpis_json_file)
build_timestamp = datetime.datetime.now().strftime('%Y-%m-%d %H:%M')

if not 'sloc' in repo_kpis:
  repo_kpis['sloc'] = dict()

WRITING_KPI_GRAPHS = len(sys.argv) > 1

# If we're either NOT writing a graph, no data exists, OR the data that exists is >2 days old generate new KPI numbers _anyway_
if not WRITING_KPI_GRAPHS or not os.path.exists(kpis_json_file) or abs(time.time() - os.path.getmtime(kpis_json_file)) > 2 * 24 * 60 * 60:
  # Just record data
  print(f'Recording new data to {kpis_json_file}')

  current_sloc = count_sloc_for_languages(repo_dir)
  current_sloc['story-lines'] = count_sloc_for_folder(
    os.path.join(repo_dir, 'playable-crises'),
    ['.toml', '.txt']
  )

  repo_kpis['sloc'][build_timestamp] = current_sloc

  save_json(kpis_json_file, repo_kpis)


if WRITING_KPI_GRAPHS:
  out_dir = sys.argv[1]
  print(f'Writing KPI graphs to {out_dir}')
  if not os.path.exists(out_dir):
    os.makedirs(out_dir, exist_ok=True)

  loc_graph_file = os.path.join(out_dir, 'loc-graph.png')

  sloc_x = [ dateutil.parser.parse(k) for k in repo_kpis['sloc'].keys() ]
  sloc_y_total = [ sum(repo_kpis['sloc'][k].values()) for k in repo_kpis['sloc'].keys()]

  all_languages = set()
  for k,v in repo_kpis['sloc'].items():
    for lang_name in v.keys():
      all_languages.add(lang_name)

  sloc_languages_y_dict = dict()
  for language in all_languages:
    sloc_languages_y_dict[language] = [ repo_kpis['sloc'][k].get(language, 0) for k in repo_kpis['sloc'].keys()]

  lang_colors = [
    '#1474FA', '#F89437', '#FD0A54', '#A8D13B', '#014152', '#219C85'
  ]
  random.shuffle(lang_colors)
  lang_colors_i = 0

  #kpi_wh = (1920, 1200)

  ss = 0.75 # Size scale
  kpi_wh = (1920 * ss, 1080 * ss)
  kpi_wh = (int(kpi_wh[0]), int(kpi_wh[1]))

  fig, ax = matplotlib.pyplot.subplots(sharey=True)
  fig.set_size_inches((kpi_wh[0] / fig.get_dpi(), kpi_wh[1] / fig.get_dpi()))
  ax.plot_date(sloc_x, sloc_y_total, linestyle='solid', label='Total', color='#202020')
  for lang, lang_loc_counts in sloc_languages_y_dict.items():
    ax.plot_date(sloc_x, lang_loc_counts, linestyle='solid', label=f'{lang}', color=lang_colors[lang_colors_i])
    lang_colors_i += 1
    if lang_colors_i >= len(lang_colors):
      lang_colors_i = 0

  ax.xaxis.set_major_formatter(matplotlib.dates.DateFormatter("%Y-%m-%d %H:%M"))
  ax.autoscale_view()
  ax.set_title('SLOC')
  ax.set_ylabel('Lines of Code')
  ax.grid(True)
  fig.autofmt_xdate()
  fig.legend()

  fig.savefig(loc_graph_file)



