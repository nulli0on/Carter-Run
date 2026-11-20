# RUN CARTER v0.18 — Real first-person weapon visual QA

Date: 2026-11-20. Branch: `feature/v18-viewmodel-qa`.

## Verified observations
- Local full game loaded 16 original/derived weapon models and their available raylib animations. Non-animated props still use procedural motion.
- The v0.17 renderer positioned viewmodels inside the office depth scene. A controlled Xvfb comparison against the same view with the weapon toggled off found that some weapons were almost invisible (e.g. M4 approximately 293 strongly changed pixels).
- The first attempted separate transparent framebuffer was **rejected** after the real screenshot appeared opaque black despite green Rust CI. It was replaced with a second 3D pass using an explicit temporary depth-test override.
- The corrected pass no longer blackens the room. A camera-right sign correction, adjusted vertical offset and 16 per-weapon scale multipliers were validated against all 16 real GLBs.
- All 16 had measurable visible geometry in the real 1440×900 Xvfb screenshot, after comparing weapon shown/hidden. These visibility counts test *presence*, not correct visual pose.
- The owner's source M4 uses `KHR_materials_pbrSpecularGlossiness` for all four materials and lacks a standard metallic/roughness base-color fallback. It rendered almost solid white in raylib 6. A derived `m4_raylib.glb` restores its texture maps; original `m4.glb` is byte-for-byte unchanged.
- Original 36 CarterOS Figma PNG exports remain unchanged; source-only CI does not include these private third-party weapon GLBs or Pixabay sound files.

## Known imperfections
- Some source animations show detached magazines, hand joints or offset poses in first-person. They remain **VISUAL_QA_REQUIRED** and are not described as perfect.
- One-track `allanims` recordings lack trustworthy action segment labels; we do not invent them.
- Screen-space tuning under other resolutions, collision and real Windows playtesting remain incomplete.
- Many GLBs contain 32-bit indices, for which raylib prints u32→u16 conversion warnings. Some large meshes may require a mesh import optimization pass.
- No Steam API achievement integration was actually playtested.

## Generate compatible M4 copy
```bash
python3 tools/convert_specgloss_materials.py assets/arsenal/m4.glb assets/arsenal/m4_raylib.glb
```
The game loads the derived copy when present. If absent, it falls back to the original M4.

## Bounded local QA
```bash
LIBGL_ALWAYS_SOFTWARE=1 ALSOFT_DRIVERS=null xvfb-run -a \
  ./target/release/run-carter --audit-weapon-animations
```
For visual inspection, run the game with `--smoke-frames=600 --qa-weapon-slot=4`. Smoke-only tuning flags `--qa-vm-scale=`, `--qa-vm-right=`, `--qa-vm-down=`, `--qa-vm-distance=` and `--qa-vm-yaw=` do not affect normal play.
