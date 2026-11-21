#!/usr/bin/env python3
"""Validate the basic structure of a GLB and report included scene counts (stdlib only)."""
import json,struct,sys
from pathlib import Path
p=Path(sys.argv[1] if len(sys.argv)>1 else 'assets/office_original_textures_complete.glb')
with p.open('rb') as f:
    header=f.read(12)
    assert len(header)==12,'Short GLB header'
    magic,version,length=struct.unpack('<4sII',header)
    assert magic==b'glTF' and version==2, 'Expected GLB v2'
    assert length==p.stat().st_size,'File size mismatch'
    length_json,kind=struct.unpack('<I4s',f.read(8))
    assert kind==b'JSON','Missing JSON chunk'
    obj=json.loads(f.read(length_json))
    bytes_bin,kind=struct.unpack('<I4s',f.read(8))
    assert kind==b'BIN\x00','Missing BIN chunk'
    assert bytes_bin>=obj['buffers'][0]['byteLength']
    assert sum(obj['buffers'][0]['byteLength']<=bytes_bin for _ in [0])==1
    for view in obj['bufferViews']:
        assert view['byteOffset']+view['byteLength']<=obj['buffers'][0]['byteLength'], 'Out-of-bounds buffer view'
    print('GLB VALID:',p)
    for field in ['nodes','meshes','materials','images','textures','accessors']:
        print(f'  {field:>11}: {len(obj.get(field,[]))}')
    print('  total size:',round(p.stat().st_size/1048576,2),'MiB')
