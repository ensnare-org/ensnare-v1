#!/usr/bin/python3

# This script will convert the named icons from SVG into a PNG format that's
# suitable to use in this app.
#
# To use:
#
# 1. Clone the repo
#    [material-design-icons](https://github.com/google/material-design-icons/)
#    and unzip somewhere. In these instructions we'll assume the location is
#    ~/Downloads/material-design-icons.
# 2. cd to the root of this source tree.
# 3. res/tools/generate-md-icons.py ~/Downloads/material-design-icons
#
# The reason we're not using fonts is because of
# https://github.com/emilk/egui/issues/3526
#
# NOTE that cloning the repo is different from downloading the GitHub release
# zip. In particular, the symbols/ directory at the root is missing.

import subprocess
import sys

# Browse available icons at https://fonts.google.com/icons?icon.platform=web
ICONS = {
    'av': ['play_arrow', 'pause', 'stop'],
}

md_dir = sys.argv[1]
print(
    "Reading material design icons/symbols from base directory {md_dir}".format(md_dir=md_dir))

subprocess.run(["mkdir", "-p", "res/images/md-icons"])
subprocess.run(["mkdir", "-p", "res/images/md-symbols"])

for (group, icons) in ICONS.items():
    for name in icons:
        outfile = "res/images/md-icons/{name}.png".format(name=name)
        args = ["convert",
                "{md_dir}/src/{group}/{name}/materialicons/24px.svg".format(md_dir=md_dir,
                                                                            group=str(
                                                                                group),
                                                                            name=name),
                "-density", "576",
                "-background", "none",
                "-negate", outfile]
        subprocess.run(args)
        subprocess.run(["mogrify", "-strip", outfile])

# Browse available symbols at https://fonts.google.com/icons
SYMBOLS = [
    'audio_file',
    'file_open',
    'file_save',
    'new_window',
    'piano',
    'play_arrow',
    'playlist_add_circle',
    'settings',
    'stop',
    'view_timeline',
]
for (symbol) in SYMBOLS:
    outfile = "res/images/md-symbols/{symbol}.png".format(symbol=symbol)
    args = ["convert",
            "{md_dir}/symbols/web/{symbol}/materialsymbolssharp/{symbol}_wght100_24px.svg".format(md_dir=md_dir,
                                                                                                  symbol=symbol),
            "-density", "576",
            "-background", "none",
            "-negate", outfile]
    subprocess.run(args)
    subprocess.run(["mogrify", "-strip", outfile])
