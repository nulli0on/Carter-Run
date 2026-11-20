# RUN CARTER v0.19 — Real-model first-person calibration

Date: 20 November 2026. Source branch: `feature/v19-viewmodel-calibration`.

## Findings from actual supplied GLBs
The v0.18 compiled game was launched under Linux Xvfb/Mesa with all 16 supplied weapon models. After turning the weapon preview off with B, image differences identified a major presentation error: most weapons were low on screen, several were tiny, and the M4 occupied approximately 1,298 changed pixels despite loading its clips correctly. The black-screen bug from v0.18's abandoned RGBA compositing path was fixed using a foreground depth override.

## Changes
- Corrected the camera-relative horizontal basis and raised held models from the overly low default.
- Added the 16-slot `VM_PROFILE_MULTIPLIERS` table in `src/main.rs`, bounded by a Rust unit test, without changing models, ammunition or old save slots.
- Preserved all source CarterOS Figma PNGs. The source-export workflow captures exact Git-tracked code/assets without redistributing standalone third-party MP3 and GLB media.
- Portable Linux and Windows CI enabled on this feature branch.

## Verification protocol
1. Run `cargo test` and `cargo build --release` in GitHub Actions.
2. Run the game with `--smoke-frames=16` and `--smoke-desktop`; both must exit successfully.
3. With the **full local model assets**, run each slot through `--smoke-frames=600 --qa-weapon-slot=N` and capture one screenshot with the model visible, then press B and capture the same view without the model.
4. Compare images, excluding HUD, record changed-pixel counts and inspect the original images for model orientation, clipping and textures.
5. Do not claim that rendered models look correct merely because GitHub source-only CI passes.

## Still open
Several original GLBs use long unlabelled `allanims` tracks, so specific action segments remain artist review work. The cardboard gun's mesh has visible white-source fragments. Third-party GLB licenses, Steam integration, the full 20-day campaign and real desktop gameplay testing are separate release blockers. This v0.19 feature is a technical and presentation iteration, not a finished product.
