#!/usr/bin/env python3
"""Verify all imported GLB files against their regenerated manifest (no network/dependencies)."""
from pathlib import Path
import json,hashlib,struct,sys
root=Path(__file__).resolve().parents[1]
manifest=json.loads((root/'assets/arsenal/manifest.json').read_text())
errors=[]
files=manifest['models']
for info in files:
    path=root/'assets/arsenal'/info['file']
    if not path.is_file():errors.append('missing '+info['file']);continue
    data=path.read_bytes()
    if hashlib.sha256(data).hexdigest()!=info['sha256']:errors.append('changed '+info['file']);continue
    if len(data)!=info['size_bytes']:errors.append('size mismatch '+info['file']);continue
    try:
        magic,version,length=struct.unpack_from('<4sII',data)
        assert magic==b'glTF' and version==2 and length==len(data)
        jsonlen,jtype=struct.unpack_from('<I4s',data,12)
        assert jtype==b'JSON'
        meta=json.loads(data[20:20+jsonlen])
        assert len(meta.get('meshes',[]))==info['meshes']
        assert len(meta.get('animations',[]))==len(info['animations'])
    except Exception as e:errors.append('invalid glb '+info['file']+': '+str(e))
if len(files)!=21:errors.append('Expected 21 unique GLBs including six newly supplied weapons')
required=['cardboard_blaster.glb','knife.glb','mark23.glb','vector.glb','m4.glb',
 'minigun.glb','flamethrower.glb','rpg7.glb','grenade.glb','dynamite.glb',
 'shotgun.glb','sniper.glb','persuader.glb','lmg.glb','grenadelauncher.glb','pistol.glb',
 'arms.glb','fan_character.glb','military_crate.glb','storage_crates.glb','weapon_crate.glb']
for rel in required:
 if rel not in {info['file'] for info in files}:errors.append('unregistered '+rel)
if errors:
 print('FAIL:',*errors,sep='\n - ');sys.exit(1)
print('PASS:',len(files),'verified GLB assets with correct SHA-256 and parseable metadata')
print('NOTE: this checks file integrity, not raylib animation/runtime rendering')
