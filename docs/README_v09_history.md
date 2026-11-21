# RUN CARTER v0.9 — CarterOS + CarterSearch + Secret Cheats

**What's new:** CarterWeb has optional local AI answers, funny deliberately unreliable searches, no-result chance, secret fictional URLs, in-game cheat email recipients, and an optional Steamworks achievement bridge. See `SECRETS_AND_AI.md` and `RELEASE_NOTES_v09.md` for usage, limitations and spoilers.

**The original Figma files and artwork are unchanged.** This is a prototype built in Rust + raylib; do not treat the release as compiled or tested until `cargo test` and a local gameplay test pass.

Run: `cargo test` and `cargo run --release`. Optional real local AI: `CARTEROS_AI_MODEL=qwen2.5:1.5b cargo run --release` after installing Ollama and downloading that model. Optional Steam submission: `cargo build --release --features steam`, after creating the achievement in Steamworks under your game's actual App ID. `Escape` exits CarterOS; `Ctrl+N` inside CarterMail composes a fictional message.

---

# RUN CARTER v0.9 — CarterOS gameplay and persistence iteration

A Rust + raylib 6.0 game-source prototype. **Not a verified runnable release:** the working environment did not have Cargo/rustc, so the code has not been compiled or played here.

## Design fidelity

The user's original CarterOS `.fig.zip`, `.svg`, 36 exported PNG assets (including 12 source-exact Figma artboards), and the original-textured 3D office GLB are unchanged. Their SHA-256 checks match v0.6. Existing default desktop / Settings artboards are still used at their native 1440×900 resolution. New interactions (live file text, notifications, moveable windows) necessarily draw dynamic overlays. There are still no supplied Figma window designs for eight apps; their bodies remain provisional and should not be presented as pixel-perfect Figma screens.

## CarterOS features introduced in v0.7

- **Multi-window desktop:** keep up to 6 app windows open; switch using dock or Ctrl+Tab; title-bar close and minimize buttons; restore apps from the dock; parked windows remain drawn behind the active one.
- **Persistent window positions:** dragging an application window saves its position in `.run_carter/window_layout.cfg`. Positions are validated on load so windows remain reachable.
- **Virtual Carter Drive v2:** create text files and folders, rename, search, move to Trash, restore, view and edit fictional document text, and save changes. All data is isolated to `.run_carter/drive_v2.cfg` — NO host files are opened or deleted. Migration imports names from v0.6 virtual-files list if present.
- **Live notification history:** on-screen notification panel shows recent in-game events, can be cleared. Automatic fictional inbox reminders appear periodically; Creator Focus suppresses their popup without hiding important alerts or immediate user-action feedback.
- **Figma right-click menu:** New folder, Sort by, Refresh, Personalize, Display settings, Privacy dashboard.
- **Launchpad search:** type an app name and press Enter to open it.
- **125% interface zoom:** pan the enlarged CarterOS canvas with middle-mouse drag, instead of losing access to off-screen regions.
- **New automated tests and CI config:** Rust tests for multi-window behavior and virtual-drive data; GitHub Actions workflow attempts cargo test/build on a Linux host with native raylib dependencies.

## New in v0.9

- **Campaign save/load:** in-game money, player position, day, remaining mail, inventory, protection upgrades and AmazonCrime orders persist to `.run_carter/game_v1.cfg`. The file format is versioned and validates ranges to reject corrupted saves. OS settings and Carter Drive remain separate.
- **Save safety:** **F5** saves a safe checkpoint; **F9** reloads that checkpoint; **F10** attempts to save and quits. Saving/loading during an attack is intentionally blocked. Completed days autosave. Dying and pressing **R** explicitly begins a new campaign, overwriting the previous safe save.
- **Actual AmazonCrime deliveries:** physical Raspberry Pi and scanner orders are queued and take 20/15 simulated seconds, respectively. Notifications announce arrival; a visible package appears near the front door. Reach the door and press **F** to collect items before they become usable. Digital AntiDoxx and door reinforcement remain instant for this prototype.
- **CarterMail shift completion:** the ten scripted emails are now limited to one pass per day; after processing them all, accepting the final "End Shift" action starts another day and pays a $45 shift bonus. Surviving an attack also advances to the next day. The game cannot be soft-locked by safely deleting every mail.
- **Obstacle-aware enemy routing:** basic 2D flow-field navigation routes the cube kids around the *same approximate* desk/shelf volumes used for player collision. This is NOT a navigation mesh: the furniture and door geometry still need proper collision meshes.
- **Accurate shop, radar and camera data:** shows pending/ready orders, next delivery time, nearest active kid and front-door visitor count. All remain fictional game information.
- **Game events into CarterOS:** new-day and delivery notifications enter the OS notification history, honoring Creator Focus for noncritical toasts.

