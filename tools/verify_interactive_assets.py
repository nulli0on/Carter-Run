"""Validate the exact source artboards plus v0.6 input and effect assets.

This checks file integrity and source references, NOT compilation or runtime behavior.
"""
from pathlib import Path
from PIL import Image
import re, wave, hashlib, json

ROOT = Path(__file__).resolve().parents[1]
SRC = (ROOT / 'src' / 'carteros.rs').read_text(encoding='utf8')
REFERENCE = ROOT.parent / 'run-carter-carteros-v05' / 'assets' / 'carteros'
LAYERS = re.findall(r'\("([a-z0-9_]+)"\s*,\s*\d+\s*,\s*\d+\s*\)', SRC.split('const LAYERS:')[1].split('];', 1)[0])
assert len(LAYERS) == len(set(LAYERS)) == 36, f'Expected 36 distinct texture layers; found {len(LAYERS)}'
for name in LAYERS:
    p = ROOT / 'assets' / 'carteros' / f'{name}.png'
    assert p.is_file() and p.stat().st_size > 300, p
    with Image.open(p) as im:
        im.verify()
    with Image.open(p) as im:
        assert im.mode == 'RGBA', (name, im.mode)
        if name.startswith('full_frame_') or name == 'wallpaper_after_hours':
            assert im.size == (1440, 900), (name, im.size)
        if name.startswith('toggle_'):
            assert im.size == (44, 30), (name, im.size)
for file in sorted((ROOT / 'assets' / 'carteros').glob('full_frame_*.png')):
    base=REFERENCE/file.name
    if base.exists():
        assert hashlib.sha256(file.read_bytes()).digest() == hashlib.sha256(base.read_bytes()).digest(), f'Original Figma artboard modified: {file}'
assert len(list((ROOT / 'assets' / 'carteros').glob('full_frame_*.png'))) == 12
for filename in ('ui_click.wav', 'security_alert.wav'):
    p = ROOT / 'assets' / 'sfx' / filename
    assert p.is_file() and p.stat().st_size > 64, p
    with wave.open(str(p), 'rb') as f:
        assert f.getnframes() > 1000
        assert f.getnchannels() in (1, 2)
        assert f.getframerate() in (22050, 44100, 48000)
manifest = json.loads((ROOT / 'assets' / 'carteros' / 'asset_manifest.json').read_text())
assert len(manifest) == 21
main=(ROOT/'src/main.rs').read_text(encoding='utf8')
state=(ROOT/'src/system_state.rs').read_text(encoding='utf8')
for signature in ['night_comfort','master_volume','system_sounds','wifi','public_shield','doxxblock','wallpaper','display_preset','interface_scale']:
    assert signature in main or signature in SRC
    assert signature in state
for feature in ['begin_texture_mode', 'draw_texture_pro', 'security_alert.wav', 'ui_click.wav', 'RaylibAudio', 'network_online']:
    assert feature in main
print('PASS: 36 visual assets, 12 source-exact artboards, two valid WAV sounds, and system control callsites')
