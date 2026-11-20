#!/usr/bin/env python3
"""Preserve original GLB while adding a PBR fallback for KHR_specularGlossiness.

raylib 6 imports pbrMetallicRoughness.baseColorTexture but not the source
model's legacy specular/glossiness diffuse map. This derived copy only edits
GLB JSON material definitions; BIN textures, meshes and animation bytes stay
unchanged. Usage:
    python3 tools/convert_specgloss_materials.py assets/arsenal/m4.glb assets/arsenal/m4_raylib.glb
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import tempfile

def read_glb(path):
    blob=Path(path).read_bytes()
    if len(blob)<28 or blob[:4]!=b"glTF":raise ValueError("Not a glTF 2.0 binary")
    version,length=struct.unpack_from("<II",blob,4)
    if version!=2 or length!=len(blob):raise ValueError("Invalid GLB length/version")
    chunks=[];off=12
    while off<len(blob):
        if off+8>len(blob):raise ValueError("Truncated chunk header")
        n,typ=struct.unpack_from("<I4s",blob,off);off+=8
        if off+n>len(blob):raise ValueError("Truncated GLB chunk")
        chunks.append((typ,blob[off:off+n]));off+=n
    if len(chunks)!=2 or chunks[0][0]!=b"JSON" or chunks[1][0]!=b"BIN\0":
        raise ValueError("Expected JSON and embedded BIN chunks")
    return json.loads(chunks[0][1]),chunks[1][1]

def convert(source,dest):
    source=Path(source);dest=Path(dest)
    if source.resolve()==dest.resolve():raise ValueError("Original must not be overwritten")
    before=source.read_bytes()
    obj,bin_chunk=read_glb(source)
    updated=0
    for material in obj.get("materials",[]):
        ext=material.get("extensions",{}).get("KHR_materials_pbrSpecularGlossiness")
        if ext is None or "baseColorTexture" in material.get("pbrMetallicRoughness",{}):continue
        pbr={"baseColorFactor":ext.get("diffuseFactor",[1.,1.,1.,1.]),
             "metallicFactor":0.0,"roughnessFactor":0.75}
        if "diffuseTexture" in ext:pbr["baseColorTexture"]=ext["diffuseTexture"]
        material["pbrMetallicRoughness"]=pbr
        updated+=1
    if not updated:raise ValueError("No specular/glossiness material requires fallback")
    serialized=json.dumps(obj,separators=(",",":"),ensure_ascii=False).encode("utf8")
    serialized+=b" "*((-len(serialized))%4)
    blob=(struct.pack("<4sII",b"glTF",2,12+8+len(serialized)+8+len(bin_chunk))
       +struct.pack("<I4s",len(serialized),b"JSON")+serialized
       +struct.pack("<I4s",len(bin_chunk),b"BIN\0")+bin_chunk)
    dest.parent.mkdir(parents=True,exist_ok=True)
    dest.write_bytes(blob)
    result,after_bin=read_glb(dest)
    assert after_bin==bin_chunk
    assert len(result.get("animations",[]))==len(obj.get("animations",[]))
    assert len(result.get("meshes",[]))==len(obj.get("meshes",[]))
    assert source.read_bytes()==before
    return {"materials_updated":updated,"derived_sha256":hashlib.sha256(blob).hexdigest(),
            "source_sha256":hashlib.sha256(before).hexdigest()}

def self_test():
    obj={"asset":{"version":"2.0"},"buffers":[{"byteLength":4}],
         "materials":[{"extensions":{"KHR_materials_pbrSpecularGlossiness":
             {"diffuseFactor":[0.3,0.4,0.5,1.0],"diffuseTexture":{"index":0}}}}],
         "animations":[{"name":"preserved"}],"meshes":[{"name":"preserved"}]}
    j=json.dumps(obj).encode();j+=b" "*((-len(j))%4);b=b"\0\0\0\0"
    raw=(struct.pack("<4sII",b"glTF",2,12+8+len(j)+8+len(b))
        +struct.pack("<I4s",len(j),b"JSON")+j
        +struct.pack("<I4s",len(b),b"BIN\0")+b)
    with tempfile.TemporaryDirectory() as tmp:
        src=Path(tmp)/"original.glb";dst=Path(tmp)/"derived.glb";src.write_bytes(raw)
        report=convert(src,dst);a,bb=read_glb(dst)
        assert src.read_bytes()==raw and bb==b and report["materials_updated"]==1
        assert a["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]=={"index":0}
        assert a["animations"]==obj["animations"]
    print("PASS: original preserved; derived PBR fallback retains texture reference and animations")

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--self-test",action="store_true")
    p.add_argument("source",nargs="?");p.add_argument("output",nargs="?")
    a=p.parse_args()
    if a.self_test:self_test()
    elif a.source and a.output:print(json.dumps(convert(a.source,a.output),indent=2))
    else:p.error("source and output are required unless --self-test")
