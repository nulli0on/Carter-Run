# RUN CARTER v0.10 — Achievements & Easter eggs

**Status:** Gameplay and UI source changes are implemented, but **the Rust application has NOT been compiled or play-tested here**. All user-provided Figma artwork is unchanged. No real internet requests are made for cheat URLs or Easter-egg email addresses.

## Achievements

All IDs below are **local** until the corresponding API ID is created and published in the game's own Steamworks application and the game is built/run with `--features steam` under Steam. Steam submissions are queued persistently; a successful `StoreStats` call is a *request*, not confirmation of final synchronization.

| Steam API ID | Display name | Unlock condition |
|---|---|---|
| `FULL_ARSENAL_ZERO_WASTE` | Full Arsenal, Zero Waste | Buy the Raspberry Pi, Airhorn, Confetti Cannon, and Drone in a single clean run; have >$0 remaining and use no emergency bailout |
| `AGAINST_ALL_ODDS` | Against All Odds (Bullshit Survivor) | Receive a harmful "le bullshit" answer (+14 exposure), then survive a kid wave before death |
| `EDGE_OF_DETONATION` | Edge of Detonation | On the third or later Colombian Powder hit, re-dose with 0–3 seconds remaining on the countdown |
| `IRON_WILL` | Iron Will | Trigger a third Colombian Powder hit, then complete the day-20 campaign milestone without detonating |
| `PURIST_MASTER` | Purist Master | Complete day 20 with at least 8 waves survived, no cheats, no detonations, no bailouts, and no powder |
| `SO_CLOSE` | SO CLOSE | Get 3–4 consecutive "le bullshit" answers, broken by a normal answer |
| `UNLUCKY` | Unlucky | Get 5 consecutive "le bullshit" answers; a dead-air/no-result response resets the streak |
| `ADDICT` | Addict | Take Colombian Powder three times in one run; begins the cartoon detonation clock |
| `CROWD_CONTROL` | 1,000 Beggars / Crowd Control | Cumulatively repel 1,000 active kids when waves finish across clean runs; these aren't actual kills |
| `SUGAR_FIEND` | Sugar Fiend | 50 Colombian Powder uses or 25 emergency re-doses with 3 seconds or less across clean runs |
| `AI_VICTIM` | AI Victim | Encounter 50 "le bullshit" answers across clean runs |
| `UNTOUCHABLE_LEGEND` | Untouchable Legend | Complete day 20 after surviving 8+ waves without being touched |
| `ZERO_SHAME_TOTAL_SWEEP` | Zero Shame, Total Sweep | Earn all 12 non-cheat, non-meta achievements across clean runs and finish the unlocks in a clean run |
| `SHAME_SHAME_SHAME` | Shame, Shame, Shame! | First cheat URL, cheat email, or enablement of debug commands |
| `SHAME_SHAME_SHAME_SHAME` | Shame, Shame, Shame, Shame! | Start with a modified v0.10-signed campaign save |
| `SHAME_TIME_TRAVELER` | Shame, Shame, Shame: Time Traveler | Large suspicious system-clock jump while the game is running (heuristic, prone to legitimate system adjustments) |
| `SHAME_MEMORY_GREMLIN` | Shame, Shame, Shame: Memory Gremlin | Unexplained in-frame cash jump >$50,000 while clean (heuristic) |
| `SHAME_OUTSIDE_CONTRACTOR` | Shame, Shame, Shame: Outside Contractor | Campaign save checksum changes outside the game while it is running (heuristic) |
| `TAX_EVADER` | Tax Evader | Get the fictional Finanzamt case dismissed during a clean run |
| `TOTALLY_LEGIT` | Totally Legit (Trust Me Bro) | Earn every other listed achievement across all runs |

**Clean-run rule:** A cheat (or tamper detection) immediately disables further non-cheat achievements for that run only. Previously earned achievements remain. Start a new run with **R** from the caught screen to restore clean eligibility. The shame/tamper achievements can still be earned on flagged runs. Multiple separate runs are intentionally required for the entire achievement collection.

**Probability note:** At the fixed 17% comedic-answer chance, 18 consecutive comedic responses has probability `0.17^18 ≈ 1.4 × 10^-14`; 50 has probability `0.17^50 ≈ 3.3 × 10^-39`. SO CLOSE and Unlucky are effectively inaccessible in normal play. No pity system is implemented. For QA, use the `ai bullshit` debug command, but debug mode correctly disables normal achievements for that run.

## Gameplay additions

* Fictional sugar packs can appear randomly when waves start or at the end of shifts (9% chance per event), or be purchased at AmazonCrime for $180. Collect from the desk with **R**. First two uses give a short-lived movement speed boost; the third starts a 75-second **cartoon** explosion countdown. Another pickup resets it and can earn Edge of Detonation. No real-world substance use is simulated or recommended.
* AmazonCrime now offers 8 catalog items. **Z** uses the Airhorn (repels nearby kids), **X** uses the Confetti Cannon (distracts/repels the active group), **C** deploys a 24-second Drone decoy; **Q** still deploys the Raspberry Pi.
* **F6** requests an emergency bailout when cash is below $80, once per day: +$400, $450 debt repaid from shift/wave payouts, +18 exposure, and a permanent bailout mark for the current run.
* A campaign completion milestone appears at **day 20**. Every second completed shift can trigger an unavoidable leak/wave to keep progression from being entirely risk-free. Play continues in endless mode after completion.
* CarterSearch retains **74% AI attempt / 17% satire / 9% blank response**. Satire increases exposure by 14%. Real model answers require optional `CARTEROS_AI_MODEL`; otherwise fallback results are labeled as such. No-answer results deliberately show no answer text.

## Debug inspector

Press **F12** or **~** to open the in-game console. Merely opening it prints: *"The player only wants to look at the code, right?"* and does not forfeit clean-run eligibility. `debug on` enables commands and triggers the Shame cutoff immediately. `help`, `status` are read-only.

Available bounded **in-game** commands: `cash 1000`, `timer 2.0`, `sugar spawn`, `ai bullshit`, `ai normal`, `kids 3`, `tp desk`, `clear aggro`. They do not open a shell, run arbitrary code, touch host files, or provide real cheats outside RUN CARTER.

## Other non-cheat Easter eggs

Try fictional URL hosts like `thekidsareunionizing.example`, `calltheitraccoon.example`, `wheredidmydoorgo.example`, and `howtouninstallchildren.example`. CarterMail also responds to `jobapplication@raccoon-hr.example`, `support@chairs-with-feelings.example`, and three other secret recipients. These produce text jokes, not cheating.

## Save, integrity, and safety notes

The signature at `.run_carter/game_v10.check` is a **non-cryptographic FNV checksum**, not a robust anti-cheat mechanism. False positives are possible after system-clock adjustments and out-of-band save editing; never use it for security enforcement or accusations of real wrongdoing. The system is entirely fictional and local. v0.9 saves can be read via default values for new fields; the signature is created after the next legitimate save.

Saving or loading is intentionally restricted during attacks and active powder countdowns; restarting or modifying files outside the game still defeats simplistic checks. These are deliberate limitations of a source prototype.

## Test commands

```bash
python3 tools/verify_v10.py
cargo test
cargo run --release
# Steam (requires published achievements and a valid Steam App ID):
cargo run --release --features steam
```

**Unverified:** Rust compiler/type checking, actual gameplay, Steam initialization and live achievement unlocks were not available in the authoring environment. The provided source is not a confirmed working binary.
