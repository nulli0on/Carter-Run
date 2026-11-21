#!/usr/bin/env python3
"""Small, source-specific FBX 7.4 binary -> static glTF/GLB converter.

Reads the uploaded office FBX, preserves embedded diffuse textures and diffuse-color
materials. Skips extremely heavy sub-meshes in 'lite' mode for weak hardware.
This is intentionally not a full FBX importer: animation, layered textures and
advanced materials are outside this first prototype's scope.

Usage: python tools/fbx_to_glb.py --input '/path/to/office 3.fbx' --output assets/office_lite.glb
Requires: Python 3.9+, numpy, scipy, Pillow.
"""
from __future__ import annotations
import argparse, collections, io, json, math, mmap, os, struct, sys, zlib
from pathlib import Path
import numpy as np
from scipy.spatial.transform import Rotation
from PIL import Image, ImageOps

class FBX:
    class Node:
        __slots__ = ('name','start','end','prop_pos','count','prop_len','children')
        def __init__(self,name,start,end,prop_pos,count,prop_len):
            self.name=name;self.start=start;self.end=end;self.prop_pos=prop_pos
            self.count=count;self.prop_len=prop_len;self.children=[]
        def child(self,name): return next((c for c in self.children if c.name==name),None)
    def __init__(self,path):
        self.file=open(path,'rb');self.mm=mmap.mmap(self.file.fileno(),0,access=mmap.ACCESS_READ)
        if self.mm[:20]!=b'Kaydara FBX Binary  ': raise ValueError('Expected binary FBX')
        self.version=struct.unpack_from('<I',self.mm,23)[0]
        self.header=struct.Struct('<QQQB' if self.version>=7500 else '<IIIB')
        self.roots=self._parse(27)
    def _parse(self,pos,limit=None):
        output=[];s=self.header;data=self.mm
        while pos+s.size<=len(data):
            end,count,plen,nlen=s.unpack_from(data,pos)
            if end==0:break
            name=data[pos+s.size:pos+s.size+nlen].decode('utf8','replace')
            start=pos+s.size+nlen
            n=self.Node(name,pos,end,start,count,plen)
            cp=start+plen
            if cp+s.size<end:n.children=self._parse(cp,end)
            output.append(n);pos=end
            if limit and pos>=limit:break
        return output
    def node(self,name):return next((c for c in self.roots if c.name==name),None)
    def props(self,node,arrays=False):
        if node is None:return []
        out=[];pos=node.prop_pos;mm=self.mm
        for _ in range(node.count):
            typ=chr(mm[pos]);pos+=1
            if typ in ('S','R'):
                ln=struct.unpack_from('<I',mm,pos)[0];pos+=4
                b=mm[pos:pos+ln];pos+=ln
                out.append(b.decode('utf-8','replace') if typ=='S' else (bytes(b) if arrays else ('blob',ln)))
            elif typ in ('L','D','I','F','Y','C'):
                fmt={'L':'q','D':'d','I':'i','F':'f','Y':'h','C':'?'}[typ]
                out.append(struct.unpack_from('<'+fmt,mm,pos)[0]);pos+=struct.calcsize(fmt)
            elif typ in 'fdilbc':
                n,enc,size=struct.unpack_from('<III',mm,pos);pos+=12
                if arrays:
                    b=mm[pos:pos+size]
                    if enc:b=zlib.decompress(b)
                    dt={'f':'<f4','d':'<f8','i':'<i4','l':'<i8','b':'i1','c':'i1'}[typ]
                    out.append(np.frombuffer(b,dtype=dt,count=n).copy())
                else:out.append((typ,n,enc,size))
                pos+=size
            else:raise ValueError(f'Unknown FBX prop {typ!r} in {node.name}')
        return out
    def named(self,node,name,arrays=False):
        return self.props(node.child(name),arrays=arrays) if node else []
    def array(self,node,name):
        p=self.named(node,name,True)
        return p[0] if p else None
    def close(self):self.mm.close();self.file.close()

