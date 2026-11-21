#!/usr/bin/env python3
"""Reintegrate low-detail original chair and desk props via voxel-based simplification."""
import importlib.util,json,sys
from pathlib import Path
import numpy as np
import argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--base',default='assets/office_original_textures.glb')
parser.add_argument('--high-detail',required=True,help='FBX converter result with --budget 220000')
parser.add_argument('--output',default='assets/office_original_textures_complete.glb')
args=parser.parse_args()
script=Path(__file__).resolve().with_name('repair_office_glb.py')
spec=importlib.util.spec_from_file_location('glb_util',script);glb=importlib.util.module_from_spec(spec);spec.loader.exec_module(glb)
mb,bb=glb.load_glb(args.base)
mh,bh=glb.load_glb(args.high_detail)

# Quantization spacing in world metres; conserves silhouettes while removing
# dense sub-millimetre detail incompatible with the target integrated GPU.
ADD={
 'CUSHION':.035,
 'pillow':.036,
 'pillow.001':.036,
 'bottom frame':.018,
 'leggy':.018,
 'metal frame':.015,
 'leather':.028,
 'Keyboard':.008,
 'Mug':.005,
 'mug content':.005,
 'Gum_branch_01':.025,
 'Gum_branch_02':.025,
 'BabyBreaths_01a.001':.027,
}

def get_accessor(m,b,idx):
 a=m['accessors'][idx];view=m['bufferViews'][a['bufferView']]
 dims={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[a['type']]
 dt={5126:'<f4',5125:'<u4',5123:'<u2',5121:'u1'}[a['componentType']]
 off=view.get('byteOffset',0)+a.get('byteOffset',0)
 stride=view.get('byteStride')
 if stride:
  ar=np.ndarray((a['count'],dims),dtype=dt,buffer=b,offset=off,strides=(stride,np.dtype(dt).itemsize)).copy()
 else:
  ar=np.frombuffer(b,dtype=dt,count=a['count']*dims,offset=off).reshape(-1,dims).copy()
 return ar

def add_array(data,component,type_,target=None,bounds=False):
 data=np.ascontiguousarray(data)
 vid=glb.append_view(mb,bb,data.tobytes(),target)
 acc={'bufferView':vid,'componentType':component,'count':len(data),'type':type_}
 if bounds and len(data):
  acc['min']=data.min(axis=0).astype(float).tolist()
  acc['max']=data.max(axis=0).astype(float).tolist()
 mb['accessors'].append(acc)
 return len(mb['accessors'])-1

def compact(pos,tri,norm,uv,spacing):
 # Cluster vertices spatially; triangles fully collapsed by the clustering
 # operation are removed. This algorithm does not invent new geometry.
 cells=np.floor(np.asarray(pos,dtype=np.float64)/spacing+.5).astype(np.int32)
 _,inverse=np.unique(cells,axis=0,return_inverse=True)
 n=int(inverse.max()+1)
 count=np.bincount(inverse,minlength=n)
 def merge(a):
  return np.stack([np.bincount(inverse,weights=a[:,c],minlength=n)/count for c in range(a.shape[1])],axis=-1).astype('<f4')
 p2=merge(pos);nor=merge(norm)
 mag=np.linalg.norm(nor,axis=1,keepdims=True);nor/=np.maximum(mag,1e-8)
 uv2=merge(uv) if uv is not None else None
 mapped=inverse[tri]
 valid=(mapped[:,0]!=mapped[:,1])&(mapped[:,0]!=mapped[:,2])&(mapped[:,1]!=mapped[:,2])
 mapped=mapped[valid]
 if len(mapped):
  # discard triangles that collapse to the same 3 vertices
  _,mask_indices=np.unique(np.sort(mapped,axis=1),axis=0,return_index=True)
  mapped=mapped[np.sort(mask_indices)]
 if len(mapped)<20:
  print('WARN: overaggressive simplification, reverting to source',file=sys.stderr)
  return pos,tri,norm,uv
 used,remap=np.unique(mapped.ravel(),return_inverse=True)
 return p2[used],remap.reshape(-1,3).astype('<u4'),nor[used],uv2[used] if uv2 is not None else None

report={'source':'meshes from user-uploaded office 3.fbx, simplified with deterministic spatial clustering','added_source_objects':[],'high_detail_budget':220000,'warnings':[]}
matnames={x['name']:i for i,x in enumerate(mb['materials'])}
for mesh in mh['meshes']:
 name=mesh['name']
 if name not in ADD:continue
 spacing=ADD[name]
 rebuilt=[];sum_old=0;sum_new=0
 for prim in mesh['primitives']:
  tri=get_accessor(mh,bh,prim['indices']).reshape(-1).astype(np.uint32).reshape(-1,3)
  pos=get_accessor(mh,bh,prim['attributes']['POSITION']).astype('float32')
  norm=get_accessor(mh,bh,prim['attributes']['NORMAL']).astype('float32')
  uv=get_accessor(mh,bh,prim['attributes']['TEXCOORD_0']).astype('float32') if 'TEXCOORD_0' in prim['attributes'] else None
  p2,ind2,n2,uv2=compact(pos,tri,norm,uv,spacing)
  matname=mh['materials'][prim['material']]['name'];matidx=matnames[matname]
  attrs={'POSITION':add_array(p2.astype('<f4'),5126,'VEC3',34962,True),'NORMAL':add_array(n2.astype('<f4'),5126,'VEC3',34962)}
  if 'baseColorTexture' in mb['materials'][matidx]['pbrMetallicRoughness']:
   if uv2 is not None:attrs['TEXCOORD_0']=add_array(uv2.astype('<f4'),5126,'VEC2',34962)
   else:report['warnings'].append(f'Missing UV on {name}/{matname}')
  iidx=add_array(ind2.reshape(-1).astype('<u4'),5125,'SCALAR',34963)
  rebuilt.append({'attributes':attrs,'indices':iidx,'material':matidx,'mode':4})
  sum_old+=len(tri);sum_new+=len(ind2)
 if rebuilt:
  mesh_index=len(mb['meshes']);mb['meshes'].append({'name':name,'primitives':rebuilt})
  node_index=len(mb['nodes']);mb['nodes'].append({'name':name,'mesh':mesh_index})
  mb['scenes'][mb.get('scene',0)]['nodes'].append(node_index)
  report['added_source_objects'].append({'name':name,'source_triangles':sum_old,'game_triangles':sum_new,'voxel_metres':spacing})
  print(f'Added {name:22} {sum_old:>7,} → {sum_new:>7,} triangles',flush=True)
# Source mug is untextured generic materials: approximate red enamel and beverage.
for name,rgb in [('Material.002',[.72,.075,.05,1]),('Material.003',[.08,.07,.065,1]),('Material.001',[.20,.12,.075,1])]:
 mat=mb['materials'][matnames[name]]
 mat['pbrMetallicRoughness'].pop('baseColorTexture',None)
 mat['pbrMetallicRoughness']['baseColorFactor']=rgb
 mat['pbrMetallicRoughness']['roughnessFactor']=0.36
out=Path(args.output)
glb.write_glb(out,mb,bb)
report['size_mib']=round(out.stat().st_size/2**20,2)
report['original_triangles_added']=sum(v['source_triangles'] for v in report['added_source_objects'])
report['decimated_triangles_added']=sum(v['game_triangles'] for v in report['added_source_objects'])
out.with_suffix('.mesh-restore.json').write_text(json.dumps(report,indent=2),encoding='utf8')
print('Saved',out,report['size_mib'],'MiB','added',report['decimated_triangles_added'])
