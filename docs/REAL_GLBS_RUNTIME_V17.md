# RUN CARTER v0.17 — Real GLB animation audit, 20 Nov 2026

## Method

A Rust release binary was built inside Ubuntu 24.04 (glibc 2.39) using GitHub Actions and downloaded to the **local complete project directory**. The game ran under Xvfb/Mesa software GL with all 21 original uploaded GLBs and the derived cardboard rigid-skin model.

Command:
```sh
LIBGL_ALWAYS_SOFTWARE=1 ALSOFT_DRIVERS=null \
  xvfb-run -a -s '-screen 0 1600x1000x24' \
  ./target/release/run-carter --audit-weapon-animations
```

**Result: exit 0. All 16 model loads succeeded; all expected animation tracks were accepted by raylib and four action poses per model updated without an error.**

| Slot | Model | Native tracks | Audit |
|---:|---|---:|---|
| 0 | cardboard_blaster_rigged.glb | 1 | SKELETAL_OK |
| 1 | knife.glb | 1 | SKELETAL_OK |
| 2 | mark23.glb | 4 | SKELETAL_OK |
| 3 | vector.glb | 4 | SKELETAL_OK |
| 4 | m4.glb | 5 | SKELETAL_OK |
| 5 | minigun.glb | 1 | SKELETAL_OK |
| 6 | flamethrower.glb | 1 | SKELETAL_OK |
| 7 | rpg7.glb | 0 | STATIC_PROCEDURAL_OK |
| 8 | grenade.glb | 0 | STATIC_PROCEDURAL_OK |
| 9 | dynamite.glb | 0 | STATIC_PROCEDURAL_OK |
| 10 | shotgun.glb | 1 | SKELETAL_OK |
| 11 | sniper.glb | 1 | SKELETAL_OK |
| 12 | persuader.glb | 1 | SKELETAL_OK |
| 13 | lmg.glb | 1 | SKELETAL_OK |
| 14 | grenadelauncher.glb | 1 | SKELETAL_OK |
| 15 | pistol.glb | 7 | SKELETAL_OK |

## Live visual check

The game was run in its normal office gameplay view with the derived cardboard blaster selected. A synthetic X11 mouse-click fired the weapon, and successive screenshots of the real rendering showed **more than 40,000 changed pixels** within the first-person weapon capture region after advancing into the authored clip. Comparison with a temporary unrigged original mesh showed the white fragment artifacts were already present in the source, rather than introduced by conversion. Original assets and Figma PNGs were restored and verified by SHA-256 afterward.

Visual QA also revealed initial excessive weapon size, leading to a normalized bounding-box target change from 0.70 to 0.38 world units and a camera-relative position adjustment.

## Other actual evidence

- Original 21 GLB files were not rewritten; a separately named 22nd derived cardboard GLB was added locally.
- The original 36 CarterOS Figma PNGs pass byte-for-byte hash verification.
- Owner-confirmed Pixabay SFX are embedded in the **full local project**, not published as standalone MP3s to the public Git repository.
- The source-only GitHub build still boots with fallback geometry; third-party model licensing remains pending.

## Remaining limitations

Audit success does not mean every pose looks good. Rigged source tracks with only one `allanims` action still lack reliable subaction labels. Rendering, hand alignment, gun orientation and camera cropping need human review for all 16 models. No Windows or Steam executable has been tested. The full game is a development build, not a finished commercial release.
