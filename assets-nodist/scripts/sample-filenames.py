#! /usr/bin/python

import os
import stringcase

files = os.listdir(".")
print(files)

for file in files:
    if file.endswith(".wav"):
        name = stringcase.spinalcase(file[:-4]).replace("--", "-")
        name = stringcase.titlecase(name[0]) + name[1:]
        print("(\"%s\", \"%s\")," % (name, file))
