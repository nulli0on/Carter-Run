#!/usr/bin/env python3
"""Rebuild the office's glTF materials using texture maps from owner-supplied TEXTURE.rar.

This is an explicit source-guided asset repair, not a claim that the exporter
restores Blender shader nodes or proprietary FBX material bindings verbatim.
"""
from __future__ import annotations
import importlib.util, io, json, sys
from pathlib import Path
from PIL import Image,ImageOps
import numpy as np

import argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--input',default='assets/office_repaired.glb',help='previous GLB with projected plywood/floor UVs')
parser.add_argument('--texture-root',required=True,help='extracted TEXTURE folder from the owner-supplied RAR')
parser.add_argument('--output',default='assets/office_original_textures.glb')
args=parser.parse_args()
SRC=Path(args.input)
OUT=Path(args.output)
BASE=Path(args.texture_root)
SCRIPT=Path(__file__).resolve().with_name('repair_office_glb.py')
spec=importlib.util.spec_from_file_location('repair',SCRIPT)
repair=importlib.util.module_from_spec(spec);sys.modules['repair']=repair;spec.loader.exec_module(repair)
m,b=repair.load_glb(SRC)
mat_by_name={v['name']:v for v in m['materials']}
asset_to_tex={}
report={'source':'user-supplied TEXTURE.rar + office 3.fbx', 'source_glb':str(SRC),'materials_with_source_color_textures':{},'material_color_fixes':{},'planar_projection_meshes':[], 'notes':['The original pack has the exact diffuse maps for floor, plywood, computer desktop, backdrop, and more, but its material node tree was not supplied. Matches on material names/geometry are manually reconstructed.','Geometry inherited from optimized v0.2 GLB, so 18 over-budget models including the chair are not yet in this variant.']}

def tex(rel,size=1024,quality=88):
 rel=rel.replace('\\','/')
 key=(rel,size)
 if key in asset_to_tex:return asset_to_tex[key]
 filename=BASE / rel
 if not filename.is_file():raise FileNotFoundError(filename)
 with Image.open(filename) as img:
  img=ImageOps.exif_transpose(img)
  if max(img.size)>size:img.thumbnail((size,size),Image.Resampling.LANCZOS)
  out=io.BytesIO()
  if 'A' in img.getbands() and img.getextrema()[-1]!=(255,255):
   img.convert('RGBA').save(out,'PNG',optimize=True)
   mimetype='image/png';ext='png'
  else:
   img.convert('RGB').save(out,'JPEG',quality=quality,subsampling=0,optimize=True)
   mimetype='image/jpeg';ext='jpg'
  data=out.getvalue()
 imgidx=len(m['images']);m['images'].append({'name':'OWNER_PACK__'+Path(rel).stem+'.'+ext,'mimeType':mimetype,'bufferView':repair.append_view(m,b,data)})
 ti=len(m['textures']);m['textures'].append({'source':imgidx,'sampler':0})
 asset_to_tex[key]=ti
 return ti

def assign(name,rel,rough=.8,met=0.,max_side=1024):
 mat=mat_by_name[name]
 pbr=mat['pbrMetallicRoughness']
 pbr['baseColorTexture']={'index':tex(rel,max_side)}
 pbr['baseColorFactor']=[1.,1.,1.,1.]
 pbr['roughnessFactor']=rough;pbr['metallicFactor']=met
 report['materials_with_source_color_textures'][name]=rel

def plain(name,rgb,rough=.9,met=0.,alpha=1.):
 mat=mat_by_name[name];pbr=mat['pbrMetallicRoughness'];pbr.pop('baseColorTexture',None)
 pbr['baseColorFactor']=[*rgb,alpha]
 pbr['roughnessFactor']=rough;pbr['metallicFactor']=met
 report['material_color_fixes'][name]=list(rgb)

for name in ['plywood','plywood 1','plywood 2']:
 assign(name,'PLywood/plywood_diff_2k.jpg',.75)
