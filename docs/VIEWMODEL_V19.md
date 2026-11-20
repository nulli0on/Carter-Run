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

## 16-slot first-person screenshot audit (20 Nov 2026)
The complete original asset bundle was used with the portable v0.19 Linux build under Xvfb/Mesa. Screenshot A was recorded with the current weapon visible, screenshot B after pressing B to hide the weapon. Within the 1440×900 cropped viewport the high-contrast changed-pixel totals were:

| Index | Weapon | v0.18 | v0.19 initial |
|---:|---|---:|---:|
| 0 | Cardboard | 45,058 | 54,794 |
| 1 | Knife | 20,380 | 17,792 |
| 2 | Mark23 | 4,467 | 11,863 |
| 3 | Vector | 5,975 | 16,106 |
| 4 | M4 | 1,298 | 30,679 |
| 5 | Minigun | 10,169 | 47,660 |
| 6 | Flamethrower | 9,859 | 26,284 |
| 7 | RPG7 | 4,885 | 14,493 |
| 8 | Grenade | 45,459 | 47,894 |
| 9 | Dynamite | 6,787 | 18,430 |
| 10 | Shotgun | 8,464 | 46,986 |
| 11 | Sniper | 4,101 | 30,532 |
| 12 | Persuader | 14,451 | 28,451 |
| 13 | LMG | 5,066 | 28,987 |
| 14 | Grenade launcher | 19,273 | 96,628 (clipped) |
| 15 | Pistol | 25,762 | 27,521 |

**Correction after visual audit:** The grenade launcher profile is reduced from 1.65× to 1.12× because the right sleeve clipped past the viewport. This latest correction requires a fresh screenshot and final CI check.

The before/after pixel counts are *visibility indicators*, not animation-quality scores. The Mark23 and M4 still show disconnected moving pieces in some poses and require further rig/animation interpretation. All original files remain untouched.
