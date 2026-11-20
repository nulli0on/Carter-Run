# RUN CARTER v0.16 — Embedded GLB animation integration

Audited against the **actual 16 GLBs** delivered in the v0.15 full package. No source assets were modified.

| Weapons | Source animation contents | Runtime mapping |
|---|---|---|
| M4 | Draw, Reload, Fire, Reload_Empty, Holster | Authored Draw / Fire / Reload / Holster |
| Vector | Draw, Shoot, Reload, Hide | Authored Draw / Fire / Reload / Holster |
| Mark 23 | Reload, Shoot, Hide, Draw | Authored Draw / Fire / Reload / Holster |
| Pistol | EmptyClipReload, Fire, Hide, Idle, ReloadClip, Unhide, Weild | Authored Draw / Fire / Reload / Holster / Idle loop |
| Cardboard Blaster | Animation | Whole single track on use |
| Knife | anims | Whole single track on use |
| Minigun | allanims | Whole single track on use, no restart during automatic fire |
| Flamethrower | allanims | Whole single track on use |
| Shotgun | allanims | Whole single track on use |
| Sniper | allanims | Whole single track on use |
| Persuader | allanims | Whole single track on use |
| Sticker LMG | allanims | Whole single track on use |
| Grenade Launcher | allanims | Whole single track on use |
| RPG-7 | **No animation clips in GLB** | Static model |
| Grenade | **No animation clips in GLB** | Static model |
| Dynamite | **No animation clips in GLB** | Static model |

**Fact checked:** 13/16 have glTF animation clips. Four have multiple named clips. Nine have one combined/undifferentiated track. Three have no glTF animations at all.

## Implementation

- `src/weapon_animation.rs`: owner-safe raylib 6 `ModelAnimations` loading, model skeleton compatibility check, pose updates and automatic cleanup.
- Original glTF timeline ranges are preserved when sampling the named clips at 60 samples/second.
- Animation state transitions respond to actual successful weapon-fire events even with sound muted.
- Switching queues the authored holster clip before loading the selected weapon where it exists; the new weapon begins its draw clip.
- Models display in a camera-relative first-person position (B toggles).
- `R` triggers an authored **visual reload/inspection** only where the clip exists, unless collecting a nearby fictional sugar packet. Game ammunition remains purchasable charges: R does not refill them.
- Models with missing or invalid animation clips gracefully use a static model fallback.
- Seven source-level unit tests verify clip metadata, timing, looping and state mapping. GitHub Actions runs full Rust tests, release build and office/desktop virtual-display smoke checks.

## Limitations

**Compilation and fallback office startup do not prove animation playback with the real third-party GLB files.** The GLBs are in the downloadable full local project, not on the public GitHub branch. Named clips are wired according to the real metadata but require visual checks for skeleton integrity, scale/orientation and rendering.

A single `allanims` track does not identify separate Draw / Fire / Reload / Holster boundaries. The code plays the **actual complete track on use**; it deliberately does not invent action-specific time slices. Clip-segmentation by visual review is still necessary for those nine weapons.

No new model animations were fabricated. All 36 original CarterOS Figma PNGs are unchanged.

### Build and test

```sh
cargo test
cargo build --release
# Requires a desktop or virtual X display:
./target/release/run-carter --smoke-frames=16
./target/release/run-carter --smoke-frames=16 --smoke-desktop
```

To test the real embedded animations, include the licensed `assets/arsenal/*.glb` folder from the full private v0.16 package.