class GLB:
    def __init__(self):
        self.buff=bytearray();self.json={'asset':{'version':'2.0','generator':'RUN CARTER basic FBX importer'},'scene':0,'scenes':[{'nodes':[]}], 'nodes':[], 'meshes':[], 'materials':[], 'images':[], 'textures':[], 'samplers':[{'magFilter':9729,'minFilter':9987,'wrapS':10497,'wrapT':10497}], 'accessors':[], 'bufferViews':[], 'buffers':[{'byteLength':0}]}
    def view(self,data,target=None):
        while len(self.buff)%4:self.buff.append(0)
        offset=len(self.buff);self.buff.extend(data)
        d={'buffer':0,'byteOffset':offset,'byteLength':len(data)}
        if target:d['target']=target
        self.json['bufferViews'].append(d)
        return len(self.json['bufferViews'])-1
    def accessor(self,arr,comp,typ,target=None):
        arr=np.ascontiguousarray(arr)
        vi=self.view(arr.tobytes(),target)
        d={'bufferView':vi,'componentType':comp,'count':len(arr),'type':typ}
        if typ=='VEC3':
            d['min']=arr.min(axis=0).astype(float).tolist();d['max']=arr.max(axis=0).astype(float).tolist()
        self.json['accessors'].append(d)
        return len(self.json['accessors'])-1
    def save(self,path):
        self.json['buffers'][0]['byteLength']=len(self.buff)
        j=json.dumps(self.json,separators=(',',':'),ensure_ascii=False).encode('utf-8')
        j+=b' ' * ((-len(j))%4)
        b=bytes(self.buff)+b'\0'*((-len(self.buff))%4)
        with open(path,'wb') as out:
            out.write(b'glTF');out.write(struct.pack('<II',2,12+8+len(j)+8+len(b)))
            out.write(struct.pack('<I4s',len(j),b'JSON'));out.write(j)
            out.write(struct.pack('<I4s',len(b),b'BIN\0'));out.write(b)

def model_props(f,n):
    p=n.child('Properties70')
    if not p:return {}
    return {r[0]:r[4:] for c in p.children if (r:=f.props(c)) and len(r)>=5}

def mat4(t=(0,0,0),r=(0,0,0),s=(1,1,1)):
    m=np.eye(4,dtype=np.float64)
    m[:3,:3]=Rotation.from_euler('xyz',r,degrees=True).as_matrix() @ np.diag(s)
    m[:3,3]=t
    return m

def texture_image(f,video,all_videos,maxsize,images,report):
    # A few Video nodes in this FBX are duplicate references without a Content node.
    # Recover those by filename from another embedded Video, if possible.
    n=video.child('Content')
    raw=f.props(n,arrays=True)[0] if n else None
    filename=(f.named(video,'RelativeFilename') or [''])[0].replace('\\','/').split('/')[-1]
    if raw is None:
        raw=all_videos.get(filename.lower())
    if raw is None:
        report['unavailable_embedded_images'].append(filename)
        return None
    try:
        with Image.open(io.BytesIO(raw)) as base:
            image=ImageOps.exif_transpose(base)
            if max(image.size)>maxsize:
                image.thumbnail((maxsize,maxsize),Image.Resampling.LANCZOS)
            output=io.BytesIO()
            if 'A' in image.getbands() or 'transparency' in image.info:
                image.convert('RGBA').save(output,'PNG',optimize=True)
                mimetype='image/png'
            else:
                image.convert('RGB').save(output,'JPEG',quality=80,optimize=True,progressive=False)
                mimetype='image/jpeg'
        idx=len(images.json['images'])
        images.json['images'].append({'name':filename,'bufferView':images.view(output.getvalue()),'mimeType':mimetype})
        images.json['textures'].append({'source':idx,'sampler':0})
        return idx
    except Exception as e:
        report['unavailable_embedded_images'].append(f'{filename}: {e}')
        return None

