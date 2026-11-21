## v0.19 first-person weapon visibility

16 model-size profiles, corrected view-right orientation, raised viewmodel position, and actual-asset screenshot comparisons. See [v0.19 verification notes](docs/VIEWMODEL_V19.md). Full Rust CI and portable/Windows builds run on `feature/v19-viewmodel-calibration`. The original CarterOS PNGs are preserved; the asset pack remains local.

# RUN CARTER v0.16 — Embedded Weapon Animations

**Verified in GitHub CI:** `cargo test` (85 PASS / 0 FAIL), Linux release build, CarterOS art hashes, and office/CarterOS 16-frame virtual-window start. The GLBs remain local, so real animation visual playback is not verified by CI. See [animation audit](docs/WEAPON_ANIMATIONS_V16.md).

# RUN CARTER v0.15 — Updated Weapon Arsenal

**Verified CI:** `cargo test` **78 passed / 0 failed**; `cargo build --release` **PASS**; both office and CarterOS **16-frame Xvfb startup tests PASS**. [Successful workflow](https://github.com/nulli0on/Carter-Run/actions/runs/44740815713).

The owner supplied `Archiv(10).zip` with 18 GLBs, including six additional gun types and new grenade/dynamite models. The **full local package** now has 21 validated GLBs: 16 weapon models and five supporting assets. All previous ten weapon inventory IDs are preserved. CarterOS's 36 original Figma PNGs remain untouched, and the game loads legacy v0.14 save files containing ten weapon charge slots.

**Controls:** `1`–`0` select original ten weapons. Mouse wheel, PageUp or PageDown cycle all 16. AmazonCrime's Armory 1 and Armory 2 pages each show eight items.

**Scope limitation:** Original 3D model binaries and raw Pixabay MP3s are intentionally not included in this public code repository. The full-asset version needs actual model scale/rotation and animation QA; the CI startup uses fallback assets. Third-party 3D model licensing must be verified separately. No Steam gameplay/unlock test has been performed.

[Updated weapon details](docs/V15_UPDATED_WEAPONS.md) · [Draft PR #5](https://github.com/nulli0on/Carter-Run/pull/5)

---

# RUN CARTER v0.13 — AmazonCrime Arsenal & Imported 3D Models

**Rust + raylib source prototype. Not a verified runnable release.** v0.11 builds on v0.10. The original CarterOS Figma artwork is unchanged. See `README_V11_ASSET_IMPORT.md` for 15 unique 3D models, model provenance, and gameplay integration; source validations passed but the full Rust build is not yet verified.

## Start here

1. Read **`README_V11_ASSET_IMPORT.md`** for the GLB model import and new AmazonCrime armory.
2. Read **`ACHIEVEMENTS_AND_EASTER_EGGS.md`** for all achievement IDs, conditions, cheat restrictions, secrets, and testing warnings.
3. Read **`RELEASE_NOTES_v10.md`** for what changed from v0.9.
4. Run the asset/source checks and a real Rust test/build on a machine with Rust and raylib native dependencies:

```bash
python3 tools/verify_v10.py
python3 tools/verify_arsenal_assets.py
cargo test
cargo run --release
```

Linux build libraries (Debian/Ubuntu): `build-essential cmake clang pkg-config libasound2-dev libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev`.

## Controls

- **Office**: WASD movement, mouse look, Shift sprint, E use CarterOS computer, 1–0 equip armory items, left click deploy harmless defensive effect, B toggle 3D weapon preview, Q Raspberry Pi, Z Airhorn, X Confetti Cannon, C Drone, R fictional sugar pickup, F collect AmazonCrime deliveries.
- **Emergency money**: F6 when cash is below $80. It gives a $400 in-game bailout with $450 debt and forfeits Zero Waste for the current run.
- **Save/load**: F5 save, F9 load, F10 save-and-quit attempt; unavailable during attacks or active cartoon sugar countdown.
- **Debug**: F12 or ~ opens a read-only inspector (no shame until cheats are enabled). `help`, `status`, then `debug on` for cheat commands. In CarterOS Settings, hidden **Ctrl+Shift+D** also enables debug features and marks the run cheated.
- **CarterOS**: Enter unlock, Escape leave, dock shortcuts, window dragging, Cmd-like Ctrl shortcuts as in v0.9. Ctrl+N in CarterMail composes a secret recipient.
- **Death**: R starts a fresh run and resets run-level cheat restriction without deleting lifetime achievement records.

## New gameplay summary

- Four tracked defensive gadgets: Raspberry Pi, Airhorn, Confetti Cannon and Decoy Drone.
- Cartoon mystery sugar: 9% chance to spawn when a wave or shift event occurs, or buy a fictional pack from AmazonCrime for $75. Three pickups activate a 75-second timer, further pickups reset it, and running out triggers a cartoon explosion.
- Forced attacks after every second completed shift before the day-20 victory milestone.
- Persistent achievement counters for AI nonsense, repelled kids, and emergency sugar saves.
- Steamworks achievement API IDs are **not automatically created**. All 19 IDs need to be configured in your actual Steamworks game dashboard before live Steam unlocking can work. The SDK bridge only attempts to submit pending achievements.
- No public real internet browsing, host filesystem editing, or execution of arbitrary terminal commands is added.

## Important limitations

- The authoring environment has no `cargo`/`rustc`, and no game window/Steam runtime; compilation and playability are **unverified**. Python verification covers files/source contracts only.
- Eight CarterOS applications still have provisional window interiors because there is no corresponding Figma design. Source Figma images were not changed.
- The anti-tamper checks are simple heuristics (non-cryptographic checksums, anomalous money changes and extreme clock adjustments). They can be bypassed or misfire, particularly with manual save editing, DST/clock changes, suspend/resume, or bugs. Do not treat them as reliable cheat detection.
- Very long "le bullshit" streak trophies are vanishingly rare at the intentionally unchanged 17% comedic roll probability. No pity system is implemented.
- No real drug use or real-world effects are simulated; the white sugar is a fictional cartoon game item.
- Steamworks requires a real Steam App ID; this source archive includes no such entitlement or verified achievement configuration.

`docs/README_v09_history.md` contains the previous release's longer baseline instructions.

## Audio event layer (v0.13)

`src/audio_events.rs` and `src/main.rs` wire gameplay/UI actions to the bundled licensed Pixabay recordings with event throttling and optional playback. This module has **6 passing GitHub-hosted Rust unit tests**, but the **full game still has not passed a Cargo build**. See `RELEASE_NOTES_V13.md` and `QA_REPORT_V13.md`. The sound inventory still marks 70 cues MISSING, with five source-file candidates needing review.
