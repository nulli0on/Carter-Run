# RUN CARTER 1.0.2

This is a standalone Windows and Linux release candidate with the save, HUD, and gameplay updates from 1.0.1 and the original assets from 1.0.0. The archive contains the rebuilt executables and editable Rust source.

## Changes in 1.0.2

- Fans use a clear path around corners during close pursuit. The game reuses the chase and decoy navigation fields while their target cells remain unchanged.
- The service district objective panel and minimap have room to fit side by side at 640 × 400. F11 switches between windowed and borderless fullscreen. Either Shift key activates sprint.
- The office HUD displays equipment recharge progress. Clicking during recharge stays quiet. The empty sound and refill prompt play when a ready item runs out of cartridges.

## Verified in this build

- All 116 Rust tests passed, including the service district corner regression. The Linux release and Windows x64 cross-build compiled using the lockfile.
- Linux playtesting exercised F11 fullscreen and window-size restoration, the compact service district HUD, title volume, focus-loss pause, delivery autosave, and the safe and unsafe window-close paths.
- Linux rendering smoke tests passed for the office and CarterOS. The runtime animation audit passed for all 16 equipment models. The original asset tree matched the supplied archive byte for byte.

## Steam handoff

Use the native executable paths `bin/windows/run-carter.exe` and `bin/linux/run-carter` as platform launch options once the corresponding depots are set up. Include the assets and required notices. Validate launches from Steam on every operating system advertised on the store page. See [SteamPipe upload setup](https://partner.steamgames.com/doc/sdk/uploading) and [Valve's build review process](https://partner.steamgames.com/doc/store/review_process).

The supplied binaries were built with the standalone configuration. Steam achievements and Cloud synchronization need an App ID and Steamworks configuration, followed by client testing. Test the native Linux binary inside the selected Steam Linux Runtime because it requires glibc 2.39. Native Windows startup, physical audio/input, and Steam Deck behavior remain unverified here. Run `release/Test-Windows-Release.ps1` on Windows, then play with normal input and audio. The package's manifests record asset provenance and hashes. Complete the asset-specific redistribution review before publishing the game's third-party artwork and recordings.
