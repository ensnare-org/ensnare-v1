#!/usr/bin/python

import colorsys
import math
import sys

# https://color.fandom.com/wiki/List_of_hues
HUE_NAMES = ["Red",
             "Prysa",
             "Zarqa",
             "Pengwin",
             "Vermilion",
             "Colyw",
             "Haha",
             "Gabby",
             "Orange",
             "Winter",
             "Sharasha",
             "Buzz",
             "Amber",
             "Builder",
             "Lekko",
             "Villager",
             "Yellow",
             "Yolett",
             "Lemon",
             "Burton",
             "Lime",
             "Sprite",
             "Jay",
             "Robin",
             "Chartreuse",
             "Misho",
             "Ggahhal",
             "Evbo",
             "Ddahal",
             "Swimsuit",
             "Medu",
             "Pearlike",
             "Green",
             "Lively",
             "Cypher",
             "Veh",
             "Erin",
             "Erus",
             "Jus",
             "Emeraldstar",
             "Spring",
             "Gold",
             "Mool",
             "Diamond",
             "Gashyanta",
             "Yreli",
             "Uroz",
             "Bobi",
             "Cyan",
             "Feil",
             "Zinor",
             "Twits",
             "Capri",
             "Iapion",
             "Uzor",
             "Underwater",
             "Azure",
             "Wet",
             "Zinoret",
             "Harza",
             "Cerulean",
             "Doto",
             "Zarqaret",
             "Gloomy",
             "Blue",
             "Rarity",
             "Linel",
             "Fluttershy",
             "Volta",
             "Over",
             "Kyryn",
             "Smot",
             "Violet",
             "Twilight",
             "Zinur",
             "Chwarae",
             "Llew",
             "Howl",
             "Kyrene",
             "Skelato",
             "Magenta",
             "Kirpan",
             "Hung",
             "Minkraf",
             "Cerise",
             "Wooder",
             "Loo",
             "Jerin",
             "Rose",
             "Hesonbwon",
             "Jerry",
             "Fasha",
             "Crimson",
             "Rara",
             "Khwarezmian",
             "Tata"]

accepted_colors = []


def add_to_accepted(name, r, g, b):
    r = int(r * 255.0)
    g = int(g * 255.0)
    b = int(b * 255.0)
    rgb = f"{r:02x}{g:02x}{b:02x}"
    darkness = (r * 0.299 + g * 0.587 + b * .114)
    accepted_colors.append(
        {"r": r, "g": g, "b": b, "rgb": rgb, "name": name, "darkness": darkness})


# Generate very light, somewhat saturated colors from named hues
for (count, name) in enumerate(HUE_NAMES):
    if count % 4 != 0:
        continue
    hue = count / len(HUE_NAMES)
    (r, g, b) = colorsys.hls_to_rgb(hue, 0.8, 1.0)
    add_to_accepted(name, r, g, b)

# Generate grays
count = 1
for hue in range(0, 256, 32):
    darkness = hue / 255.0
    hue = 0.0
    saturation = 0.0
    (r, g, b) = colorsys.hls_to_rgb(hue, darkness, saturation)
    add_to_accepted(f"Gray{count}", r, g, b)
    count = count + 1

print(
    "<html><head><style>body {font: 16px Helvetica Neue, sans-serif;} td {padding: 4px} .dark {color: white} .light {color: black}</style></head><body>")

code = ""
for ac in accepted_colors:
    code += ac["name"] + ",\r\n"
print(f"<pre>enum PatternColorScheme {{\r\n{code}}}</pre>")

print(
    "<html><head><style>body {font: 16px Helvetica Neue, sans-serif;} td {padding: 4px} .dark {color: white} .light {color: black}</style></head><body><table>")
code = ""
for ac in accepted_colors:
    r = ac["r"]
    g = ac["g"]
    b = ac["b"]
    rgb = ac["rgb"]
    name = ac["name"]
    darkness = ac["darkness"]
    if darkness < 150:
        css_class = "dark"
        is_dark = "WHITE"
    else:
        css_class = "light"
        is_dark = "BLACK"
    print(
        f"<tr><td class=\"{css_class}\" style=\"background-color:#{rgb};\">{name}: {rgb}</td></tr>")
    code += f"PatternColorScheme::{name} => (Color32::{is_dark}, Color32::from_rgb({r}, {g}, {b})),\r\n"
print(f"</table><pre>{code}</pre>")

print(f"</body></html>")
