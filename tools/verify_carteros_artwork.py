#!/usr/bin/env python3
"""Verify the 36 original exported Figma PNG files, with no external dependencies."""
from pathlib import Path
import hashlib, json
ROOT=Path(__file__).resolve().parents[1]
manifest=json.loads((ROOT/'docs/CARTEROS_ARTWORK_HASHES.json').read_text())
files=manifest['files']
assert len(files)==36,len(files)
assert len(set(x['file'] for x in files))==len(files)
for info in files:
    path=ROOT/'assets/carteros'/info['file']
    assert path.is_file(),f'MISSING Figma asset: {path.name}'
    data=path.read_bytes()
    assert len(data)==info['bytes'],path.name
    assert hashlib.sha256(data).hexdigest()==info['sha256'],path.name
print('PASS: all 36 original CarterOS Figma PNG files match exact SHA-256 and byte length')