## Existing gameplay

First-person 3D office, scripted CarterMail, simulated phishing/location exposure, basic kid attacks, decoy items, AmazonCrime, unlockable apps, camera/privacy/network controls, Night Comfort, sound controls, original wallpaper themes, in-game money, game-world time continuing while CarterOS is open. This is simulated game software, **not** a host operating system, Internet browser, real security suite, or IP/location service.

## Requirements

Rust stable/Cargo, raylib 6.0 native build prerequisites (CMake, C compiler, graphics and audio libraries). Linux example (Debian/Ubuntu):

```bash
sudo apt install build-essential cmake clang pkg-config libasound2-dev libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev
./scripts/check.sh
cargo run --release
```

No Rust binary is bundled. `./scripts/check.sh` runs asset integrity checks and then Rust tests and release compilation. If compilation fails, capture the **full first compiler error** and fix it before assuming gameplay is tested.

## Controls

Office: **WASD** walk, mouse look, **Shift** sprint, **E** open computer when near desk, **Q** deploy a Raspberry Pi decoy, **F** collect delivered goods by the entrance, **F5** save, **F9** reload, **F10** quit. On death: **R** restart.

CarterOS: **Enter** unlock, **Esc/E** leave computer, **Ctrl+L** lock, click dock/Launchpad to open apps, drag title bar to move, red/yellow controls close/minimize. **Ctrl+Tab** cycle open windows, **Ctrl+M** minimize, **Alt+F4** close. Original green title-bar control intentionally does not stretch Figma artboards. **Middle-mouse drag** pans the 125% enlarged interface.

Carter Drive (Files app): **Ctrl+N** new text, **Ctrl+Shift+N** new folder, **F2** rename, **Ctrl+F** filter by name, **Enter** open selected document, **Up/Down** select, **Backspace** move to Trash, **Ctrl+T** toggle Trash, **Ctrl+R** restore from Trash. Text editor: type to append, Enter line break, Backspace delete, **Ctrl+S** save, **Esc** close once saved. File operations act ONLY on the game's virtual storage.

All keyboard shortcuts are game-local while interacting with CarterOS. The title-bar maximize control is **not yet implemented** as native resizing because no matching expanded Figma layout exists.

## Known limitations

- **No completed running-game verification**: Rust compiler/toolchain not installed in the authoring environment. Asset tests pass but `cargo test` and `cargo run` are pending.
- The green maximize button and true resizable windows remain unimplemented; a change without corresponding Figma artwork would deviate from the exact source design.
- Virtual folders are currently named entries; nested folder navigation is not implemented.
- The document editor is append-oriented and limited to 8192 bytes per document; no cursor-position editing, formatting, or external file operations.
- Eight app window bodies are still temporary UI panels, because the Figma file contains only their icons.
- The browser, app store, camera, radar and security features are **fictional in-game simulations**, not real services. Player collision is still approximate; the new 2D navigation flow field needs gameplay testing and a proper collision mesh. A fully playable build still needs verification.
- Before public release, verify commercial licensing for the third-party office model and consent for using a real creator's name/likeness.

See `RELEASE_NOTES.md` and `QA_REPORT.md` for the implementation and verification audit.

## v0.9 acceptance notes

Run `python3 tools/verify_v08.py` to check source presence and numerical navigation assumptions. For the real acceptance gate, run `./scripts/check.sh` on a machine with Rust stable and native raylib prerequisites; it executes the original Figma and asset checks followed by `cargo test` and a release build. This archive is a source implementation, not a tested executable.