def convert(infile,outfile,budget=120000,maxsize=512,full=False,overrides_file=None):
    f=FBX(infile);glb=GLB()
    overrides=json.loads(Path(overrides_file).read_text(encoding='utf8')) if overrides_file and Path(overrides_file).is_file() else {}
    material_fixes=overrides.get('model_materials',{})
    texture_fixes=overrides.get('material_textures',{})
    obj=f.node('Objects'); con=f.node('Connections')
    items={f.props(n)[0]:n for n in obj.children}
    idprops={i:f.props(n) for i,n in items.items()}
    idname={i:p[1].split('\0')[0] for i,p in idprops.items()}
    models={i:n for i,n in items.items() if n.name=='Model'}
    geometry={i:n for i,n in items.items() if n.name=='Geometry'}
    materials={i:n for i,n in items.items() if n.name=='Material'}
    textures={i:n for i,n in items.items() if n.name=='Texture'}
    videos={i:n for i,n in items.items() if n.name=='Video'}
    mats_by_model=collections.defaultdict(list)
    geo_by_model={};parent={};texture_by_mat=collections.defaultdict(list);video_by_texture={}
    for c in con.children:
        p=f.props(c)
        if len(p)<3:continue
        kind,src,dst=p[:3]
        ss=items[src].name if src in items else '?'
        dd=items[dst].name if dst in items else '?'
        if kind=='OO' and ss=='Material' and dd=='Model':mats_by_model[dst].append(src)
        if kind=='OO' and ss=='Geometry' and dd=='Model':geo_by_model[dst]=src
        if kind=='OO' and ss=='Model' and dd=='Model':parent[src]=dst
        if kind=='OP' and ss=='Texture' and dd=='Material':texture_by_mat[dst].append((src,p[3] if len(p)>3 else ''))
        if kind=='OO' and ss=='Video' and dd=='Texture':video_by_texture[dst]=src
    report={'input':str(infile),'fbx_version':f.version,'imported_meshes':[], 'skipped_heavy_meshes':[], 'models_without_material':[], 'unavailable_embedded_images':[], 'used_texture_images':[], 'notes':['Static geometry only; basic FBX transforms, diffuse color, diffuse image. No animation / PBR channels / collision mesh.']}
    all_embedded={}
    for v in videos.values():
        name=(f.named(v,'RelativeFilename') or [''])[0].replace('\\','/').split('/')[-1].lower()
        if v.child('Content') and name and name not in all_embedded:
            all_embedded[name]=f.props(v.child('Content'),arrays=True)[0]
    material_index={}
    image_index={}
    image_name_to_video={name:vid for vid,v in videos.items() for name in [(f.named(v,'RelativeFilename') or [''])[0].replace('\\','/').split('/')[-1].lower()] if name in all_embedded}
    for mid,ma in materials.items():
        p=model_props(f,ma)
        color=p.get('DiffuseColor',p.get('Color',[0.7,0.7,0.7]))[:3]
        rgba=[max(0,min(1,float(x))) for x in color]+[1.0]
        name=idname[mid]
        desc={'name':name,'doubleSided':True,'pbrMetallicRoughness':{'baseColorFactor':rgba,'metallicFactor':0.0,'roughnessFactor':0.95}}
        baseid=next((tid for tid,channel in texture_by_mat[mid] if channel=='DiffuseColor'),None)
        vid=image_name_to_video.get(texture_fixes[name].lower()) if name in texture_fixes else video_by_texture.get(baseid)
        if vid in videos:
                if vid not in image_index:
                    image_index[vid]=texture_image(f,videos[vid],all_embedded,maxsize,glb,report)
                iidx=image_index[vid]
                if iidx is not None:
                    desc['pbrMetallicRoughness']['baseColorTexture']={'index':iidx}
                    desc['pbrMetallicRoughness']['baseColorFactor']=[1.0,1.0,1.0,1.0]
                    report['used_texture_images'].append(idname[vid])
                    if glb.json['images'][iidx]['mimeType']=='image/png':
                        desc['alphaMode']='MASK';desc['alphaCutoff']=0.35
        material_index[mid]=len(glb.json['materials']);glb.json['materials'].append(desc)
    fallback=len(glb.json['materials'])
    glb.json['materials'].append({'name':'AUTO_FALLBACK__unassigned','doubleSided':True,'pbrMetallicRoughness':{'baseColorFactor':[0.54,0.56,0.58,1.0],'metallicFactor':0,'roughnessFactor':1}})
    cache={}
    def world(modelid,stack=None):
        if modelid in cache:return cache[modelid]
        if stack is None:stack=set()
        if modelid in stack:return np.eye(4)
        stack.add(modelid)
        p=model_props(f,models[modelid])
        local=mat4(p.get('Lcl Translation',[0,0,0])[:3],p.get('Lcl Rotation',[0,0,0])[:3],p.get('Lcl Scaling',[1,1,1])[:3])
        if modelid in parent:local=world(parent[modelid],stack)@local
        cache[modelid]=local
        stack.remove(modelid)
        return local
    totaltris=0;bounding_min=np.array([float('inf')]*3);bounding_max=np.array([-float('inf')]*3)
    for modelid,geoid in geo_by_model.items():
        geo=geometry[geoid]
        count=f.props(geo.child('PolygonVertexIndex'))[0][1]
        name=idname[modelid]
        # Large meshes are disproportionately detailed plants, a chair, an iMac,
        # and textiles. Preserve the computer as it is central to the concept.
        if not full and count>budget and not name.startswith('Apple iMac'):
            report['skipped_heavy_meshes'].append({'name':name,'polygon_corners':count})
            continue
        inds=f.array(geo,'PolygonVertexIndex')
        coords=f.array(geo,'Vertices')
        if inds is None or coords is None or len(inds)==0:continue
        pts=coords.reshape((-1,3))
        prop=model_props(f,models[modelid])
        geommat=mat4(prop.get('GeometricTranslation',[0,0,0])[:3],prop.get('GeometricRotation',[0,0,0])[:3],prop.get('GeometricScaling',[1,1,1])[:3])
        transform=world(modelid)@geommat
        # FBX uses centimetres in this asset; convert world coordinates to metres.
        xyz=(pts@transform[:3,:3].T+transform[:3,3])*0.01
        normal_transform=np.linalg.pinv(transform[:3,:3]).T
        normal_layer=geo.child('LayerElementNormal')
        raw_norm=f.array(normal_layer,'Normals') if normal_layer else None
        if raw_norm is not None:
            raw_norm=raw_norm.reshape((-1,3))@normal_transform.T
            normlen=np.linalg.norm(raw_norm,axis=1,keepdims=True)
            raw_norm=np.divide(raw_norm,normlen,out=np.zeros_like(raw_norm),where=normlen>1e-12)
        uv_layer=geo.child('LayerElementUV')
        uv=f.array(uv_layer,'UV').reshape((-1,2)) if uv_layer else None
        uv_idx=f.array(uv_layer,'UVIndex') if uv_layer else None
        me=geo.child('LayerElementMaterial')
        matmap=(f.named(me,'MappingInformationType') or ['AllSame'])[0] if me else 'AllSame'
        mat_indices=f.array(me,'Materials') if me else None
        mlist=mats_by_model[modelid][:]
        if name in material_fixes:
            target=next((mid for mid in materials if idname[mid]==material_fixes[name]),None)
            if target is not None:mlist=[target]
            else:print(f'WARNING: override for {name} names unknown material {material_fixes[name]}')
        if not mlist:report['models_without_material'].append(name)
        # Build material submesh with shared corner vertices (position + UV + flat/smooth normal).
        groups={}
        def emit(group,point_index,loop_index):
            if raw_norm is not None and len(raw_norm)>loop_index:
                nn=raw_norm[loop_index];nk=tuple((nn*10000).astype(int).tolist())
            else:
                nn=np.array([0.,1.,0.]);nk=(0,10000,0)
            uvi=int(uv_idx[loop_index]) if uv_idx is not None and len(uv_idx)>loop_index else -1
            key=(point_index,uvi,nk)
            i=group['mapping'].get(key)
            if i is None:
                i=len(group['positions']);group['mapping'][key]=i
                group['positions'].append(xyz[point_index].astype(np.float32))
                group['normals'].append(nn.astype(np.float32))
                group['uvs'].append(np.array([float(uv[uvi][0]),1-float(uv[uvi][1])],dtype=np.float32) if uv is not None and 0<=uvi<len(uv) else np.array([0.,0.],dtype=np.float32))
            group['indices'].append(i)
        polygon=[];start=0;polyidx=0;icount=0
        for loc,raw in enumerate(inds):
            raw=int(raw);closed=raw<0
            polygon.append(((-raw-1) if closed else raw, loc))
            if closed:
                ix=int(mat_indices[min(polyidx,len(mat_indices)-1)]) if mat_indices is not None and len(mat_indices)>0 and matmap=='ByPolygon' else int(mat_indices[0]) if mat_indices is not None and len(mat_indices)>0 else 0
                material=material_index[mlist[ix]] if 0<=ix<len(mlist) else fallback
                group=groups.setdefault(material,{'mapping':{},'positions':[],'normals':[],'uvs':[],'indices':[]})
                if len(polygon)>=3:
                    for corner in range(1,len(polygon)-1):
                        # Flip winding if the transform mirrors the mesh.
                        tri=[polygon[0],polygon[corner],polygon[corner+1]]
                        if np.linalg.det(transform[:3,:3])<0:tri.reverse()
                        for idx,li in tri:emit(group,idx,li)
                        icount+=1
                polygon=[];polyidx+=1
        prim=[]
        for mat,g in groups.items():
            if not g['positions']:continue
            arrpos=np.stack(g['positions']).astype('<f4');arrnor=np.stack(g['normals']).astype('<f4');arruv=np.stack(g['uvs']).astype('<f4');indexes=np.asarray(g['indices'],dtype='<u4')
            pidx=glb.accessor(arrpos,5126,'VEC3',34962)
            nidx=glb.accessor(arrnor,5126,'VEC3',34962)
            uidx=glb.accessor(arruv,5126,'VEC2',34962)
            iidx=glb.accessor(indexes,5125,'SCALAR',34963)
            prim.append({'attributes':{'POSITION':pidx,'NORMAL':nidx,'TEXCOORD_0':uidx},'indices':iidx,'material':mat,'mode':4})
            bounding_min=np.minimum(bounding_min,arrpos.min(axis=0));bounding_max=np.maximum(bounding_max,arrpos.max(axis=0))
        if prim:
            meshno=len(glb.json['meshes']);glb.json['meshes'].append({'name':name,'primitives':prim})
            no=len(glb.json['nodes']);glb.json['nodes'].append({'name':name,'mesh':meshno});glb.json['scenes'][0]['nodes'].append(no)
            totaltris+=icount
            report['imported_meshes'].append({'name':name,'triangles':icount,'materials':len(prim)})
        print(f'Converted {name:36} {icount:>7,} triangles',flush=True)
    if not glb.json['meshes']:raise RuntimeError('No model geometry converted')
    outfile=Path(outfile);outfile.parent.mkdir(parents=True,exist_ok=True)
    glb.save(outfile)
    report.update({'triangle_count':totaltris,'mesh_count':len(glb.json['meshes']),'texture_count':len(glb.json['images']),'output_size_mb':round(outfile.stat().st_size/1048576,2),'world_bounds_m':{'min':bounding_min.tolist(),'max':bounding_max.tolist()}})
    jsonpath=outfile.with_suffix('.report.json');jsonpath.write_text(json.dumps(report,indent=2),encoding='utf-8')
    print('\nConverted:',outfile,'size MiB:',report['output_size_mb'],'triangles:',totaltris,'textures:',report['texture_count'])
    print('Bounds in metres:',report['world_bounds_m']);print('Report:',jsonpath)
    f.close()

if __name__=='__main__':
    a=argparse.ArgumentParser(description=__doc__)
    a.add_argument('--input',required=True)
    a.add_argument('--output',default='assets/office_lite.glb')
    a.add_argument('--budget',type=int,default=120000,help='Skip meshes with more than this many polygon corners (except iMac)')
    a.add_argument('--texture-max',type=int,default=512,help='Maximum embedded texture dimension in pixels')
    a.add_argument('--full',action='store_true',help='Include ALL detailed meshes; not recommended for low-end PCs')
    a.add_argument('--overrides',default=None,help='JSON mapping of model/material texture overrides')
    args=a.parse_args();convert(args.input,args.output,args.budget,args.texture_max,args.full,args.overrides)
