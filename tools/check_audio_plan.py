#!/usr/bin/env python3
"""Verify the repository's planned sound-cue inventory, without claiming binaries exist."""
import csv
from pathlib import Path
p=Path(__file__).resolve().parents[1]/"docs/AUDIO_INVENTORY_100.csv"
with p.open(encoding="utf-8",newline="") as f: rows=list(csv.DictReader(f))
assert len(rows)==100,len(rows)
assert len(set(r["cue"] for r in rows))==100
counts={status:sum(x["status"]==status for x in rows) for status in ("SUPPLIED_ORIGINAL","CANDIDATE_REVIEW","MISSING")}
assert counts=={"SUPPLIED_ORIGINAL":25,"CANDIDATE_REVIEW":5,"MISSING":70},counts
for r in rows:
    if r["status"]=="MISSING":assert not r["source_candidate"]
    else: assert r["source_candidate"]
print("PASS: 100 planned, 25 originals, 5 candidates, 70 MISSING — binary availability NOT checked")
