#!/usr/bin/env python3
"""Validate RUN CARTER v0.15 imported GLBs and original CarterOS unchanged hash records."""
import json,hashlib,sys
from pathlib import Path
root=Path(__file__).resolve().parents[1]
manifest=json.loads((root/'assets/arsenal/manifest.json').read_text())
assert manifest['asset_count']==21
assert manifest['weapon_count']==16
assert len(manifest['models'])==21
for m in manifest['models']:
 p=root/'assets/arsenal'/m['file']
 data=p.read_bytes()
 assert data[:4]==b'glTF' and int.from_bytes(data[8:12],'little')==len(data),p
 assert len(data)==m['size_bytes'] and hashlib.sha256(data).hexdigest()==m['sha256'],p
 j=json.loads(data[20:20+int.from_bytes(data[12:16],'little')])
 assert m['meshes']==len(j.get('meshes',[])),p
print('PASS: 21 GLBs, 16 weapon assets, hash+container+JSON checks; runtime rendering not implied')
