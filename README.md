# RUN CARTER — Endless Office 1.0.2

Carter has an inbox full of questionable offers and a front door full of people asking for free PCs. Take real offers, spend the proceeds on the office, and decide how much attention you're willing to attract.

## Play

Extract the whole folder before running the game. It starts offline and works without Steam or an AI setup.

- **Windows 10/11 x64:** Run `RUN_CARTER_Windows.bat`.
- **Linux x86-64:** Run `./RUN_CARTER_Linux.sh` from the extracted folder. If your unzip tool drops permissions, run `chmod +x RUN_CARTER_Linux.sh bin/linux/run-carter` first. Requires glibc 2.39 or newer, X11 and OpenGL.
- **Start a run:** Choose **Standard** for the gradual curve or **Overtime** for faster fans and earlier reinforcements. Continue loads the last checkpoint.
- **Office:** WASD moves, Shift sprints, E opens the nearby computer, F collects packages near the door, and Esc pauses.
- **CarterOS:** Unlock with Enter. Use the dock to open CarterMail, AmazonCrime or other apps. **Close Shift** is in CarterMail, and Esc returns to the office.
- **Equipment:** Scroll or press 1–0 to equip. Click to use and hold for automatic props. The bar below the equipment controls fills while the item recharges. Q throws a Pi decoy, Z uses the horn, X uses the confetti cannon and C sends out the drone. R inspects the item or picks up nearby Colombian Powder.
- **Checkpoints:** The game autosaves at safe points. F5 saves manually and F10 saves and quits when safe. Closing the window during an attack offers to resume or quit to the last checkpoint. After being caught, Enter retries the checkpoint. Continue can recover the preceding valid checkpoint if the current one is damaged.

The window fits the current display at startup and can be resized. Press F11 to toggle borderless fullscreen. Use **Settings** on the title or pause menu for mouse sensitivity, vertical inversion, and volume. The office and the arena pause when the game loses focus.

### Saves

| Platform | Save folder |
| --- | --- |
| Windows | `%LOCALAPPDATA%\RUN CARTER` |
| Linux | `$XDG_DATA_HOME/run-carter`, or `~/.local/share/run-carter` if XDG_DATA_HOME is unset |

For a first launch with an older campaign, copy your previous `.run_carter` folder beside the launchers. The game imports it when the new save folder is absent. Existing data in the new folder takes priority, and the original files remain available. If both folders already contain a campaign, back up the new save folder before copying any older files into it.

Day 1 can be spent exploring. Close a quiet shift to move on without an attack. High exposure can trigger a wave at closing time, and major leaks can start one early. A courier visit or leaked clip can change the timing. An early crowd has three fans, and each tag costs a $5,000 free PC. The unpaid amount becomes debt and future payouts repay it. A spare PC built in CarterStudio covers one tag. Late-game crowds can exceed 100 across reinforcements, with a ceiling of 240 total.

During a breach, repel the crowd to keep the office, or press **E** at the chute beside the window in the original office, or on the back wall at the repair shop. The chute drops you into the service district with the same day, equipment and money. Follow the green beacon or minimap, use the streets and gadgets to lose the pursuing fans, and press **E** at the marked door of the **New Office**. Gold PC and Pi depots each offer a pickup. Press E nearby to claim one spare PC or two Pi decoys. You have one claim per escape. Fans sprint faster than Carter on open streets. Only that door ends the chase and starts the next shift. You keep your campaign but forfeit the abandoned shift's pay. The New Office is a converted repair shop, with CarterOS on the left workbench and package pickup at the marked floor bay beside the front door. Checkpoints store which office you are in. Quitting mid-chase resumes from the last safe office checkpoint.

