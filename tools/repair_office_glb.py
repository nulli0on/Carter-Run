#!/usr/bin/env python3
"""RUN CARTER: clearly labelled *replacement* materials for under-specified FBX surfaces.

The original FBX has dozens of valid embedded picture/prop textures, but its
'plywood', 'wod floring', 'white wall' and door materials have NO linked diffuse
images. This script preserves the working texture links from the lite GLB,
generates low-resolution tileable wood/plaster surfaces for otherwise blank
materials, and projects UVs from existing world-space geometry.

Not a reconstruction of original, unprovided author textures. Requires numpy,
Pillow. Does NOT need the original FBX or any internet services.

Usage: python3 tools/repair_office_glb.py --input assets/office_lite.glb \\
          --output assets/office_repaired.glb
"""
from __future__ import annotations
import argparse, io, json, math, struct
from pathlib import Path
import numpy as np
from PIL import Image

# Material names from the exact uploaded FBX. Small, deliberate changes only.
REPLACEMENTS = {
    'wod floring': ('oak_floor', 0.90),
    'plywood': ('oak_desk', 0.80),
    'plywood 1': ('walnut_furniture', 0.80),
    'plywood 2': ('walnut_furniture', 0.80),
    'Door_material': ('walnut_furniture', 0.80),
    'white wall': ('plaster', 0.65),
}
# Material colors adjusted without inventing image assignments.
# These thirteen original FBX meshes have no connected material. Matching them
# to already-defined materials is an informed visual approximation, not source
# recovery. Keep all choices explicit and reversible.
INFERRED_MATERIALS = {
    'Cube.031': 'plywood 2', 'Cube.032': 'plywood 2',
    'Cube.033': 'plywood 2', 'Cube.034': 'plywood 2',
    'cable_outlets_2': 'black plastic',
    'Cube.018': 'matt white plastic',
    'Cube.023': 'black plastic', 'Cube.024': 'black plastic',
    'Plane': 'photos', 'Plane.001': 'photos',
    'Cylinder': 'metal',
    'Icosphere': 'light bulb', 'Icosphere.001': 'light bulb',
}
MAT_COLORS = {
    'cardboard handle': (0.28, 0.31, 0.32),
    'frame': (0.13, 0.15, 0.17),
    'frame 2': (0.18, 0.20, 0.21),
    'photo frame': (0.17, 0.19, 0.21),
    'Metal.001': (0.59, 0.60, 0.61),
    'hidden_material': (0.66, 0.67, 0.68),
    'AUTO_FALLBACK__unassigned': (0.43, 0.46, 0.49),
}

def make_wood(seed, base_rgb, floor=False, size=512):
    rng = np.random.default_rng(seed)
    y, x = np.mgrid[:size, :size].astype(np.float32)
    # Each texture tile spans several boards; visually distinct seams and grain.
    boards = 4 if floor else 2
    period = size / boards
    cy = (y % period)/period
    band = np.floor(y/period).astype('int32')
    palette = rng.normal(0, 7, size=(boards,3)).astype(np.float32)
    patch = palette[band]
    winding = x + 10*np.sin(y/13.0 + 0.9*np.sin(x/81.0))
    grain = (np.sin(winding*0.18 + np.sin(y/11.0)) * 4.2
             + np.sin(winding*0.48 + y*0.13) * 1.2
             + np.sin(winding*0.047 + 0.5*y/23.0) * 5.0)
    noise = rng.normal(0,1.7,size=(size,size)).astype(np.float32)
    seam=np.where((cy<0.026)|(cy>0.974),-47.0,0.0)
    # Staggered end joints in the floor, subtle on the desk.
    if floor:
        joints=np.where(np.abs(((x + (band%2)*size*.5) % size)-size*.5) < 2.5,-32.,0.)
        seam=seam+joints
    base=np.array(base_rgb,dtype=np.float32)
    img=np.clip(base[None,None,:]+patch+grain[:,:,None]+noise[:,:,None]+seam[:,:,None],0,255).astype('uint8')
    return Image.fromarray(img,'RGB')

