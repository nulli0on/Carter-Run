#!/usr/bin/env python3
"""Check source-derived geometry stability and repaired material references."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from repair_office_glb import load_glb, REPLACEMENTS, INFERRED_MATERIALS
root=Path(__file__).resolve().parent.parent/'assets'
baseline,_=load_glb(root/'office_lite.glb')
fixed,_=load_glb(root/'office_repaired.glb')
assert len(baseline['meshes'])==len(fixed['meshes'])==147
assert len(baseline['nodes'])==len(fixed['nodes'])==147
assert len(baseline['images'])==38 and len(fixed['images'])==42
assert [m['name'] for m in fixed['meshes']]==[m['name'] for m in baseline['meshes']]
assert sum(len(m['primitives']) for m in baseline['meshes'])==sum(len(m['primitives']) for m in fixed['meshes'])
assert all(a['attributes']['POSITION']==b['attributes']['POSITION'] and a['indices']==b['indices']
           for base_mesh,fixed_mesh in zip(baseline['meshes'],fixed['meshes'])
           for a,b in zip(base_mesh['primitives'],fixed_mesh['primitives']))
fix_by_name={m['name']:m for m in fixed['materials']}
assert all('baseColorTexture' in fix_by_name[n]['pbrMetallicRoughness'] for n in REPLACEMENTS)
assert len(INFERRED_MATERIALS)==13
assert all(n.startswith('RUN_CARTER_PROCEDURAL_') for n in [i['name'] for i in fixed['images'][-4:]])
reassigned=0
for mesh in fixed['meshes']:
    if mesh['name'] in INFERRED_MATERIALS:
        reassigned+=1
        for p in mesh['primitives']:
            assert fixed['materials'][p['material']]['name']==INFERRED_MATERIALS[mesh['name']]
assert reassigned==13
print('PASS: geometry counts, triangles/accessors, original images, generated materials, 13 inferred assignments')
