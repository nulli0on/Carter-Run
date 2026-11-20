#!/usr/bin/env python3
"""Make a rigid-joint skin for the supplied skinless node-animated cardboard GLB.

This retains source mesh coordinates and all original animation samples.
The source file is never modified. It only accepts the verified six-part
hierarchy rather than making dangerous guesses for unrelated GLB models.
"""
import argparse
import json
from pathlib import Path
import struct

J=b"JSON"
B=b"BIN\0"

def read_glb(path):
    blob=Path(path).read_bytes()
    if len(blob)<28 or blob[:4]!=b"glTF": raise ValueError("not a GLB")
    version,length=struct.unpack_from("<II",blob,4)
    if version!=2 or length!=len(blob): raise ValueError("invalid GLB header")
    off=12;chunks=[]
    while off<len(blob):
        if off+8>len(blob): raise ValueError("truncated GLB")
        n,typ=struct.unpack_from("<I4s",blob,off);off+=8
        if off+n>len(blob): raise ValueError("truncated chunk")
        chunks.append((typ,blob[off:off+n]));off+=n
    if len(chunks)!=2 or chunks[0][0]!=J or chunks[1][0]!=B:
        raise ValueError("requires one JSON chunk and one BIN chunk")
    return json.loads(chunks[0][1]),bytearray(chunks[1][1])

def rewrite(source,target):
    obj,data=read_glb(source)
    if obj.get("skins") or not obj.get("animations"):
        raise ValueError("requires skinless, animated source")
    nodes=obj["nodes"]
    targets={ch["target"]["node"] for a in obj["animations"] for ch in a["channels"]}
    parents={}
    for i,node in enumerate(nodes):
        for child in node.get("children",[]):
            if child in parents:raise ValueError("multi-parented node")
            parents[child]=i
    mesh_nodes=[i for i,node in enumerate(nodes) if "mesh" in node]
    joints=[parents.get(i) for i in mesh_nodes]
    if len(mesh_nodes)!=6 or any(i is None for i in joints):
        raise ValueError("unexpected rigid-part hierarchy")
    if len(set(joints))!=6 or sum(j in targets for j in joints)!=5:
        raise ValueError("animated joint hierarchy differs from audited asset")
    roots={parents.get(j) for j in joints}
    if len(roots)!=1 or next(iter(roots)) in targets:
        raise ValueError("moving or inconsistent skeleton root")
    root=next(iter(roots))
    if any("rotation" in nodes[j] or "scale" in nodes[j] for j in joints):
        raise ValueError("requires translation-only bind poses")

    def view(payload,alignment=4):
        while len(data)%alignment:data.append(0)
        offset=len(data);data.extend(payload)
        idx=len(obj["bufferViews"])
        obj["bufferViews"].append({"buffer":0,"byteOffset":offset,"byteLength":len(payload)})
        return idx
    def accessor(v,n,component,kind):
        idx=len(obj["accessors"])
        obj["accessors"].append({"bufferView":v,"byteOffset":0,"componentType":component,"count":n,"type":kind})
        return idx

    transforms=bytearray()
    for j in joints:
        x,y,z=nodes[j].get("translation",[0.,0.,0.])
        transforms.extend(struct.pack("<16f",
            1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., -x,-y,-z,1.))
    inverse=accessor(view(transforms),len(joints),5126,"MAT4")
    obj["skins"]=[{"inverseBindMatrices":inverse,"joints":joints,"skeleton":root}]
    for index,nodeid in enumerate(mesh_nodes):
        node=nodes[nodeid]
        node["skin"]=0
        for prim in obj["meshes"][node["mesh"]]["primitives"]:
            if prim.get("mode",4)!=4 or "POSITION" not in prim["attributes"]:
                raise ValueError("requires triangle position mesh")
            n=obj["accessors"][prim["attributes"]["POSITION"]]["count"]
            if n<=0:raise ValueError("empty mesh")
            prim["attributes"]["JOINTS_0"]=accessor(view(bytes([index,0,0,0])*n),n,5121,"VEC4")
            prim["attributes"]["WEIGHTS_0"]=accessor(view(struct.pack("<4f",1.,0.,0.,0.)*n),n,5126,"VEC4")
    obj["buffers"][0]["byteLength"]=len(data)
    js=json.dumps(obj,ensure_ascii=False,separators=(",",":")).encode("utf8")
    js+=b" "*((-len(js))%4)
    data.extend(b"\0"*((-len(data))%4))
    length=12+8+len(js)+8+len(data)
    out=struct.pack("<4sII",b"glTF",2,length)
    out+=struct.pack("<I4s",len(js),J)+js
    out+=struct.pack("<I4s",len(data),B)+data
    Path(target).write_bytes(out)
    result,_=read_glb(target)
    assert len(result["skins"])==1
    assert len(result["animations"])==len(obj["animations"])
    return {"joint_count":len(joints),"animation_count":len(obj["animations"]),"bytes":len(out)}

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("source")
    p.add_argument("output")
    args=p.parse_args()
    print(json.dumps(rewrite(args.source,args.output),indent=2))
