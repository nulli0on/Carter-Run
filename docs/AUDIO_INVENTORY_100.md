# RUN CARTER — audio inventory v0.12

**Source:** `Carter‘s Sounds and More.zip` and its nested `Archiv.zip`; compared by SHA-256 against the earlier 25 recordings.

**Exactly 100 planned cues:** 25 previously supplied; 5 newly supplied *possible* matches requiring review; **70 marked MISSING**. 34 unique uploaded recordings total (9 genuinely new). SHA verification and MP3 metadata probing were completed in the local package, not in this public repository.

**No raw third-party MP3/GLB files have been published here.** Licenses for commercial redistribution must be verified first. A candidate match is NOT a final usable sound.

See [`docs/AUDIO_INVENTORY_100.csv`](AUDIO_INVENTORY_100.csv) for all 100 statuses.

## Still missing: 70 cues

- [ ] `os_shutdown.wav` — **MISSING**
- [ ] `os_lock.wav` — **MISSING**
- [ ] `os_unlock.wav` — **MISSING**
- [ ] `ui_window_close.wav` — **MISSING**
- [ ] `ui_error.wav` — **MISSING**
- [ ] `ui_critical_alarm.wav` — **MISSING**
- [ ] `mail_send.wav` — **MISSING**
- [ ] `mail_receive.wav` — **MISSING**
- [ ] `web_search_submit.wav` — **MISSING**
- [ ] `shop_purchase.wav` — **MISSING**
- [ ] `shop_payment_denied.wav` — **MISSING**
- [ ] `footsteps_run_wood.wav` — **MISSING**
- [ ] `footsteps_carpet.wav` — **MISSING**
- [ ] `player_jump.wav` — **MISSING**
- [ ] `player_land.wav` — **MISSING**
- [ ] `player_breath_panicked.wav` — **MISSING**
- [ ] `door_close.wav` — **MISSING**
- [ ] `door_slam.wav` — **MISSING**
- [ ] `door_lock.wav` — **MISSING**
- [ ] `door_unlock.wav` — **MISSING**
- [ ] `office_room_tone.wav` — **MISSING**
- [ ] `office_item_drop.wav` — **MISSING**
- [ ] `crowd_distant.wav` — **MISSING**
- [ ] `crowd_approach.wav` — **MISSING**
- [ ] `kid_free_pc_shout_01.wav` — **MISSING**
- [ ] `kid_free_pc_shout_02.wav` — **MISSING**
- [ ] `kids_wall_bang.wav` — **MISSING**
- [ ] `kids_door_kick.wav` — **MISSING**
- [ ] `kids_run_footsteps.wav` — **MISSING**
- [ ] `kid_repelled.wav` — **MISSING**
- [ ] `kid_hits_player.wav` — **MISSING**
- [ ] `wave_start_stinger.wav` — **MISSING**
- [ ] `wave_complete_stinger.wav` — **MISSING**
- [ ] `weapon_switch.wav` — **MISSING**
- [ ] `weapon_mag_in.wav` — **MISSING**
- [ ] `weapon_dry_fire.wav` — **MISSING**
- [ ] `weapon_ammo_pickup.wav` — **MISSING**
- [ ] `weapon_impact_wood.wav` — **MISSING**
- [ ] `weapon_impact_metal.wav` — **MISSING**
- [ ] `vector_fire_loop.wav` — **MISSING**
- [ ] `dynamite_fuse.wav` — **MISSING**
- [ ] `minigun_spin_up.wav` — **MISSING**
- [ ] `minigun_spin_down.wav` — **MISSING**
- [ ] `flamethrower_ignite.wav` — **MISSING**
- [ ] `flamethrower_loop.wav` — **MISSING**
- [ ] `flamethrower_stop.wav` — **MISSING**
- [ ] `rpg_launch.wav` — **MISSING**
- [ ] `rpg_rocket_flight.wav` — **MISSING**
- [ ] `knife_swing.wav` — **MISSING**
- [ ] `grenade_pin_pull.wav` — **MISSING**
- [ ] `cardboard_gun_pop.wav` — **MISSING**
- [ ] `delivery_doorbell.wav` — **MISSING**
- [ ] `delivery_box_set_down.wav` — **MISSING**
- [ ] `package_open.wav` — **MISSING**
- [ ] `weapon_crate_open.wav` — **MISSING**
- [ ] `raspberry_pi_decoy_beep.wav` — **MISSING**
- [ ] `airhorn.wav` — **MISSING**
- [ ] `confetti_cannon.wav` — **MISSING**
- [ ] `drone_motor.wav` — **MISSING**
- [ ] `sugar_pickup.wav` — **MISSING**
- [ ] `sugar_boost.wav` — **MISSING**
- [ ] `sugar_timer_tick.wav` — **MISSING**
- [ ] `sugar_timer_final_warning.wav` — **MISSING**
- [ ] `sugar_saved_last_second.wav` — **MISSING**
- [ ] `ai_bullshit_sting.wav` — **MISSING**
- [ ] `cheat_shame_sting.wav` — **MISSING**
- [ ] `music_main_menu.ogg` — **MISSING**
- [ ] `music_office_calm.ogg` — **MISSING**
- [ ] `music_chase_loop.ogg` — **MISSING**
- [ ] `music_end_run.ogg` — **MISSING**

## Candidate matches (must listen)

- `ui_window_open.wav` ← `soundshelfstudio-ui-swipe-confirm-522221.mp3` — **CANDIDATE_REVIEW**
- `weapon_draw.wav` ← `freesound_community-pistol-draw-unholster-80799.mp3` — **CANDIDATE_REVIEW**
- `weapon_holster.wav` ← `freesound_community-gun-holster-44888.mp3` — **CANDIDATE_REVIEW**
- `weapon_mag_out.wav` ← `freesound_community-handgun-clip-out-106437.mp3` — **CANDIDATE_REVIEW**
- `weapon_bolt_chamber.wav` ← `oxidvideos-weapon-cocking-618522 2.mp3` — **CANDIDATE_REVIEW**

## Unassigned recordings

`freesound_community-police-357-pistol-unholster-48003.mp3`, `freesound_community-holster-pistol-7132.mp3`, `creatorshome-draw-a-sword-327726.mp3`, `freesound_community-taking-pills-33981.mp3` are preserved in the local bundle but not falsely marked as satisfying any planned cue.

## Local integration status

The local v0.12 bundle contains 34 deduplicated MP3 recordings, a full-hash manifest, and changes to `src/main.rs` that request uploaded MP3s for UI click, achievement unlock and weapon draw with fallback behavior. **Neither v0.12 nor v0.11 has passed a real Cargo build.**
