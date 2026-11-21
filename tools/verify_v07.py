#!/usr/bin/env python3
"""Asset and static wiring verification, NOT a Rust compilation or play test."""
from pathlib import Path
import hashlib, sys
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
# Retained hashes come from v0.6 and make this check independent of extraction location.
import json
HASHES=json.loads((ROOT/'tools/source_asset_hashes.json').read_text())
errors=[]
for relative,digest in HASHES.items():
    p=ROOT/relative
    if not p.is_file():errors.append(f'missing original asset {relative}');continue
    if hashlib.sha256(p.read_bytes()).hexdigest()!=digest:errors.append(f'CHANGED ORIGINAL: {relative}')
    if p.suffix=='.png':
        try:
            with Image.open(p) as im:im.verify()
        except Exception as exc:errors.append(f'invalid PNG {relative}: {exc}')
model=ROOT/'assets/office_original_textures_complete.glb'
original_assets=list((ROOT/'assets/carteros').glob('*.png'))
source=(ROOT/'src/carteros.rs').read_text()
main=(ROOT/'src/main.rs').read_text()
drive=(ROOT/'src/drive.rs').read_text()
for feature in ('fn create_file','fn save_editor','fn minimize_active','fn cycle_active','fn load_positions','fn save_positions',
                'file_search_active','fn inform_background','fn draw_window','fn handle_custom_app'):
    if feature not in source:errors.append(f'missing OS feature: {feature}')
for feature in ('fn save(','fn load(','fn rename(','fn trash(','fn set_content('):
    if feature not in drive:errors.append(f'missing virtual drive feature: {feature}')
for key in ('find:','save:','new_file:','new_folder:','cycle_window:','toggle_trash:','ui_pan_x'):
    if key not in main:errors.append(f'unwired input: {key}')
if (ROOT/'assets/sorryos').exists():errors.append('old SorryOS assets still shipped')
if len(original_assets)!=36:errors.append(f'wrong CarterOS PNG count: {len(original_assets)}')
# Source bracket balance is structural only and deliberately not claimed as Rust syntax verification.
print(f'Original visual assets: {len(original_assets)}; office GLB: {model.stat().st_size:,} bytes')
print('Figma reference hashes:',hashlib.sha256((ROOT/'design_reference/CarterOS_original_export.svg').read_bytes()).hexdigest()[:16],
      hashlib.sha256((ROOT/'design_reference/CarterOS_original.fig.zip').read_bytes()).hexdigest()[:16])
if errors:
    for err in errors:print('FAIL:',err,file=sys.stderr)
    sys.exit(1)
print('PASS: v0.7 source wiring and PNG/model byte-identity checks (NOT compilation)')
