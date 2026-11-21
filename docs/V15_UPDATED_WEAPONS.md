# RUN CARTER v0.15 — Your updated 3D weapons

Uploaded archive: `Archiv(10).zip` (18 GLBs, supplied manifest was outdated).

## Exact modifications
- Replaced `grenade.glb` with uploaded `granate.glb`, and `dynamite.glb` with `dynamite_stick.glb` in the local distributable.
- Six newly purchasable weapon slots 11–16: `shotgun.glb` (Foam Shotgun), `sniper.glb` (T-Shirt Sniper), `persuader.glb` (Persuader Megaphone), `lmg.glb` (Sticker LMG), `grenadelauncher.glb` (Confetti Launcher), `pistol.glb` (Pistol prop).
- 10 legacy IDs and save slots remain stable. M4 and minigun uploaded bytes are exact duplicates of the previous assets.
- 16 items fit existing ARMORY 1 and ARMORY 2 tabs (8 per page), without editing any of the original CarterOS Figma artwork.
- `1`–`0` keys still select the original ten. Mouse wheel / PageUp / PageDown cycle through *all owned* weapons.
- `game_save.rs` upgrades 10-slot v0.14 save files to 16 slots and retains cash, existing equipment and campaign.
- Achievement Full Arsenal, Zero Waste now requires owning all 16 current weapons on a solvent clean run, without a bailout.

## Limitations
- GLB positions, scale, orientation and animations require gameplay testing. A model being loadable is not proof of animation readiness.
- The model filenames/assets are included in the full **local** ZIP, not committed to this **public** repository. Original Sketchfab-related GLB asset license information is absent. Do not redistribute raw GLBs or Steam-packaged game until commercial rights to those files are independently confirmed.
- CI builds with fallback geometry where third-party model assets are withheld.
- All original CarterOS PNGs were verified unmodified against v0.14 in the local package. GitHub also enforces the Figma PNG manifest checks.
