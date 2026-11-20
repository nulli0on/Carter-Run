# RUN CARTER v0.17 — Animation compatibility

- Original user GLBs remain unchanged in the full local package.
- Cardboard blaster's five animated object parts have been converted into rigid-joint skinning for raylib's GLTF animation importer. Runtime uses `cardboard_blaster_rigged.glb`, built from the original with `tools/convert_cardboard_for_raylib.py`.
- 12 other skeleton-animated weapon models retain authored tracks; RPG, grenade and dynamite have no authored clips, so use procedural viewmodel recoil/draw/holster rather than claiming animations exist.
- Named animation clips preserve original timeline ranges. Unlabelled `allanims` tracks do not have fabricated action boundaries.
- Press F9 while in first-person view to show animation and loaded-track diagnostics.
- `--audit-weapon-animations` on the complete local package loads all 16 assets, updates four action poses, and reports failure if animations are missing. This is an optional real-asset diagnostic, not a source-only CI guarantee.
- Scale normalization uses the raylib model bounding box. Orientation, exact first-person feel and user-supplied source-GLB runtime rigging require visual QA.
- Commercial redistribution rights to source third-party GLBs are not yet confirmed; those files are excluded from the public Git repository.
