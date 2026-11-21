#!/usr/bin/env python3
"""Read source glTF animation metadata from weapon GLBs (stdlib only)."""
import json
import pathlib
import struct
import sys

def inspect(path):
    raw = pathlib.Path(path).read_bytes()
    if len(raw) < 20 or raw[:4] != b"glTF":
        raise ValueError(f"{path}: not a GLB container")
    magic, version, declared = struct.unpack_from("<4sII", raw)
    if version != 2 or declared != len(raw):
        raise ValueError(f"{path}: GLB header/length mismatch")
    size, chunk_type = struct.unpack_from("<II", raw, 12)
    if chunk_type != 0x4E4F534A:
        raise ValueError(f"{path}: missing JSON chunk")
    doc = json.loads(raw[20:20+size])
    tracks = []
    for clip in doc.get("animations", []):
        accessors = [doc["accessors"][s["input"]] for s in clip["samplers"]]
        starts = [a["min"][0] for a in accessors if "min" in a]
        ends = [a["max"][0] for a in accessors if "max" in a]
        tracks.append({
            "name": clip.get("name","<unnamed>"),
            "start_seconds": round(min(starts), 4) if starts else None,
            "end_seconds": round(max(ends), 4) if ends else None,
            "channels": len(clip.get("channels",[]))
        })
    return {"asset":pathlib.Path(path).name, "clips":tracks}

if __name__ == "__main__":
    paths = [p for arg in sys.argv[1:] for p in
             (sorted(pathlib.Path(arg).glob("*.glb")) if pathlib.Path(arg).is_dir() else [pathlib.Path(arg)])]
    if not paths:
        raise SystemExit("Usage: python3 tools/inspect_glb_animations.py assets/arsenal")
    out = [inspect(p) for p in paths]
    print(json.dumps({"models":out,"with_clips":sum(bool(x["clips"]) for x in out)},indent=2))