CarterMail is the steadier source of work. Hardware reviews pay $2,250 to $3,300 and higher-exposure live tours pay $3,600 to $5,850. A spare build costs $1,450 in parts. CarterStudio has one upload per shift, a remote build consult, and a spare-PC build option. Uploads can attract repair clubs, stock shortages, sponsor exclusivity requests, outage complaints and location leaks. Your reply can change the exposure, pay a bonus, or set up a different offer the next shift. Install CarterStudio and FanCord through the in-game Apple Store. Rare FanCord collaborations promise $7,800. You can request verification and a smaller remote deal through CarterMail. Bad deals can expose your address, leak your footage, or take your cash.

There is no fixed final day. A normal shift pays $900 gross, plus a $450 privacy bonus when eligible. Payouts repay bailout, PC and operating debt before the remaining cash reaches your balance. Operating costs start on Day 12 and rise over time. CarterMail shows the charge before closing, and unpaid costs are deducted from later earnings. The title screen tracks the best shift completed in a clean run.

## Inbox generation

The game assembles new email subjects and two lines of copy locally on first launch and each shift. The built-in generator uses a small grammar and works offline. Senders, payout terms, exposure, and scanner results are fixed by the game. An installed Ollama model can optionally write alternative copy ahead of upcoming shifts and contribute written comments in court. Install Ollama and run `ollama pull qwen3.5:0.8b` to use the default model. Set `CARTEROS_AI_MODEL` to choose a different installed tag or to `off` for the built-in generator only.

## Court speech

- The clerk, juror, and judge have distinct voices. Their questions and rulings play on both platforms with F2 to toggle narration. Subtitles are always available.
- **F8** starts microphone setup. Wait for the recording indicator, then speak in English for up to seven seconds. F8 or Esc cancels. You can edit the transcript before submitting it with Enter or type your statement from scratch.
- Microphone input requires Python and a supported default input device. Install the optional packages with `python -m pip install -r scripts/court-voice-requirements.txt` (or use `python3` on Linux). Linux may need PortAudio.
- Download and extract [the small US-English Vosk model](https://alphacephei.com/vosk/models) into `speech/vosk-model-small-en-us-0.15` in the game folder. Alternatively, point `CARTEROS_VOSK_MODEL` to the extracted folder. Set `CARTEROS_SPEECH_PYTHON` to your virtual environment's Python executable if you use one.
- Recognition runs on your computer and audio stays in memory. The optional Ollama dialogue can receive only the statement you submit. The verdict follows the exhibits and your answers. Esc returns you to the title from the hearing, and Continue resumes from the latest recorded statement.

## Build from source

Use stable Rust and CMake with a C/C++ toolchain and Clang/libclang for raylib bindings. On Linux also install the X11 and OpenGL development packages listed in the CI workflow.

```sh
cargo test --locked
cargo build --release --locked
```

The build is the standalone edition. Local achievements work without Steam.

## Court and finale — contains spoilers

The fictional Finanzamt starts assessing tax above $30,000 in business revenue. The office and CarterOS show the balance, and F7 files a payment. Routine tax never sends the player to a hearing.

Examiners review powder receipts, restricted armory invoices, and overdue filings at shift close. An active case needs two distinct findings, including a flagged purchase. The earliest summons arrives on Day 7. Accept it in CarterMail to attend, defer it until the next shift, or reopen it through `finanzamt://archiv` in CarterWeb. Saved cases from earlier builds can still use the archive. The hearing gives you three statements and Tab opens the exhibits. Carter represents himself unless you spend $2,400 of documented earnings on counsel with F4 before the first statement. Compare the balance, verified receipts, and service record. Every case has a contradiction, and a successful defense earns **Tax Evader** and reopens the office.

On a guilty verdict, press Enter to face the Finanzamt in a top-down arena with your combat inventory. Use WASD, Shift, aim and click, and the wheel to cycle items. Clear the clerks to drop the shield. Dodge the targeted **Steuerbescheid** and move out of the circle before **FRIST ABGELAUFEN** lands. At half health, dodge the follow-up **Nachforderung** aimed at where you moved. A victory reveals the final notice. Leaving the arena early returns to title and Continue restarts the fight at full health. A completed outcome is saved. If you lose, press R to retry the appeal.

This is a fictional hearing and economy.
