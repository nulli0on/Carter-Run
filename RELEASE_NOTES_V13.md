# RUN CARTER v0.13 — Event-driven sound integration

## Functional source changes

This release connects real gameplay to sound events instead of triggering just the existing UI click and a few special effects. New `src/audio_events.rs` is a pure-Rust queue with bounded buffering and time-based deduplication; it is independent of raylib, Steam, the sound device, and game saves.

**Events emitted:** computer open, keyboard typing, mail handling, shopping success/failure, incoming/collected deliveries, steps during actual movement, inventory weapon use and unavailable equipment, selected-weapon changes, wave start/end, repeated door pounding, player caught, powered gadgets, sugar collected, countdown started, and cartoon detonation. Existing achievement and critical-alert sounds remain hooked up.

**Audible mappings from the supplied recordings:** initial computer startup, keyboard typing, purchase/pickup, shop error, footsteps, door pounding, wave/notification feedback, player damage, explosion, weapon draw, M4 prop, automatic prop and Mark 23 prop firing. Other triggers are recorded but intentionally silent until matching sounds arrive. Do not misrepresent those as finished sound effects.

## Technical changes

- `Game.audio_events` is transient and not persisted into campaign saves.
- All effects use exact previously supplied asset filenames; no Figma, audio, office or character assets were modified.
- Master volume and speaker mute still control the whole raylib output. CarterOS 'System Sounds' toggles UI sounds only. Missing/unsupported sound formats fail softly.
- Keyboard: 100ms; footstep: 310ms; active-wave door pounding: 2400ms; more cues have individual throttles.
- `src/audio_events.rs` is uploaded to `feature/audio-events-v13` and independently compiled/tested in a real GitHub Actions Rust runner; six tests passed.

## Testing limits

GitHub Action: https://github.com/nulli0on/Carter-Run/actions/runs/44617068144

**Module: 6/6 PASS** (actual `rustc --test`). Full Rust/raylib game: **NOT YET COMPILED**: complete game source is not in the public GitHub repository, and this container lacks Rust. Actual audio playback, 3D movement, and Steam integration remain unverified.

The source-only and complete v0.13 archives include the changes to `src/main.rs`, which have not yet been pushed to GitHub. Public redistribution of raw Pixabay MP3 files remains separate from commercial in-game use. The user's license/provenance confirmation is recorded; original item URLs and any download certificates are not yet provided.
