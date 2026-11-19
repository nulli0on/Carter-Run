# RUN CARTER — Rust + CarterOS

Development repository for the first-person comedy-survival game RUN CARTER.

## Current repository state
The game source and its model bundle have **not been uploaded to this GitHub repository yet**. The verified local development package is RUN CARTER v0.11. The repository is initialized so that a GitHub Actions build can execute once the source is pushed.

## Build
After uploading the source tree from the v0.11 package, run:

```bash
cargo test
cargo build --release
```

GitHub Actions will build and run tests on each push or pull request when `Cargo.toml` and `src/main.rs` exist.

## Assets and licensing
The local v0.11 package contains 15 unique uploaded 3D assets: 10 fictional armory items, first-person arms, a fan NPC, and three crate variants. Their original files were preserved. Their third-party reuse/redistribution licenses **haven't been verified**. Do not publish those binaries to this public repository or on Steam without checking each license.

The CarterOS visuals and textures were preserved from the original user Figma export.

## Scope
RUN CARTER v0.11 adds an AmazonCrime armory, charge-limited non-lethal crowd-distraction mechanics, inventory persistence, and source integrity checks. This version is not yet a successfully compiled/tested game.

## Branch policy
Develop features in a branch and open a draft PR. Do not merge unverified runtime code.
