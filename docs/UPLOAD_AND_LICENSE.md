# RUN CARTER v0.12 — Binary handoff / incomplete GitHub upload

## What's on GitHub
- The exact 100-sound inventory, 70 missing cues, 5 plausible candidate assignments, and a CI script to validate the plan.
- Original source/game project is **not** yet in this repository. Passing the inventory-only workflow does not build the game.

## Local full package (in this ChatGPT session)
`RUN_CARTER_v0.12_Audio_Integrated.zip` includes v0.11 Rust source, art, office, 15 unique GLB models, and 34 imported MP3 files with a hash-checked asset manifest. User must transfer the locally generated archive into their cloned repository after evaluating third-party asset licenses.

## Upload procedure
1. Download the full project archive from the ChatGPT response.
2. Clone `https://github.com/nulli0on/Carter-Run.git`, checkout `feature/audio-inventory-v12`, and extract the contents of the `run-carter-carteros-v12/` folder into the repository root.
3. Preserve original CarterOS/Figma files unchanged.
4. Do **not** publish `assets/arsenal/*.glb` or `assets/sfx/imported/*.mp3` to the **public** repository without verifying and documenting each third-party reuse license; keep unlicensed files local instead. Apply `.gitignore` to both.
5. Commit approved source/config/assets; open a PR. GitHub Actions will then run the Rust compilation job when `Cargo.toml` and `src/main.rs` exist.
6. Verify `cargo test`, `cargo build --release` and in-game audio. Do not merge on a skipped or failing build.

Note: The GitHub connector cannot directly ingest local container ZIP/GLB/MP3 bytes, so only repository-native text files could be edited in this pass.
