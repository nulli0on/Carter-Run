# RUN CARTER v0.13 — Event-driven audio integration

## Changes in local full source
- New `src/audio_events.rs`: queue of typed gameplay/UI sound events, debounce intervals for repeated effects, bounded buffering, and six independent Rust tests.
- `src/main.rs` (local source package) emits events for computer startup, keyboard, mail, purchases/denials, deliveries, pickups, weapon-fire/empty, footsteps, crowd waves, pounding, sugar, gadgets and catching/detonation.
- Existing 34 user-supplied Pixabay MP3s are preserved; additional optional sound bindings in main use those exact filenames. Missing cues remain silent and are not deceptively marked complete.
- Master volume / speakers / system sounds are respected. Events expire from the queue even if audio is disabled; no backlog on re-enable.
- Figma textures, 3D GLB models and original MP3 bytes remain unchanged (local SHA-256 verification of 94 original asset files passed).

## CI scope
This branch includes the **full standalone audio event module** and an Action to compile/run its tests. This **does not** establish that the full game compiles: `Cargo.toml` and `src/main.rs` are not currently in the public GitHub repository. The complete v0.13 game remains in the separately downloadable project package.

Raw Pixabay audio files remain excluded from public source Git history under the license's standalone redistribution restriction; they can be bundled as part of the finished commercial game. Model licenses are separate and still need verification.

## Next blockers
1. Upload the approved local project source to this feature branch so `cargo test` and `cargo build --release` actually run.
2. Fix compiler/runtime issues and inspect in-game audio balance.
3. Check the five tentative audio mappings by listening; source filenames alone aren't conclusive.
