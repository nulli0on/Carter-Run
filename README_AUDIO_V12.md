# RUN CARTER v0.12 — Audio intake

The full source from v0.11 remains in this package. Added 34 SHA-256-deduplicated user-uploaded MP3 files, a 100-slot inventory, a 70-cue missing-only list, and source wiring for mouse-click, achievement, and weapon-equip audio with graceful fallbacks.

Run: `python3 tools/verify_audio_inventory.py`. Rust compilation is not verified in this environment. `assets/sfx/audio_inventory.json` is the source of truth for missing sound cues.

**Audio provenance (owner-confirmed, 19 Nov 2026):** All imported MP3s, including voices, came from Pixabay royalty-free uploads. The content license broadly permits commercial use inside a game. Public standalone redistribution of original MP3s may still be restricted. Original sound-page URLs have not been individually collected. Check third-party 3D model licenses separately. 
