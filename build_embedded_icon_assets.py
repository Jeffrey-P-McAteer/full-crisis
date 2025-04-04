# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "Pillow",
# ]
# ///

import os
import sys

import PIL
import PIL.Image

full_size_icon = os.path.join(os.path.dirname(__file__), 'icon', 'full-crisis-icon.png')
if not os.path.exists(full_size_icon):
  print(f'Expected to find {full_size_icon}, but that file does not exist!')
  sys.exit(1)

for s in [16,32,64]:
  icon_img = PIL.Image.open(full_size_icon)

  output_bin = os.path.join(os.path.dirname(__file__), 'icon', f'full-crisis-icon.{s}x{s}.rgba.bin')
  icon_img.thumbnail((s, s), PIL.Image.Resampling.LANCZOS)

  pix = icon_img.load()

  # size * size image of RGBA pixels (each 4 * u8) in row-major order.
  output_bytes = bytearray()
  for y in range(0, s):
    for x in range(0, s):
      r = pix[x,y][0]
      g = pix[x,y][1]
      b = pix[x,y][2]
      a = pix[x,y][3]
      output_bytes.extend(bytes([r,g,b,a]))

  with open(output_bin, 'wb') as fd:
    fd.write(output_bytes)

  print(f'Wrote {len(output_bytes)} bytes to {output_bin}')


# Also write out a .ico asset for windows embedding into the .exe format
icon_img = PIL.Image.open(full_size_icon)
icon_ico_file = os.path.join(os.path.dirname(__file__), 'icon', f'full-crisis-icon.ico')
# See https://learn.microsoft.com/en-us/windows/apps/design/style/iconography/app-icon-construction
icon_sizes = [(16,16), (24, 24), (32,32), (48, 48), (64,64), (96,96), (256,256)]
icon_img.save(icon_ico_file, sizes=icon_sizes)
print(f'Saved {icon_ico_file}')

