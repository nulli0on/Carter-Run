# RUN CARTER v0.13 — QA evidence

- Original v0.12 assets compared byte-for-byte against v0.13: 94/94 identical (including Figma images, original design files, models and MP3 recordings).
- `tools/verify_audio_inventory.py`: PASS (100 cue slots, 34 hashed recordings, 70 missing, 5 candidates).
- `tools/verify_figma_fidelity.py`: PASS (21 split layers and 12 original screen exports).
- `tools/verify_arsenal_assets.py`: PASS (15 unique hashed original GLB files).
- `tools/verify_v10.py`: PASS (19 achievements and event hooks).
- `tools/verify_v09.py`: PASS (15 cheat URLs and browser/mail integration).
- GitHub `rustc --test src/audio_events.rs`: **6 tests PASS**: initial event, keyboard debounce, weapon frequency, footstep debounce, invalid/negative time, bounded queue.
- `tools/verify_v13_audio.py`: local source/manifest checks PASS.

**Unverified:** Main game compile, raylib lifetime/API compatibility, runtime asset loading, sound duration and balancing, in-game 3D/NPC behavior, Steam achievement delivery. GitHub full-build workflow still skips without `Cargo.toml` and `src/main.rs`.