assign('wod floring','planks/Planks009_2K_Color.png',.89)
plain('white wall',(.84,.81,.78),.97)
plain('Door_material',(.87,.86,.83),.82)
plain('hidden_material',(.85,.83,.80),.91)
plain('PVC',(.13,.14,.14),.67)
plain('Plastic',(.11,.12,.12),.65)
plain('Marble',(.82,.80,.78),.92)
# Window glass: allow the existing backdrop geometry to show through.
plain('Glass',(.92,.95,.96),.08,0.,.12)
mat_by_name['Glass']['alphaMode']='BLEND'
mat_by_name['Glass']['doubleSided']=True
assign('photos','3806.png',.94,0.,512)
assign('display','mpm_F18_desktop_diff.JPG',.4,0.,1024)
assign('Material','Backdrop008_4K_Color.png',1.,0.,2048)
# Additional material maps (when corresponding meshes are available in the full export).
assign('CUSHION','36 fabric/Fabric036_2K_Color.png',.94,0.,512)
assign('pillow 1','36 fabric/Fabric036_2K_Color.png',.95,0.,512)
plain('pillow 2',(.88,.79,.14),.99)
# Metal and black chair details: imported from existing FBX base colors.
# For PBR correctness avoid blue tinted opaque glass and glaring white lights.
plain('glass',(.94,.96,.98),.08,0.,.05)
mat_by_name['glass']['alphaMode']='BLEND'
plain('display frame',(.06,.07,.08),.4)
plain('glass frame',(.045,.05,.06),.2)
plain('box',(.045,.045,.04),.7)

# The FBX export includes Boolean/cutter helper meshes (CTRL_Hole, etc.)
# which should not be drawn in the game. In particular CTRL_Hole fills
# the window opening with an opaque gray polygon.
helper_names={'CTRL_Hole','CTRL_Hole.002','CTRL_Baseboard.001'}
keep=[]
for ni in m['scenes'][m.get('scene',0)]['nodes']:
    node=m['nodes'][ni]
    mesh_idx=node.get('mesh')
    if mesh_idx is None or m['meshes'][mesh_idx].get('name') not in helper_names:
        keep.append(ni)
m['scenes'][m.get('scene',0)]['nodes']=keep
report['removed_helper_mesh_nodes'] = sorted(helper_names)

# Existing UV coordinates for floor/plywood were repaired by v0.2.
# Some materials did not have any UV layer; generate minimally invasive planar UVs
# on specific target meshes only. We don't change already textured originals.
projection={'display':'imac','photos':'small_photo','Material':'panorama'}
for mesh in m['meshes']:
 for prim in mesh['primitives']:
  material=m['materials'][prim['material']]['name']
  if material not in projection: continue
  if 'TEXCOORD_0' in prim['attributes']: continue
  pos=repair.get_array(m,b,prim['attributes']['POSITION'])
  extent=np.ptp(pos,axis=0)
  if material=='display':
   # iMac screen lies near x~0.95; local z is horizontal, y is vertical.
   u=(pos[:,2]-pos[:,2].min())/max(extent[2],1e-6)
   v=(pos[:,1]-pos[:,1].min())/max(extent[1],1e-6)
  elif material=='photos':
   u=(pos[:,2]-pos[:,2].min())/max(extent[2],1e-6)
   v=(pos[:,1]-pos[:,1].min())/max(extent[1],1e-6)
  else:
   # panoramic backdrop: rear curved wall, approximately facing +Z.
   u=(pos[:,0]-pos[:,0].min())/max(extent[0],1e-6)
   v=(pos[:,1]-pos[:,1].min())/max(extent[1],1e-6)
  # glTF UV v=0 is top in glTF-defined texture coordinates.
  # Source GLB uses V flip on standard FBX UV during conversion, map upright.
  uv=np.stack([u,1.-v],axis=1)
  prim['attributes']['TEXCOORD_0']=repair.add_float2(m,b,uv)
  report['planar_projection_meshes'].append({'mesh':mesh['name'],'material':material,'axis_extent':extent.tolist()})

repair.write_glb(OUT,m,b)
report['size_mib']=round(OUT.stat().st_size/1048576,2)
OUT.with_suffix('.source-textures.json').write_text(json.dumps(report,indent=2),encoding='utf8')
print('Wrote',OUT, OUT.stat().st_size)
print('Texture count',len(m['images']),'mapped materials',len(report['materials_with_source_color_textures']))
print('Projections',report['planar_projection_meshes'])
