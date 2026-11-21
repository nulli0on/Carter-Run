"""Validate original Figma/SVG extraction without claiming runtime fidelity."""
from pathlib import Path
from PIL import Image
import json, hashlib, sys
ROOT=Path(__file__).resolve().parents[1]; assets=ROOT/'assets/carteros'
manifest=json.loads((assets/'asset_manifest.json').read_text())
assert len(manifest)==21, f'Expected 21 separate source asset layers, got {len(manifest)}'
for name,data in manifest.items():
    p=assets/data['file'];assert p.exists(),p
    with Image.open(p) as im:
        assert im.format=='PNG' and im.mode=='RGBA',p
        assert im.size==(data['width'],data['height']),(name,im.size)
        assert im.getbbox()==(0,0,im.width,im.height),name
        assert data['x']>=0 and data['y']>=0
        assert data['x']+im.width<=1440 and data['y']+im.height<=900
source=ROOT/'design_reference/CarterOS_original_export.svg'
assert source.exists() and b'6120' in source.read_bytes()[:500]
fig=ROOT/'design_reference/CarterOS_original.fig.zip';assert fig.exists()
code=(ROOT/'src/carteros.rs').read_text()
for name in manifest:assert f'("{name}",' in code, name
full=list(assets.glob('full_frame_*.png'))
assert len(full)==12, len(full)
for p in full:
    with Image.open(p) as im:
        assert im.size==(1440,900) and im.format=='PNG',p
        assert f'("{p.stem}",' in code,p.stem
assert 'mod carteros;' in (ROOT/'src/main.rs').read_text()
print('PASS: 21 split Figma layers + 12 full original artboards; all aligned and referenced by Rust')
print('Original SVG sha256:',hashlib.sha256(source.read_bytes()).hexdigest())
print('Original fig sha256:',hashlib.sha256(fig.read_bytes()).hexdigest())