def make_plaster(size=256):
    rng=np.random.default_rng(147)
    noise=rng.normal(0, 1.45, size=(size,size,1))
    # Subdued, intentionally no bright pure white.
    rgb=np.clip(np.array([184,190,187],dtype='float32')+noise,0,255).astype('uint8')
    return Image.fromarray(rgb,'RGB')

TEXTURES = {
    'oak_floor':lambda:make_wood(17,(112,87,65),True),
    'oak_desk':lambda:make_wood(29,(165,130,93)),
    'walnut_furniture':lambda:make_wood(43,(119,94,73)),
    'plaster':make_plaster,
}

def load_glb(path):
    data=Path(path).read_bytes()
    if data[:4]!=b'glTF' or struct.unpack_from('<I',data,4)[0]!=2:
        raise ValueError('Not a glTF 2.0 GLB')
    length=struct.unpack_from('<I',data,8)[0]
    if length!=len(data): raise ValueError('GLB size mismatch')
    jlength,jtype=struct.unpack_from('<I4s',data,12)
    if jtype!=b'JSON':raise ValueError('No JSON header chunk')
    meta=json.loads(data[20:20+jlength]);offset=20+jlength
    blength,btype=struct.unpack_from('<I4s',data,offset)
    if btype!=b'BIN\x00':raise ValueError('No BIN chunk')
    return meta,bytearray(data[offset+8:offset+8+blength])

def append_view(meta,binary,data,target=None):
    while len(binary)%4: binary.append(0)
    view={'buffer':0,'byteOffset':len(binary),'byteLength':len(data)}
    if target: view['target']=target
    meta['bufferViews'].append(view)
    binary.extend(data)
    return len(meta['bufferViews'])-1

