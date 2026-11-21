# RUN CARTER updated armory asset audit

## New from Archiv(10).zip

| File in package | Gameplay item | Change |
|---|---|---|
| grenade.glb | Party Grenade | Replaced old grenade with `granate.glb` (user upload) |
| dynamite.glb | Dynamite Pinata | Replaced old bundle with `dynamite_stick.glb` |
| shotgun.glb | Foam Shotgun | New weapon #11 |
| sniper.glb | T-Shirt Sniper | New weapon #12 |
| persuader.glb | Persuader Megaphone | New weapon #13 |
| lmg.glb | Sticker LMG | New weapon #14 |
| grenadelauncher.glb | Confetti Launcher | New weapon #15 |
| pistol.glb | Pistol (prop) | New weapon #16 |

The old GLB copies of M4 and minigun in Archiv(10).zip are hash-identical to v0.14. Other supplied original models are also preserved. The archive's supplied manifest.json lists only 15 old assets and is therefore **outdated**; it has been regenerated based on actual files. Original Figma PNGs have not been modified.

## Controls

- Keys `1`–`0`: original 10 weapons by fixed ID.
- Mouse wheel or `PageUp` / `PageDown`: cycle through owned weapons including six new items.
- Left mouse button: use selected comedy prop.
- `B`: view currently selected model preview.

**Graphics limitation:** on-screen weapon preview scales/axis transforms are provisional and skeletal animation playback remains unfinished. A successful build/startup does not imply model animation correctness.

**License limitation:** some GLBs have third-party origins (Sketchfab generator metadata); commercial redistribution permission remains to be confirmed separately for each model. Keep standalone GLBs out of public GitHub until permissions are checked.
