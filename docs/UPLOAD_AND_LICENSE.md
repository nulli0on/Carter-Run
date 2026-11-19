# RUN CARTER v0.12 — GitHub handoff and Pixabay audio licensing

## User-confirmed audio provenance (19 November 2026)

The project owner confirms that **all 34 unique recordings, including voices, were obtained from Pixabay** and are intended for use under its royalty-free license. This corrects the previous documentation which treated their source as unknown.

This confirmation is recorded as user-supplied provenance, **not** an independently checked file-by-file license audit. Original asset URLs and any available Pixabay download certificates should be retained in a sound attribution/provenance ledger.

## What Pixabay permits and restricts

Pixabay's current Terms of Service (last updated 20 October 2026) grant broad **commercial-use** rights to downloaded licensed content, including incorporation in creative works such as a video game, usually without mandatory attribution.

However, its regular Content License expressly **forbids standalone distribution** of unchanged content. Therefore:
- **Use as in-game sound/voice effects in RUN CARTER or its Steam release:** generally permitted, provided no other rights are infringed and the recordings are integrated as part of the larger game.
- **Upload original MP3s individually to a public GitHub repository:** **not automatically permitted**; this may be standalone redistribution. Keep raw files excluded until their individual license classification/terms are confirmed.
- **CC0 exception:** Pixabay content published before January 9, 2019 is marked CC0 under Pixabay's current terms and has different redistribution terms. Do not assume these files are CC0.
- **Voice recordings/identifiable people:** ensure that recognizable individuals are not used in misleading or defamatory contexts; the stock-audio license alone cannot guarantee every third-party personality/publicity right.

Official sources:
- https://pixabay.com/service/license-summary/
- https://pixabay.com/service/terms/
- https://pixabay.com/service/faq/

## What's on GitHub
- The exact 100-sound inventory, 70 missing cues, 5 plausible candidate assignments, and a CI script to validate those counts.
- The complete game source is **not** yet in this repository. Passing the audio inventory job does not build the game.

## Local full package
`RUN_CARTER_v0.12_Audio_Integrated.zip` includes Rust source, the original CarterOS art/office, uploaded 3D models, and 34 deduplicated MP3s with SHA-256 asset manifest. This local project can be used for development and later game packaging.

## Upload procedure
1. Download the generated full archive; clone `https://github.com/nulli0on/Carter-Run.git` and checkout `feature/audio-inventory-v12`.
2. Extract the contents of the `run-carter-carteros-v12/` folder into the repository root; preserve the original CarterOS assets.
3. Commit Rust source and approved project files. Keep standalone MP3 and third-party GLB files excluded from **public** Git history pending asset-specific rights review. Store asset origin URLs and license information in project documentation.
4. Once `Cargo.toml` and `src/main.rs` are actually in the repository, the existing GitHub workflow will attempt `cargo test` and `cargo build --release`.
5. Resolve compiler failures, verify game audio events and playable state, then consider merging.

The connected GitHub API tool cannot ingest arbitrary local container ZIP, MP3 or GLB bytes directly; the full local archive has not been published to GitHub.