def get_array(meta,binary,idx):
    a=meta['accessors'][idx];v=meta['bufferViews'][a['bufferView']]
    dtype={5126:'<f4',5125:'<u4',5123:'<u2'}[a['componentType']]
    items={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[a['type']]
    off=v.get('byteOffset',0)+a.get('byteOffset',0)
    step=v.get('byteStride')
    if step:
        raw=np.ndarray((a['count'],items),dtype=dtype,buffer=binary,offset=off,strides=(step,np.dtype(dtype).itemsize)).copy()
    else:
        raw=np.frombuffer(binary,dtype=dtype,count=a['count']*items,offset=off).reshape((-1,items)).copy()
    return raw

def add_float2(meta,binary,arr):
    arr=np.ascontiguousarray(arr,dtype='<f4')
    view=append_view(meta,binary,arr.tobytes(),34962)
    meta['accessors'].append({'bufferView':view,'componentType':5126,'count':len(arr),'type':'VEC2'})
    return len(meta['accessors'])-1

def projected_uv(pos,norm,kind,frequency):
    # Face-oriented planar projection, scale in world metres. The source meshes
    # already have split normals on most box faces, so the projection stays crisp.
    axis=np.argmax(np.abs(norm),axis=1)
    a=np.where(axis==0,pos[:,2],pos[:,0])
    b=np.where(axis==1,pos[:,2],pos[:,1])
    if kind=='oak_floor':
        a=pos[:,0]
        b=pos[:,2]
    if kind=='plaster':
        a=a*1.25;b=b*1.25
    return np.stack((a*frequency,b*frequency),axis=1)

def write_glb(path,meta,binary):
    while len(binary)%4:binary.append(0)
    meta['buffers'][0]['byteLength']=len(binary)
    j=json.dumps(meta,separators=(',',':'),ensure_ascii=False).encode('utf8')
    j+=b' '*((-len(j))%4)
    size=12+8+len(j)+8+len(binary)
    with open(path,'wb') as f:
        f.write(b'glTF');f.write(struct.pack('<II',2,size))
        f.write(struct.pack('<I4s',len(j),b'JSON'));f.write(j)
        f.write(struct.pack('<I4s',len(binary),b'BIN\x00'));f.write(binary)

def run(src,out):
    meta,bin=load_glb(src)
    mats=meta['materials']
    tex_index={}
    for label,generator in TEXTURES.items():
        image=generator();output=io.BytesIO();image.save(output,'JPEG',quality=85,subsampling=0,optimize=True)
        imindex=len(meta['images']);meta['images'].append({'name':f'RUN_CARTER_PROCEDURAL_{label}.jpg','mimeType':'image/jpeg','bufferView':append_view(meta,bin,output.getvalue())})
        tindex=len(meta['textures']);meta['textures'].append({'source':imindex,'sampler':0})
        tex_index[label]=tindex
    report={'input_glb':str(src),'output_glb':str(out),'authenticity':'Original model + original surviving image textures + explicitly generated replacement surfaces. NOT a restoration of unavailable original image maps.','procedural_materials':{},'original_textured_materials':[],'untextured_materials_remaining':[],'new_uv_accessors':0,'inferred_unassigned_objects':{}}
    for i,m in enumerate(mats):
        name=m.get('name','')
        pbr=m['pbrMetallicRoughness']
        if name in REPLACEMENTS:
            kind,scale=REPLACEMENTS[name]
            pbr['baseColorTexture']={'index':tex_index[kind]}
            pbr['baseColorFactor']=[1,1,1,1]
            pbr['roughnessFactor']=0.93
            report['procedural_materials'][name]={'texture':kind,'frequency_per_m':scale}
        elif name in MAT_COLORS:
            pbr['baseColorFactor']=[*MAT_COLORS[name],1]
    material_by_name={m.get('name'):i for i,m in enumerate(mats)}
    for mesh in meta['meshes']:
        for p in mesh['primitives']:
            if mesh.get('name') in INFERRED_MATERIALS and mats[p['material']].get('name')=='AUTO_FALLBACK__unassigned':
                guess=INFERRED_MATERIALS[mesh['name']]
                if guess not in material_by_name:raise ValueError('Unknown guessed material: '+guess)
                p['material']=material_by_name[guess]
                report['inferred_unassigned_objects'][mesh['name']]=guess
            m=mats[p['material']];name=m.get('name','')
            if name in REPLACEMENTS:
                kind,scale=REPLACEMENTS[name]
                pos=get_array(meta,bin,p['attributes']['POSITION'])
                nor=get_array(meta,bin,p['attributes']['NORMAL'])
                uv=projected_uv(pos,nor,kind,scale)
                p['attributes']['TEXCOORD_0']=add_float2(meta,bin,uv)
                report['new_uv_accessors']+=1
            elif 'baseColorTexture' not in m['pbrMetallicRoughness']:
                # Original geometry has UV channels even without textures. Remove
                # unused attributes to reduce GLTF_VALIDATOR UNUSED_OBJECT noise.
                p['attributes'].pop('TEXCOORD_0',None)
    for m in mats:
        name=m.get('name','')
        if name not in REPLACEMENTS:
            if 'baseColorTexture' in m['pbrMetallicRoughness']:
                report['original_textured_materials'].append(name)
            else:
                report['untextured_materials_remaining'].append(name)
    output=Path(out);output.parent.mkdir(parents=True,exist_ok=True)
    write_glb(output,meta,bin)
    report['size_mib']=round(output.stat().st_size/1048576,2)
    report_path=output.with_suffix('.repair.json')
    report_path.write_text(json.dumps(report,indent=2),encoding='utf8')
    print(f'Wrote {output} ({report["size_mib"]} MiB)')
    print(f'Procedurally surfaced: {len(report["procedural_materials"])} materials; existing original textures preserved: {len(report["original_textured_materials"])} materials')
    print(f'UV channels regenerated on {report["new_uv_accessors"]} geometry primitives')
    print(f'Inferred materials for {len(report["inferred_unassigned_objects"])} originally unassigned objects')
    print('Report:',report_path)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--input',default='assets/office_lite.glb')
    p.add_argument('--output',default='assets/office_repaired.glb')
    a=p.parse_args();run(a.input,a.output)
