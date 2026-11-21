#!/usr/bin/env python3
"""Strict source and referenced audio-file checks; NOT a runtime Rust compiler."""
from pathlib import Path
import re
import json
R=Path(__file__).resolve().parents[1]
source=(R/'src/main.rs').read_text(encoding='utf-8')
module=(R/'src/audio_events.rs').read_text(encoding='utf-8')
assert 'mod audio_events;' in source
assert 'audio_events:audio_events::Queue' in source
assert 'game.audio_events.advance(dt)' in source
assert 'let frame_cues=game.audio_events.take()' in source
assert 'if os.system.speakers {' in source
assert 'if is_ui && !os.system.system_sounds' in source
assert 'fn sound_events_follow_real_actions_not_fake_reloads()' in source
assert module.count('#[test]')==6
for cue in ('MailHandled','ShopPurchase','ShopDenied','Footstep','DoorPound','WeaponFire','WaveStart','WaveEnd','PlayerCaught','SugarPickup','Detonation','Delivery','WeaponDraw'):
    assert f'Cue::{cue}' in source, cue
# The actual filename spellings are verified rather than substituting sounds.
paths=re.findall(r'new_sound\("([^"]+)"\)',source)
assert len(paths)>=17,paths
for relative in paths:
    assert (R/relative).is_file(),f'MISSING actual sound: {relative}'
manifest=json.loads((R/'assets/sfx/audio_inventory.json').read_text())
assert manifest['total_planned']==100
assert sum(c['status']=='MISSING' for c in manifest['cues'])==70
print(f'PASS: audio pipeline references {len(paths)} existing sound files, 6 module tests, 70 unresolved cues')
