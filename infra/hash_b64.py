#!python3

import base64
import hashlib
import json
from os import path
import sys

obj_in = json.load(sys.stdin)
obj_out = {}

for name, file_path in obj_in.items():
    hash_object = hashlib.sha256()

    if not path.isfile(file_path):
        obj_out[name] = ""
        continue

    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            hash_object.update(chunk)

    hash_binary = hash_object.digest()
    obj_out[name] = base64.b64encode(hash_binary).decode('utf-8')

json.dump(obj_out, sys.stdout)
