#!/usr/bin/env python3
"""Validate RUN CARTER's 100-cue audio inventory and bundled recordings."""
from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "assets/sfx/audio_inventory.json"


def validate():
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    assert len(data["cues"]) == data["total_planned"] == 100
    counts = {status: sum(c["status"] == status for c in data["cues"])
              for status in ("SUPPLIED_ORIGINAL", "CANDIDATE_REVIEW", "MISSING")}
    assert counts == {"SUPPLIED_ORIGINAL": 25, "CANDIDATE_REVIEW": 5, "MISSING": 70}, counts
    assert len(data["recordings"]) == 34
    known = {r["filename"] for r in data["recordings"]}
    for row in data["cues"]:
        assert row["status"] in counts
        assert row["source_file"] in known if row["source_file"] else row["status"] == "MISSING"
    for recording in data["recordings"]:
        f = ROOT / "assets/sfx/imported" / recording["filename"]
        assert f.is_file(), f"MISSING bundled asset: {f}"
        assert f.stat().st_size == recording["bytes"], f
        assert hashlib.sha256(f.read_bytes()).hexdigest() == recording["sha256"], f
    print("PASS: 100 sound cue slots / 34 hashed MP3s / 70 missing / 5 candidates")


if __name__ == "__main__":
    try:
        validate()
    except Exception as exc:
        print(f"AUDIO AUDIT FAILED: {exc}", file=sys.stderr)
        raise
