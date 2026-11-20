//! RUN CARTER v0.7: exact Figma CarterOS exported assets integrated in 3D office.
//! E at the desk opens CarterOS; source layers are untouched Figma SVG exports.
//! Office geometry/collision remains a prototype; gameplay and desktop run in native Rust.

mod carteros;
mod system_state;
mod drive;
mod game_save;
mod nav;
mod cheats;
mod web_search;
mod steam_bridge;
mod achievements;
mod debug_console;
mod integrity;
mod arsenal;
mod audio_events;
mod weapon_animation;
use audio_events::Cue;
use weapon_animation::{Motion as WeaponMotion, WeaponVisual};
use achievements::Tracker;
use std::time::{SystemTime,UNIX_EPOCH,Instant};
use debug_console::{Command as DebugCommand,Console as DebugConsole};
use cheats::CheatKind;
use raylib::prelude::*;
use raylib::core::audio::RaylibAudio;
use raylib::core::drawing::RaylibTextureModeExt;
use carteros::{Action as OsAction, Assets as OsAssets, Context as OsContext, Input as OsInput, MailView, CarterOs};
use std::f32::consts::PI;
use game_save::{Snapshot,SavedOrder};

const EYE_HEIGHT: f32 = 1.58;
const WALK_SPEED: f32 = 1.9;
const SPRINT_SPEED: f32 = 3.2;
const WAVE_LENGTH: f32 = 35.0;
const CAMPAIGN_VICTORY_DAY: u32 = 20;
const SUGAR_DETONATION_SECONDS: f32 = 75.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Office,
    Computer,
    Caught,
}

#[derive(Clone, Copy)]
struct Email {
    sender: &'static str,
    subject: &'static str,
    first_line: &'static str,
    second_line: &'static str,
    clue: &'static str,
    payout: i32,
    leak: f32,
}

const EMAILS: [Email; 10] = [
    Email {
        sender: "little_timmy88@webmail.fake",
        subject: "free pc????",
        first_line: "Dear Carter I am your biggest fan.",
        second_line: "I need a 5090 gaming PC by TOMORROW.",
        clue: "Normal begging. No suspicious attachment.",
        payout: 0,
        leak: 9.0,
    },
    Email {
        sender: "press@bytebubble.example",
        subject: "$200 keyboard sponsorship",
        first_line: "We want a 30-second keyboard review.",
        second_line: "No downloads needed; please confirm.",
        clue: "A genuine offer. No download requested.",
        payout: 200,
        leak: 13.0,
    },
    Email {
        sender: "help@appl3-security.fake",
        subject: "URGENT: confirm your home address",
        first_line: "Your devices were locked for safety.",
        second_line: "Click to install our address verifier.",
        clue: "FAKE DOMAIN: 'appl3'. The link is malicious.",
        payout: 0,
        leak: 88.0,
    },
    Email {
        sender: "free-pc-department@fans.fake",
        subject: "WHERE IS MY PC CARTER",
        first_line: "I HAVE BEEN WAITING FOR ELEVEN MINUTES.",
        second_line: "I AM CALLING ALL MY FRIENDS.",
        clue: "An annoying email, not a phishing attack.",
        payout: 0,
        leak: 18.0,
    },
    Email {
        sender: "partners@coolcase.example",
        subject: "New case review - $350",
        first_line: "Can we ship you our latest PC case?",
        second_line: "We have an ordinary written agreement.",
        clue: "A real sponsorship in this fictional world.",
        payout: 350,
        leak: 25.0,
    },
    Email {
        sender: "admin@streem-tips.fake",
        subject: "Your private stream was leaked!",
        first_line: "Download the rescue patch RIGHT NOW.",
        second_line: "Do not tell anyone about this message.",
        clue: "High-pressure scam with a suspicious patch.",
        payout: 0,
        leak: 73.0,
    },
    Email {
        sender: "mom@family.fake",
        subject: "eat something",
        first_line: "You have been in that office all day.",
        second_line: "Also, somebody is shouting outside.",
        clue: "Family message. No software involved.",
        payout: 0,
        leak: 0.0,
    },
    Email {
        sender: "giveaways@official-looking.fake",
        subject: "You won 800000 PCs!!",
        first_line: "We have selected your secret address.",
        second_line: "Open our installer to claim the prize.",
        clue: "Extremely suspicious unsolicited prize.",
        payout: 0,
        leak: 95.0,
    },
    Email {
        sender: "editor@collab.example",
        subject: "Short video-editing contract",
        first_line: "Can you record a PC build this week?",
        second_line: "We can pay $160 after you respond.",
        clue: "Normal client contract. It looks legitimate.",
        payout: 160,
        leak: 12.0,
    },
    Email {
        sender: "unknown@nearby.fake",
        subject: "we are getting closer",
        first_line: "HELLO CARTER. WE CAN SEE THE LIGHTS.",
        second_line: "PLEASE LEAVE THE RTX 5090 AT THE DOOR.",
        clue: "Not malware. Just extremely unsettling.",
        payout: 0,
        leak: 40.0,
    },
];

#[derive(Clone, Copy)]
struct Kid {
    x: f32,
    z: f32,
    speed: f32,
    variant: usize,
}

#[derive(Clone, Copy)]
struct Decoy {
    x: f32,
    z: f32,
    remaining: f32,
}

#[derive(Clone, Copy)]
struct Delivery {
    item: u8,         // 1 = Raspberry Pi; 3 = scanner refill
    eta: f32,         // Seconds until available at the door
    ready: bool,
}

struct Game {
    screen: Screen,
    cheat_state:cheats::CheatState,
    x: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
    money: i32,
    exposure: f32,
    day: u32,
    inbox_index: usize,
    scans: u32,
    inspected: bool,
    raspberry_pis: u32,
    door_upgrade: bool,
    phone_cooldown: f32,
    omni_last_day: u32,
    alarm: f32,
    active_wave: f32,
    kids: Vec<Kid>,
    decoy: Option<Decoy>,
    orders: Vec<Delivery>,
    delivery_notice: bool,
    repelled_this_wave:u32,
    finished_day: bool,
    message: String,
    message_seconds: f32,
    achievements: Tracker,
    arsenal: arsenal::Arsenal,
    airhorns:u32, confetti_cannons:u32, drones:u32,
    sugar_visible:bool, sugar_hits:u32, sugar_timer:f32, sugar_boost:f32,
    campaign_won:bool, rng_state:u64,
    bailout_debt:i32,bailout_last_day:u32,
    audio_events:audio_events::Queue,
}

impl Game {
    fn new() -> Self {
        let achievements=Tracker::load();
        let mut cheat_state=cheats::CheatState::read();
        // A clean new run must not inherit cheat powers from an older campaign.
        if !achievements.run.cheated{cheat_state.god_mode=false;cheat_state.noclip=false;}
        Self {
            screen: Screen::Office,
            cheat_state,
            x: 0.00,
            z: -0.80,
            yaw: PI / 2.0,
            pitch: 0.0,
            money: 180,
            exposure: 12.0,
            day: 1,
            inbox_index: 0,
            scans: 2,
            inspected: false,
            raspberry_pis: 1,
            door_upgrade: false,
            phone_cooldown: 0.0,
            omni_last_day: 0,
            alarm: -1.0,
            active_wave: -1.0,
            kids: Vec::new(),
            decoy: None,
            orders: Vec::new(),
            delivery_notice: false,
            repelled_this_wave:0,
            finished_day: false,
            message: String::from("Welcome to RUN CARTER. Answer emails, make money, survive."),
            message_seconds: 8.0,
            achievements,
            arsenal:arsenal::Arsenal::new(),
            airhorns:0,confetti_cannons:0,drones:0,
            sugar_visible:false,sugar_hits:0,sugar_timer:-1.0,sugar_boost:0.0,
            campaign_won:false,bailout_debt:0,bailout_last_day:0,
            audio_events:audio_events::Queue::default(),
            rng_state:SystemTime::now().duration_since(UNIX_EPOCH)
                .map(|x|x.as_nanos() as u64).unwrap_or(0xC4A7E2),
        }
    }

    fn from_save() -> Self {
        let mut game=Self::new();
        let report=integrity::verify(&Snapshot::path());
        if report==integrity::Result::Modified {
            game.achievements.tamper(achievements::Tamper::SaveOrConfig);
            game.inform("Save data changed outside the game. Achievement flagged; starting without that save.");
            return game;
        }
        if let Some(s)=Snapshot::load(&Snapshot::path()) {
            game.x=s.x;game.z=s.z;game.yaw=s.yaw;game.pitch=s.pitch;
            game.money=s.money;game.exposure=s.exposure;game.day=s.day;
            game.inbox_index=s.inbox_index;game.scans=s.scans;
            game.raspberry_pis=s.raspberry_pis;game.door_upgrade=s.door_upgrade;
            game.phone_cooldown=s.phone_cooldown;game.omni_last_day=s.omni_last_day;
            game.orders=s.orders.into_iter().map(|o|Delivery{item:o.item,eta:o.eta,ready:o.ready}).collect();
            if let Some(arsenal)=arsenal::Arsenal::restore(s.weapon_owned,s.weapon_charges,s.weapon_selected as usize) {
                game.arsenal=arsenal;
            }
            game.airhorns=s.airhorns;game.confetti_cannons=s.confetti_cannons;
            game.drones=s.drones;game.sugar_hits=s.sugar_hits;game.sugar_timer=s.sugar_timer;
            game.sugar_visible=s.sugar_visible;game.sugar_boost=s.sugar_boost;
            game.campaign_won=s.campaign_won;
            game.bailout_debt=s.bailout_debt;game.bailout_last_day=s.bailout_last_day;
            game.inform("Campaign restored. CarterOS uses a separate save for its settings.");
        }
        game
    }
    fn snapshot(&self)->Snapshot {
        Snapshot{x:self.x,z:self.z,yaw:self.yaw,pitch:self.pitch,
            money:self.money,exposure:self.exposure,day:self.day,
            inbox_index:self.inbox_index,scans:self.scans,raspberry_pis:self.raspberry_pis,
            door_upgrade:self.door_upgrade,phone_cooldown:self.phone_cooldown,
            omni_last_day:self.omni_last_day,
            orders:self.orders.iter().map(|o|SavedOrder{item:o.item,eta:o.eta,ready:o.ready}).collect(),
            airhorns:self.airhorns,confetti_cannons:self.confetti_cannons,drones:self.drones,
            sugar_hits:self.sugar_hits,sugar_timer:self.sugar_timer,sugar_visible:self.sugar_visible,
            sugar_boost:self.sugar_boost,campaign_won:self.campaign_won,
            weapon_owned:self.arsenal.ownership_mask(), weapon_charges:self.arsenal.charges,
            weapon_selected:self.arsenal.selected as u8,
            bailout_debt:self.bailout_debt,bailout_last_day:self.bailout_last_day}
    }
    fn save_campaign(&self)->Result<(),String>{
        if self.alarm>=0.0 || self.active_wave>=0.0 || self.screen==Screen::Caught || self.sugar_timer>=0.0 {
            return Err("Cannot save during an attack. Reach safety first.".to_owned());
        }
        self.snapshot().save(&Snapshot::path()).map_err(|e|format!("Save failed: {e}"))?;
        integrity::seal(&Snapshot::path()).map_err(|e|format!("Save signature failed: {e}"))?;
        Ok(())
    }
    fn random_percent(&mut self,percent:u32)->bool {
        self.rng_state^=self.rng_state<<13;
        self.rng_state^=self.rng_state>>7;
        self.rng_state^=self.rng_state<<17;
        (self.rng_state%100)<percent as u64
    }
    fn maybe_spawn_sugar(&mut self) {
        if !self.sugar_visible && self.random_percent(9) {
            self.sugar_visible=true;
            self.inform("A mysterious packet of FICTIONAL SUGAR appeared by the desk. R to collect.");
        }
    }
    fn take_sugar(&mut self) {
        if !self.sugar_visible || (self.x-0.10).hypot(self.z+1.65)>0.85 {
            self.inform("No mysterious sugar packet nearby.");return;
        }
        self.sugar_visible=false;
        self.audio_events.push(Cue::SugarPickup);
        let remaining=self.sugar_timer;
        self.achievements.sugar();
        self.sugar_hits=self.sugar_hits.saturating_add(1);
        self.sugar_boost=25.0; // short-lived comic speed boost; NOT a real-world effect.
        if self.sugar_hits>=3 {
            if remaining>0.0 {self.achievements.emergency_save(remaining);}
            self.sugar_timer=SUGAR_DETONATION_SECONDS;
            self.audio_events.push(Cue::SugarWarning);
            self.inform(if remaining>0.0 && remaining<=3.0 {
                "EMERGENCY SUGAR SAVE! You were milliseconds from cartoon detonation."
            }else{"Third sugar hit: absurd cartoon countdown started. Find more FICTIONAL sugar!"});
        }else{self.inform("Fictional sugar rush! Temporary speed boost.");}
    }
    fn fire_equipped(&mut self) {
        let Some(weapon)=self.arsenal.fire() else {
            self.audio_events.push(Cue::WeaponEmpty);
            self.inform("Equipment recharging or out of cartridges. Buy a refill in AmazonCrime.");return;
        };
        self.audio_events.push(Cue::WeaponFire(self.arsenal.selected as u8));
        let mut candidates:Vec<(usize,f32)>=self.kids.iter().enumerate().filter_map(|(i,k)|{
            let dx=k.x-self.x;let dz=k.z-self.z;
            let range=dx.hypot(dz);
            let facing=(dx*self.yaw.sin()+dz*self.yaw.cos())/range.max(0.001);
            if range<=weapon.range && facing>0.57 {Some((i,range))} else {None}
        }).collect();
        candidates.sort_by(|a,b|a.1.total_cmp(&b.1));
        let mut hit:Vec<usize>=candidates.iter().take(weapon.burst).map(|(i,_)|*i).collect();
        hit.sort_unstable_by(|a,b|b.cmp(a));
        let count=hit.len();for i in hit{self.kids.remove(i);}
        self.repelled_this_wave=self.repelled_this_wave.saturating_add(count as u32);
        self.inform(&format!("{} fired a harmless comedy effect; {} fan(s) distracted.",weapon.name,count));
    }
    fn use_airhorn(&mut self) {
        if self.airhorns==0{self.inform("No airhorns. Buy one from AmazonCrime.");return;}
        self.airhorns-=1;self.audio_events.push(Cue::Airhorn);
        let before=self.kids.len();
        self.kids.retain(|k|(k.x-self.x).hypot(k.z-self.z)>1.2);
        self.repelled_this_wave=self.repelled_this_wave.saturating_add((before-self.kids.len()) as u32);
        self.inform(&format!("AIRHORN! {} nearby kids ran away.",before-self.kids.len()));
    }
    fn use_confetti(&mut self) {
        if self.confetti_cannons==0{self.inform("No confetti cannons available.");return;}
        self.confetti_cannons-=1;self.audio_events.push(Cue::Confetti);
        let before=self.kids.len();self.kids.clear();
        self.repelled_this_wave=self.repelled_this_wave.saturating_add(before as u32);
        self.inform(&format!("CONFETTI! {} kids distracted by an imaginary giveaway.",before));
    }
    fn deploy_drone(&mut self) {
        if self.drones==0{self.inform("No decoy drones available.");return;}
        if self.decoy.is_some(){self.inform("A distraction is already deployed.");return;}
        self.drones-=1;self.audio_events.push(Cue::Drone);
        self.decoy=Some(Decoy{x:self.x,z:self.z,remaining:24.0});
        self.inform("Drone decoy deployed: 24 seconds of fake free-PC announcements.");
    }
    fn check_victory(&mut self) {
        if !self.campaign_won && self.day>=CAMPAIGN_VICTORY_DAY {
            self.campaign_won=true;
            self.achievements.victory();
            self.inform("CAMPAIGN COMPLETE! You survived 20 days. Endless office mode unlocked.");
        }
    }
    fn run_debug(&mut self,command:DebugCommand,enabled:&mut bool)->String {
        match command {
            DebugCommand::Help=>"help | status | debug on | cash N | timer S | sugar spawn | ai bullshit | ai normal | kids N | tp desk | clear aggro".into(),
            DebugCommand::Status=>format!("Day {} | ${} | exposure {:.0}% | sugar timer {:.1} | cheated {}",
                self.day,self.money,self.exposure,self.sugar_timer,self.achievements.run.cheated),
            DebugCommand::DebugOn=>{
                if !*enabled {
                    *enabled=true;self.achievements.cheat();self.cheat_state.applied();
                    let _=self.cheat_state.write();
                }
                "DEBUG ENABLED. Achievements disabled for this run. You already chose Shame.".into()
            },
            action if !*enabled=>{
                format!("{:?} requires 'debug on'. Looking at code is free; cheating isn't.",action)
            },
            DebugCommand::Cash(n)=>{self.money=self.money.saturating_add(n).min(5_000_000);format!("Granted ${n} (fictional).")},
            DebugCommand::Countdown(seconds)=>{self.sugar_timer=seconds;format!("Cartoon explosion timer set to {seconds:.1} seconds.")},
            DebugCommand::SugarSpawn=>{self.sugar_visible=true;"Fictional sugar appeared by desk. R to collect.".into()},
            DebugCommand::AiBullshit=>{self.exposure=(self.exposure+14.0).min(100.0);self.achievements.bullshit();self.check_new_exposure();"Forced 'le bullshit' AI roll (+14 exposure).".into()},
            DebugCommand::AiNormal=>{self.achievements.answer_normal();"Forced normal AI roll.".into()},
            DebugCommand::Kids(n)=>{for i in 0..n {
                self.kids.push(Kid{x:0.7,z:2.25,speed:0.6,variant:i as usize%3});
            }if self.active_wave<0.0{self.active_wave=WAVE_LENGTH;}format!("Spawned {n} fictional fans.")},
            DebugCommand::Teleport=>{self.x=0.0;self.z=-0.80;"Teleported to desk.".into()},
            DebugCommand::ClearAggro=>{self.exposure=0.0;self.alarm=-1.0;self.active_wave=-1.0;self.kids.clear();"Aggro cleared.".into()},
            DebugCommand::Unknown=>"Unknown or malformed command. Type help.".into(),
        }
    }
    fn emergency_bailout(&mut self) {
        if self.money>=80 {self.inform("Bailouts are only offered below $80 cash.");return;}
        if self.bailout_last_day==self.day {
            self.inform("Only one embarrassing bailout is permitted per day.");return;
        }
        if self.alarm>=0.0||self.active_wave>=0.0 {
            self.inform("Bailouts unavailable during a fan invasion.");return;
        }
        self.bailout_last_day=self.day;
        self.bailout_debt=self.bailout_debt.saturating_add(450);
        self.money+=400;
        self.exposure=(self.exposure+18.0).min(100.0);
        self.achievements.bailout();
        self.inform("EMERGENCY BAILOUT: +$400; repay $450 from future earnings. Zero Waste forfeited.");
        self.check_new_exposure();
    }
    fn repay_debt(&mut self,payout:i32)->i32 {
        let paid=self.bailout_debt.min(payout);
        self.bailout_debt-=paid;
        payout-paid
    }
    fn at_delivery_door(&self)->bool {
        self.x.abs()<0.5 && self.z>1.5
    }
    fn ready_deliveries(&self)->usize {self.orders.iter().filter(|o|o.ready).count()}
    fn pending_deliveries(&self)->usize {self.orders.iter().filter(|o|!o.ready).count()}
    fn next_delivery_eta(&self)->f32 {
        self.orders.iter().filter(|o|!o.ready).map(|o|o.eta)
            .min_by(|a,b|a.total_cmp(b)).unwrap_or(-1.0)
    }
    fn collect_deliveries(&mut self) {
        if !self.at_delivery_door(){self.inform("Move to the front door to collect packages.");return;}
        let mut pis=0u32;let mut scans=0u32;
        self.orders.retain(|order|{
            if !order.ready{return true;}
            match order.item {1=>pis+=1,3=>scans+=3,_=>{}}
            false
        });
        if pis==0 && scans==0 {self.inform("There are no packages waiting at the door.");}
        else {
            self.raspberry_pis+=pis;self.scans+=scans;
            self.audio_events.push(Cue::Pickup);
            self.inform(&format!("Delivery collected: +{pis} Raspberry Pi, +{scans} scans."));
        }
    }
    fn inform(&mut self, info: &str) {
        self.message = info.to_string();
        self.message_seconds = 5.0;
    }

    fn use_cheat(&mut self,kind:CheatKind)->bool {
        let first=self.cheat_state.applied();
        self.achievements.cheat();
        match kind {
            CheatKind::Money=>{
                self.money=(self.money+100_000).min(5_000_000);
                self.inform("A very confused accountant wired you $100,000.");
            },
            CheatKind::GodMode=>{
                self.cheat_state.god_mode=true;
                self.inform("GOD MODE ON. The kids can touch you but cannot catch you.");
            },
            CheatKind::MaxStats=>{
                self.day=self.day.max(99);self.money=self.money.max(1_000_000);
                self.raspberry_pis=self.raspberry_pis.max(99);
                self.scans=self.scans.max(99);
                self.door_upgrade=true;self.cheat_state.god_mode=true;
                self.exposure=0.0;self.alarm=-1.0;self.active_wave=-1.0;self.kids.clear();
                self.inform("MAX STATS. Day 99, a million dollars and deeply questionable confidence.");
            },
            CheatKind::Items=>{
                self.raspberry_pis=self.raspberry_pis.saturating_add(25).min(100_000);
                self.scans=self.scans.saturating_add(50).min(100_000);
                self.door_upgrade=true;
                self.inform("OVERPOWERED ITEM PACK: 25 Pi decoys, 50 scans, reinforced door.");
            },
            CheatKind::ClearAggro=>{
                self.exposure=0.0;self.alarm=-1.0;self.active_wave=-1.0;
                self.kids.clear();self.inform("AGGRO RESET. Everybody remembered their homework.");
            },
            CheatKind::Noclip=>{
                self.cheat_state.noclip=!self.cheat_state.noclip;
                self.x=0.0;self.z=-0.80;
                self.inform(if self.cheat_state.noclip {"NOCLIP ON. Obstacles no longer matter."}
                            else {"NOCLIP OFF. You are legally solid again."});
            },
        }
        // The cheat flag persists separately, including when called during an attack.
        #[cfg(not(test))]
        let _=self.cheat_state.write();
        first
    }
    fn at_computer(&self) -> bool {
        (self.x - 0.08).abs() < 0.75 && (self.z + 0.80).abs() < 1.05
    }

    fn open_email(&mut self, accept: bool) { self.open_email_with_multiplier(accept,1.0); }
    fn open_email_with_multiplier(&mut self, accept: bool, multiplier:f32) {
        if self.inbox_index >= EMAILS.len(){
            if accept {self.end_shift();}
            else {self.inform("Inbox already empty. Press Enter to finish the shift.");}
            return;
        }
        let mail = EMAILS[self.inbox_index];
        self.audio_events.push(Cue::MailHandled);
        if accept {
            self.money += mail.payout;
            self.exposure = (self.exposure + if mail.leak >= 60.0 {mail.leak * multiplier} else {mail.leak}).min(100.0);
            if mail.leak >= 60.0 {
                self.inform("BAD LINK! Your location data was exposed.");
            } else if mail.payout > 0 {
                self.inform("Sponsorship confirmed. Your channel is gaining attention.");
            } else {
                self.inform("Email answered. The sender won't stop talking about PCs.");
            }
        } else {
            self.inform("Email deleted. No money earned and no link opened.");
        }
        self.inbox_index += 1;
        self.inspected = false;
        if self.exposure >= 100.0 && self.alarm < 0.0 && self.active_wave < 0.0 {
            self.alarm = if self.door_upgrade { 14.0 } else { 7.0 };
            self.inform("LOCATION LEAKED! The free-PC seekers are on their way!");
        }
    }

    fn end_shift(&mut self) {
        if self.alarm>=0.0 || self.active_wave>=0.0 {
            self.inform("Cannot finish your shift while the kids are attacking.");return;
        }
        if self.inbox_index<EMAILS.len() {
            self.inform("You still have unread emails today.");return;
        }
        self.day+=1;
        self.check_victory();
        self.maybe_spawn_sugar();
        self.inbox_index=0;
        self.inspected=false;
        self.exposure=(self.exposure-15.0).max(5.0);
        let reward=self.repay_debt(45);
        self.money+=reward;
        if self.day%2==0 && !self.campaign_won {
            self.exposure=100.0;self.check_new_exposure();
        }
        self.finished_day=true;
        self.inform("Shift complete! +$45. A new batch of emails has arrived.");
    }
    fn check_new_exposure(&mut self) {
        if self.exposure >= 100.0 && self.alarm < 0.0 && self.active_wave < 0.0 {
            self.alarm=if self.door_upgrade {14.0}else{7.0};
            self.inform("LOCATION LEAKED! Incoming fans!");
        }
    }
    fn inspect(&mut self) {
        if self.inbox_index>=EMAILS.len(){self.inform("No new mail left to inspect today.");return;}
        if self.inspected {
            self.inform("Already scanned. Read the inspection result below.");
        } else if self.scans > 0 {
            self.scans -= 1;
            self.inspected = true;
            self.inform("Scanner finished. Read the inspection clue.");
        } else {
            self.inform("No scans remaining. Buy more at AmazonCrime.");
        }
    }

    fn buy(&mut self, item: i32) {
        if (arsenal::FIRST_SHOP_ID..arsenal::FIRST_SHOP_ID+arsenal::COUNT as i32).contains(&item) {
            let index=(item-arsenal::FIRST_SHOP_ID) as usize;
            let result=self.arsenal.buy(index,&mut self.money);
            match result {
                Ok(message)=>{
                    self.audio_events.push(Cue::ShopPurchase);
                    self.achievements.purchase(item,self.money);
                    self.inform(&format!("{} {}",arsenal::CATALOG[index].name,message));
                },
                Err(e)=>{self.audio_events.push(Cue::ShopDenied);self.inform(e);},
            }
            return;
        }
        let (price, label) = match item {
            1 => (85, "Raspberry Pi ordered. Collect it at the front door."),
            2 => (95, "AntiDoxx digital patch installed. Exposure reduced."),
            3 => (65, "Scanner kit ordered. Collect it at the front door."),
            4 if !self.door_upgrade => (190, "Reinforced door installed. +7s warning."),
            4 => {self.inform("The door is already reinforced.");return;}
            5 => (120,"Airhorn added. Z: frighten close attackers."),
            6 => (220,"Confetti cannon added. X: distract all active kids."),
            7 => (300,"Decoy drone added. C: lure kids for 24 seconds."),
            8 => (75,"Fictional AmazonCrime sugar pack delivered to the desk. R to take."),
            _ => return,
        };
        if self.money<price {self.audio_events.push(Cue::ShopDenied);self.inform("Insufficient money. Reply to legitimate sponsors.");return;}
        if (item==1||item==3) && self.orders.len()>=12 {
            self.inform("Delivery queue full. Collect pending packages first.");return;
        }
        self.money-=price;
        self.audio_events.push(Cue::ShopPurchase);
        match item {
            1=>self.orders.push(Delivery{item:1,eta:20.0,ready:false}),
            2=>self.exposure=(self.exposure-45.0).max(0.0),
            3=>self.orders.push(Delivery{item:3,eta:15.0,ready:false}),
            4=>self.door_upgrade=true,
            5=>self.airhorns=self.airhorns.saturating_add(1),
            6=>self.confetti_cannons=self.confetti_cannons.saturating_add(1),
            7=>self.drones=self.drones.saturating_add(1),
            8=>self.sugar_visible=true,
            _=>{}
        }
        self.achievements.purchase(item,self.money);
        self.inform(label);
    }

    fn block_unknown_callers(&mut self) {
        if self.phone_cooldown>0.0 {
            self.inform("Phone protection is on cooldown. Try again later.");
            return;
        }
        self.exposure=(self.exposure-7.0).max(0.0);
        self.phone_cooldown=45.0;
        self.inform("Unknown callers blocked. Exposure -7%. Cooldown: 45 seconds.");
    }

    fn optimize_with_omni(&mut self) {
        if self.omni_last_day==self.day {
            self.inform("OmniPlus has already optimized today. No further benefit.");
            return;
        }
        self.omni_last_day=self.day;
        self.exposure=(self.exposure-20.0).max(0.0);
        self.inform("OmniPlus optimization complete. Exposure -20%.");
    }

    fn drop_decoy(&mut self) {
        if self.raspberry_pis == 0 {
            self.inform("Out of Raspberry Pis. Buy another at AmazonCrime.");
        } else if self.decoy.is_some() {
            self.inform("A Raspberry Pi is already active.");
        } else {
            self.raspberry_pis -= 1;
            self.decoy = Some(Decoy { x: self.x, z: self.z, remaining: 13.0 });
            self.audio_events.push(Cue::Drone);
            self.inform("Raspberry Pi deployed! Kids will investigate it.");
        }
    }

    fn start_wave(&mut self) {
        let count = (3 + self.day as usize).min(12);
        self.kids.clear();self.repelled_this_wave=0;
        for i in 0..count {
            self.kids.push(Kid {
                x: 1.00 - (i % 3) as f32 * 0.13,
                z: 2.42 - (i / 3) as f32 * 0.16,
                speed: 0.65 + (i % 4) as f32 * 0.07 + self.day as f32 * 0.04,
                variant: i % 3,
            });
        }
        self.active_wave = WAVE_LENGTH;
        self.audio_events.push(Cue::WaveStart);
        self.alarm = -1.0;
        self.maybe_spawn_sugar();
        self.inform("THE KIDS ARE HERE. RUN CARTER!");
    }

    fn tick(&mut self, dt: f32) {
        self.arsenal.tick(dt);
        if self.sugar_boost>0.0 {self.sugar_boost=(self.sugar_boost-dt).max(0.0);}
        if self.sugar_timer>0.0 {
            self.sugar_timer-=dt;
            if self.sugar_timer<=0.0 {
                self.sugar_timer=-1.0;
                self.achievements.detonation();
                self.audio_events.push(Cue::Detonation);
                self.screen=Screen::Caught;
                self.inform("KABOOM! Cartoon sugar overload. A spectacularly bad decision.");
                return;
            }
        }
        for order in &mut self.orders {
            if !order.ready {
                order.eta=(order.eta-dt).max(0.0);
                if order.eta<=0.0 {order.ready=true;self.delivery_notice=true;self.audio_events.push(Cue::Delivery);}
            }
        }
        if self.delivery_notice {
            self.delivery_notice=false;
            self.inform("AmazonCrime package delivered! Collect it at the front door (F).");
        }
        self.message_seconds = (self.message_seconds - dt).max(0.0);
        self.phone_cooldown=(self.phone_cooldown-dt).max(0.0);
        if let Some(mut d) = self.decoy {
            d.remaining -= dt;
            self.decoy = if d.remaining > 0.0 { Some(d) } else { None };
        }
        if self.alarm >= 0.0 {
            self.alarm -= dt;
            if self.alarm <= 0.0 { self.start_wave(); }
        }
        if self.active_wave >= 0.0 {
            self.audio_events.push(Cue::DoorPound);
            self.active_wave -= dt;
            let mut was_caught=false;
            let (tx,tz)=self.decoy.map(|d|(d.x,d.z)).unwrap_or((self.x,self.z));
            let flow=nav::Flow::new(tx,tz,free_position);
            for kid in &mut self.kids {
                let direction=flow.direction(kid.x,kid.z);
                if let Some((dx,dz))=direction {
                    let distance=dx.hypot(dz);
                    if distance>0.0001 {
                        let change=(kid.speed*dt).min(distance);
                        let nx=kid.x+dx/distance*change;
                        let nz=kid.z+dz/distance*change;
                        if free_position(nx,nz){kid.x=nx;kid.z=nz;}
                    }
                }
                if !self.cheat_state.god_mode && (kid.x-self.x).hypot(kid.z-self.z)<0.35 {
                    was_caught=true;break;
                }
            }
            if was_caught {
                self.achievements.hit();
                self.audio_events.push(Cue::PlayerCaught);
                self.screen=Screen::Caught;
                self.inform("CAN I HAVE A FREE PC?? You were caught!");
                return;
            }
            if self.active_wave <= 0.0 {
                self.audio_events.push(Cue::WaveEnd);
                self.achievements.wave_survived(self.repelled_this_wave.saturating_add(self.kids.len() as u32));
                self.repelled_this_wave=0;
                self.kids.clear();
                self.active_wave = -1.0;
                self.exposure = 25.0;
                self.day += 1;
                self.check_victory();
                self.maybe_spawn_sugar();
                self.inbox_index=0;
                self.inspected=false;
                self.finished_day=true;
                let reward=self.repay_debt(80);
                self.money += reward;
                self.scans += 1;
                self.inform("You survived! +$80, +1 scan. The next day begins.");
            }
        }
    }
}

fn free_position(x: f32, z: f32) -> bool {
    // Rough blocker volumes matching this particular uploaded office model.
    // NOT full mesh collision: this is a starter until navigation meshes are built.
    if !(-1.31..=1.09).contains(&x) || !(-2.56..=2.57).contains(&z) { return false; }
    if x > 0.33 && (-2.19..=1.16).contains(&z) { return false; } // long desk
    if x < -0.69 && (-2.35..=2.53).contains(&z) { return false; } // shelves
    true
}

fn move_player(g: &mut Game, dx: f32, dz: f32) {
    let x = g.x + dx;
    let z = g.z + dz;
    if (g.cheat_state.noclip && (-1.31..=1.09).contains(&x)) || free_position(x,g.z) { g.x = x; }
    if (g.cheat_state.noclip && (-2.56..=2.57).contains(&z)) || free_position(g.x,z) { g.z = z; }
}


fn draw_overlay(d: &mut RaylibDrawHandle<'_>, g: &Game) {
    let w = d.get_screen_width();
    let h = d.get_screen_height();
    let navy = Color::new(12, 18, 29, 240);
    let white = Color::new(235, 241, 249, 255);
    let muted = Color::new(155, 170, 186, 255);
    let red = Color::new(242, 76, 89, 255);
    let teal = Color::new(90, 219, 202, 255);
    if g.screen == Screen::Caught {
        d.draw_rectangle(0, 0, w, h, Color::new(10, 13, 20, 244));
        d.draw_text("YOU WERE CAUGHT", w/2 - 260, h/2-100, 45, red);
        d.draw_text(if g.achievements.run.detonated {"The fictional sugar countdown reached zero. Cartoon explosion!"}
            else {"A kid touched Carter. The free PC remains undelivered."},w/2-325,h/2-13,20,white);
        d.draw_text(&format!("Day {}  |  R: restart",g.day),w/2-130,h/2+58,23,teal);
        return;
    }
    d.draw_rectangle(0, 0, w, 67, navy);
    d.draw_text("RUN CARTER", 20, 11, 27, white);
    d.draw_text(&format!("DAY {} | CASH ${} | PI x{} | SCANS {} | BAILOUT DEBT ${}",g.day,g.money,g.raspberry_pis,g.scans,g.bailout_debt),20,44,16,muted);
    if g.sugar_timer>0.0 {
        d.draw_rectangle(w/2-190,156,380,46,Color::new(130,14,41,243));
        d.draw_text(&format!("CARTOON DETONATION: {:.1}s",g.sugar_timer),w/2-171,167,20,Color::WHITE);
    }else if g.sugar_visible {d.draw_text("MYSTERY SUGAR: near desk (R)",w/2-195,165,20,teal);}
    if g.achievements.run.cheated {d.draw_text("ACHIEVEMENTS DISABLED THIS RUN",w/2-178,218,19,red);}
    if g.campaign_won {d.draw_text("CAMPAIGN CLEARED - ENDLESS MODE",w/2-178,248,19,teal);}
    if g.cheat_state.god_mode {d.draw_text("GOD",w/2-30,11,21,teal);}
    if g.cheat_state.noclip {d.draw_text("NOCLIP",w/2+35,12,17,teal);}
    d.draw_text("LOCATION EXPOSURE",w-239,11,16,white);
    d.draw_rectangle(w-239,37,185,12,Color::DARKGRAY);
    d.draw_rectangle(w-239,37,(185.0*g.exposure/100.0) as i32,12,if g.exposure>70.0{red}else{teal});
    d.draw_text(&format!("{}%",g.exposure as i32),w-48,35,13,white);
    if g.alarm >= 0.0 {
        d.draw_rectangle(w/2-226,79,452,69,Color::new(117,18,28,246));
        d.draw_text("LOCATION LEAKED!",w/2-127,87,24,white);
        d.draw_text(&format!("KIDS ARRIVE IN {:.1}s",g.alarm),w/2-130,120,20,white);
    } else if g.active_wave >= 0.0 {
        d.draw_rectangle(w/2-226,79,452,69,Color::new(117,18,28,246));
        d.draw_text("RUN CARTER!",w/2-107,86,25,white);
        d.draw_text(&format!("SURVIVE {:.0}s  |  {} KIDS",g.active_wave.max(0.0).ceil(),g.kids.len()),w/2-138,121,20,white);
    }
    if g.message_seconds>0.0 {
        d.draw_rectangle(14,h-103,(w-28).min(850),33,Color::new(14,30,45,237));
        d.draw_text(&g.message,25,h-95,18,white);
    }
    d.draw_rectangle(0,h-47,w,47,navy);
    let prompt=if g.at_delivery_door() && g.ready_deliveries()>0 {"F: COLLECT DELIVERY"}
        else if g.at_computer(){"E: USE COMPUTER"}else{"APPROACH THE DESK"};
    d.draw_text(&format!("WASD | SHIFT | {prompt} | Q: Pi Z: Horn X: Confetti C: Drone R: Sugar | F12: Console"),20,h-35,15,white);
    d.draw_rectangle(15,h-85,370,64,Color::new(16,22,37,218));
    let wpn=&arsenal::CATALOG[g.arsenal.selected];
    d.draw_text(&format!("[{} / {}] {}  |  {} charges",g.arsenal.selected+1,arsenal::COUNT,wpn.name,g.arsenal.charges[g.arsenal.selected]),25,h-76,17,white);
    d.draw_text("1-0/wheel/PgUp-PgDn: equip | Click: fire | R: inspect/reload anim | B: show/hide",25,h-52,14,muted);
    d.draw_line(w/2-7,h/2,w/2+7,h/2,Color::WHITE);
    d.draw_line(w/2,h/2-7,w/2,h/2+7,Color::WHITE);
}

fn handle_os_action(a:OsAction,g:&mut Game,os:&CarterOs){
    match a {
        OsAction::Exit=>g.screen=Screen::Office,
        OsAction::AcceptMail=>{
            if os.network_online(){g.open_email_with_multiplier(true,os.phishing_multiplier());}
            else{g.inform("CarterMail: offline. Enable Wi-Fi in Settings.");}
        },
        OsAction::DeleteMail=>g.open_email(false),
        OsAction::InspectMail=>g.inspect(),
        OsAction::BuyItem(i)=>{
            if os.network_online(){g.buy(i);}else{g.inform("AmazonCrime: cannot order while offline.");}
        },
        OsAction::Spend(n)=>{g.money=(g.money-n).max(0);},
        OsAction::SecurePhone=>g.block_unknown_callers(),
        OsAction::OmniProtect=>g.optimize_with_omni(),
        OsAction::PayWage(n)=>{g.money+=n;},
        OsAction::VideoUpload=>{
            g.money+=140;g.exposure=(g.exposure+19.0).min(100.0);
            g.inform("Video uploaded. Earned $140. Your audience is growing.");
            g.check_new_exposure();
        },
        OsAction::FanPost=>{
            g.exposure=(g.exposure+10.0).min(100.0);
            g.inform("FanCord message posted. Some fans are getting curious.");
            g.check_new_exposure();
        },
        OsAction::FakeSpoof=>{
            g.exposure=(g.exposure-35.0).max(0.0);
            if g.alarm>=0.0 {g.alarm=-1.0;}
            g.inform("FakeMyLocation: decoy successfully broadcast (fictional). ");
        },
        OsAction::Cheat(kind)=>{g.use_cheat(kind);},
        OsAction::DebugEnable=>{}, // Activation handled in the input loop with debug console state.
        OsAction::SearchOutcome(kind)=>{
            match kind {
                web_search::AnswerKind::Comedic=>{
                    g.achievements.bullshit();
                    g.exposure=(g.exposure+14.0).min(100.0);
                    g.inform("LE BULLSHIT! Fake AI advice increased location exposure by 14%.");
                    g.check_new_exposure();
                },
                web_search::AnswerKind::NoResults=>g.achievements.dead_air(),
                web_search::AnswerKind::Ai|web_search::AnswerKind::OfflineFallback=>g.achievements.answer_normal(),
                _=>{},
            }
        },
    }
    // No save is written while attackers are active. Progress is committed
    // after an attack ends, preserving an honest checkpoint.
    let _=g.save_campaign();
}

fn main() {
    // CI-only bounded visual startup check: actual raylib window, no fake render path.
    // Normal gameplay stays unchanged unless this explicit argument is supplied.
    let smoke_frames=std::env::args().filter_map(|a|{
        a.strip_prefix("--smoke-frames=").and_then(|s|s.parse::<u32>().ok())
    }).find(|n|(1..=1200).contains(n));
    let smoke_desktop=smoke_frames.is_some() && std::env::args().any(|a|a=="--smoke-desktop");
    let mut rendered_frames=0u32;
    let (mut rl, thread)=raylib::init()
        .size(carteros::W,carteros::H)
        .title("RUN CARTER | CarterOS Figma edition")
        .build();
    rl.set_target_fps(60);
    // Esc closes the in-game desktop instead of terminating the program.
    rl.set_exit_key(None);
    rl.disable_cursor();
    let office=match rl.load_model(&thread,"assets/office_original_textures_complete.glb"){
        Ok(model)=>Some(model),
        Err(err)=>{eprintln!("Office asset unavailable: {err:?}");None}
    };
    let assets=OsAssets::load(&mut rl,&thread);
    let fan_model=rl.load_model(&thread,"assets/arsenal/fan_character.glb").ok();
    let crate_model=rl.load_model(&thread,"assets/arsenal/military_crate.glb").ok();
    let mut selected_visual:Option<WeaponVisual>=None;
    let mut selected_model_idx:Option<usize>=None;
    let mut pending_visual_idx:Option<usize>=None;
    let mut ui_target=rl.load_render_texture(&thread,carteros::W as u32,carteros::H as u32)
        .expect("CarterOS 1440x900 framebuffer could not be allocated");
    let mut os=CarterOs::new();
    let audio=RaylibAudio::init_audio_device().ok();
    let click_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-computer-mouse-click-352734.mp3")
        .or_else(|_|a.new_sound("assets/sfx/ui_click.wav")).ok());
    let achievement_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-achievement-unlock-243762.mp3").ok());
    let weapon_draw_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-pistol-draw-unholster-80799.mp3").ok());
    let alert_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/security_alert.wav").ok());
    // Optional Pixabay sound candidates from the owner's collection; no synthetic placeholders.
    let keyboard_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-keyboard-typing-5997.mp3").ok());
    let open_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-start-computeraif-14572.mp3").ok());
    let purchase_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-item-pickup-37089.mp3").ok());
    let denied_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-new-notification-010-352755.mp3").ok());
    let walk_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/soumages-walking-on-wood-363349.mp3").ok());
    let knock_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-intense-door-knocking-637240.mp3").ok());
    let hurt_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-oof-97698.mp3").ok());
    let explosion_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/dragon-studio-explosion-with-debris-494320.mp3").ok());
    let m4_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-m4-assault-rifle-long-burst-102682.mp3").ok());
    let machine_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/dragon-studio-machine-gun-sfx-631486.mp3").ok());
    let gunshot_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/dragon-studio-gunshot-511311.mp3").ok());
    let pickup_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/freesound_community-item-pickup-37089.mp3").ok());
    let notification_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-welcome-notification-131917.mp3").ok());
    let mut previous_alarm=false;
    let mut last_preset=2u8;
    let mut game=Game::from_save();
    if smoke_desktop{ game.screen=Screen::Computer; }
    let steam=steam_bridge::Bridge::new();
    let mut steam_retry=5.0f32;
    let mut cursor_on=false;
    let mut console=DebugConsole::default();
    let mut debug_enabled=false;
    let mut show_weapon_preview=true; // Animated first-person weapon model (B toggles).
    let mut last_money=game.money;
    let mut last_wall=SystemTime::now();
    let mut last_mono=Instant::now();
    let mut integrity_elapsed=0.0f32;
    let (mut ui_pan_x,mut ui_pan_y)=(0.0f32,0.0f32);
    while !rl.window_should_close(){
        if smoke_frames.is_some_and(|n|rendered_frames>=n){break;}
        rendered_frames=rendered_frames.saturating_add(1);
        let dt=rl.get_frame_time().min(0.05);
        let mut manual_reload_requested=false;
        game.audio_events.advance(dt);
        steam.tick();
        // In-game timers rely on monotonic frame delta, never the system clock.
        if last_mono.elapsed().as_secs()>=15 {
            let passed=last_mono.elapsed().as_secs_f64();
            let wall=match SystemTime::now().duration_since(last_wall){Ok(x)=>x.as_secs_f64(),Err(e)=>-e.duration().as_secs_f64()};
            if wall< -600.0 || (wall-passed).abs()>1800.0 {
                if !game.achievements.has("SHAME_TIME_TRAVELER"){
                    game.achievements.tamper(achievements::Tamper::ClockJump);
                    game.inform("Suspicious clock jump detected (heuristic; may be a system adjustment).");
                }
            }
            last_wall=SystemTime::now();last_mono=Instant::now();
        }
        if !game.achievements.run.cheated && (game.money as i64-last_money as i64).abs()>50_000 {
            game.achievements.tamper(achievements::Tamper::MemoryJump);
            game.inform("Unexpected cash jump detected; debug or external modification suspected.");
        }
        last_money=game.money;
        integrity_elapsed+=dt;
        if integrity_elapsed>=8.0 {
            integrity_elapsed=0.0;
            if integrity::verify(&Snapshot::path())==integrity::Result::Modified
                && !game.achievements.has("SHAME_OUTSIDE_CONTRACTOR") {
                game.achievements.tamper(achievements::Tamper::ExternalModification);
                game.inform("Save modified while playing; Outside Contractor recorded.");
            }
        }
        if !game.achievements.pending.is_empty() {
            steam_retry-=dt;
            if steam_retry<=0.0 {
                steam_retry=10.0;
                if let Some(id)=game.achievements.pending.iter().next().cloned() {
                    if steam.unlock(&id) {
                        game.achievements.pending.remove(&id);
                        let _=game.achievements.save();
                        os.notify(&format!("Steam stats submission requested: {}",Tracker::label(&id)));
                    }
                }
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F10){
            if let Err(e)=game.save_campaign(){eprintln!("Campaign not saved on exit: {e}");}
            break;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F5) {
            match game.save_campaign(){Ok(())=>game.inform("Campaign saved."),Err(e)=>game.inform(&e)}
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F9) {
            if game.alarm>=0.0||game.active_wave>=0.0||game.sugar_timer>0.0 {
                game.inform("Cannot load during an attack or cartoon sugar countdown.");
            }else if Snapshot::load(&Snapshot::path()).is_some(){
                game=Game::from_save();last_money=game.money;
            }else {game.inform("No valid campaign save found.");}
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F12) || rl.is_key_pressed(KeyboardKey::KEY_GRAVE) {
            console.open=!console.open;
            if console.open {console.log("The player only wants to look at the code, right?");}
        }
        if console.open {
            while let Some(c)=rl.get_char_pressed(){if !c.is_control(){console.write(&c.to_string());}}
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE){console.buffer.pop();}
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                let cmd=console.submit();
                let response=game.run_debug(cmd,&mut debug_enabled);
                console.log(&response);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE){console.open=false;}
            if game.screen!=Screen::Caught {game.tick(dt);}
        }else{match game.screen {
            Screen::Caught=>{
                if rl.is_key_pressed(KeyboardKey::KEY_R){
                    let mut tracker=std::mem::take(&mut game.achievements);
                    tracker.restart_clean_run();
                    game=Game::new();game.achievements=tracker;
                    last_money=game.money;
                    game.cheat_state.god_mode=false;game.cheat_state.noclip=false;
                    let _=game.cheat_state.write();
                    os=CarterOs::new();debug_enabled=false;
                    let _=game.save_campaign();
                }
            },
            Screen::Office=>{
                if rl.is_key_pressed(KeyboardKey::KEY_E) && game.at_computer(){
                    game.screen=Screen::Computer;
                    game.audio_events.push(Cue::ComputerOpen);
                }else{
                    if rl.is_key_pressed(KeyboardKey::KEY_F6){game.emergency_bailout();}
                    const KEYS:[KeyboardKey;10]=[
                        KeyboardKey::KEY_ONE,KeyboardKey::KEY_TWO,KeyboardKey::KEY_THREE,
                        KeyboardKey::KEY_FOUR,KeyboardKey::KEY_FIVE,KeyboardKey::KEY_SIX,
                        KeyboardKey::KEY_SEVEN,KeyboardKey::KEY_EIGHT,KeyboardKey::KEY_NINE,
                        KeyboardKey::KEY_ZERO,
                    ];
                    for (i,key) in KEYS.iter().enumerate(){
                        if rl.is_key_pressed(*key) && !game.arsenal.equip(i) {
                            game.inform("This model is not yet owned. Purchase it in AmazonCrime.");
                        }
                    }
                    // Scroll / PageUp / PageDown cycles ALL owned items while
                    // 1..0 keep their legacy direct-selection behavior.
                    let wheel=rl.get_mouse_wheel_move();
                    if wheel.abs()>0.01 || rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) || rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN){
                        let next=if wheel.abs()>0.01 {wheel<0.0}
                            else {rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP)};
                        game.arsenal.cycle(next);
                    }
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT){game.fire_equipped();}
                    if rl.is_key_pressed(KeyboardKey::KEY_B){show_weapon_preview=!show_weapon_preview;}
                    if rl.is_key_pressed(KeyboardKey::KEY_Q){game.drop_decoy();}
                    if rl.is_key_pressed(KeyboardKey::KEY_Z){game.use_airhorn();}
                    if rl.is_key_pressed(KeyboardKey::KEY_X){game.use_confetti();}
                    if rl.is_key_pressed(KeyboardKey::KEY_C){game.deploy_drone();}
                    if rl.is_key_pressed(KeyboardKey::KEY_R){
                        if game.sugar_visible && (game.x-0.10).hypot(game.z+1.65)<=0.85 {
                            game.take_sugar();
                        }else{
                            // Current mechanics use purchased charges, not magazines.
                            // R is a visual reload/inspection; it does NOT grant free ammo.
                            manual_reload_requested=true;
                        }
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_F){
                        game.collect_deliveries();
                        let _=game.save_campaign();
                    }
                    let md=rl.get_mouse_delta();
                    game.yaw=(game.yaw+md.x*0.0032).rem_euclid(2.0*PI);
                    game.pitch=(game.pitch-md.y*0.0032).clamp(-1.30,1.30);
                    let speed=(if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT){SPRINT_SPEED}else{WALK_SPEED})
                        *(if game.sugar_boost>0.0{1.50}else{1.0});
                    let forward=(game.yaw.sin(),game.yaw.cos());
                    let right=(game.yaw.cos(),-game.yaw.sin());
                    let (mut mx,mut mz):(f32,f32)=(0.0,0.0);
                    if rl.is_key_down(KeyboardKey::KEY_W){mx+=forward.0;mz+=forward.1;}
                    if rl.is_key_down(KeyboardKey::KEY_S){mx-=forward.0;mz-=forward.1;}
                    if rl.is_key_down(KeyboardKey::KEY_D){mx+=right.0;mz+=right.1;}
                    if rl.is_key_down(KeyboardKey::KEY_A){mx-=right.0;mz-=right.1;}
                    let magnitude=(mx*mx+mz*mz).sqrt();
                    if magnitude>0.01{
                        let (old_x,old_z)=(game.x,game.z);
                        move_player(&mut game,mx/magnitude*speed*dt,mz/magnitude*speed*dt);
                        if (game.x-old_x).hypot(game.z-old_z)>0.001 {game.audio_events.push(Cue::Footstep);}
                    }
                }
                game.tick(dt);
            },
            Screen::Computer=>{
                os.set_threat_visible(game.alarm>=0.0||game.active_wave>=0.0);
                let mouse=rl.get_mouse_position();
                // Map physical mouse coordinates through the Figma-native 1440x900
                // render surface, including letterboxing and 125% accessibility zoom.
                let sw=rl.get_screen_width() as f32;
                let sh=rl.get_screen_height() as f32;
                let zoom=if os.system.interface_scale==125 {1.25} else {1.0};
                let fit=(sw/carteros::W as f32).min(sh/carteros::H as f32)*zoom;
                let scaled_w=carteros::W as f32*fit;
                let scaled_h=carteros::H as f32*fit;
                if zoom>1.0 && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE){
                    let delta=rl.get_mouse_delta();ui_pan_x+=delta.x;ui_pan_y+=delta.y;
                }
                let range_x=((scaled_w-sw)/2.0).max(0.0);
                let range_y=((scaled_h-sh)/2.0).max(0.0);
                ui_pan_x=ui_pan_x.clamp(-range_x,range_x);
                ui_pan_y=ui_pan_y.clamp(-range_y,range_y);
                let ox=(sw-scaled_w)/2.0+ui_pan_x;
                let oy=(sh-scaled_h)/2.0+ui_pan_y;
                let logical_x=(mouse.x-ox)/fit;
                let logical_y=(mouse.y-oy)/fit;
                let mut typed=String::new();
                while let Some(c)=rl.get_char_pressed(){if !c.is_control(){typed.push(c);}}
                if !typed.is_empty(){game.audio_events.push(Cue::Keyboard);}
                let input=OsInput{
                    x:logical_x as i32,y:logical_y as i32,
                    click:rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
                    right_click:rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT),
                    down:rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
                    wheel:rl.get_mouse_wheel_move(),
                    chars:typed,
                    backspace:rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE),
                    enter:rl.is_key_pressed(KeyboardKey::KEY_ENTER),
                    escape:rl.is_key_pressed(KeyboardKey::KEY_ESCAPE),
                    lock:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_L),
                    up:rl.is_key_pressed(KeyboardKey::KEY_UP),
                    down_key:rl.is_key_pressed(KeyboardKey::KEY_DOWN),
                    left:rl.is_key_pressed(KeyboardKey::KEY_LEFT),
                    right:rl.is_key_pressed(KeyboardKey::KEY_RIGHT),
                    find:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_F),
                    save:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_S),
                    new_file:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && !rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) && rl.is_key_pressed(KeyboardKey::KEY_N),
                    new_folder:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) && rl.is_key_pressed(KeyboardKey::KEY_N),
                    rename:rl.is_key_pressed(KeyboardKey::KEY_F2),
                    toggle_trash:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_T),
                    restore:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_R),
                    cycle_window:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_TAB),
                    minimize_window:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_M),
                    close_window:(rl.is_key_down(KeyboardKey::KEY_LEFT_ALT)||rl.is_key_down(KeyboardKey::KEY_RIGHT_ALT))
                        && rl.is_key_pressed(KeyboardKey::KEY_F4),
                    maximize_window:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && rl.is_key_pressed(KeyboardKey::KEY_ENTER),
                    debug_toggle:(rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                        && (rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)||rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT))
                        && rl.is_key_pressed(KeyboardKey::KEY_D),
                };
                for a in os.handle(&input,game.money){
                    if a==OsAction::DebugEnable {
                        let response=game.run_debug(DebugCommand::DebugOn,&mut debug_enabled);
                        os.notify(&response);
                        continue;
                    }
                    let first_cheat=!game.achievements.run.cheated;
                    handle_os_action(a,&mut game,&os);
                    if let OsAction::Cheat(_)=a {
                        if first_cheat {
                            os.notify("ACHIEVEMENT: Shame, Shame, Shame! (local; Steam sync pending)");
                        } else {os.notify("Cheat activated. The shame remains permanent.");}
                        steam_retry=0.0;
                    }
                }
                if input.click && os.system.system_sounds && os.system.speakers {
                    if let Some(ref sfx)=click_sound{sfx.play();}
                }
                if game.screen==Screen::Computer {
                    for a in os.tick(dt){handle_os_action(a,&mut game,&os);}
                    // Dangerous activity outside the monitor continues while seated.
                    game.tick(dt);
                }
            },
        }
        for id in game.achievements.take_new() {
            if os.system.speakers && os.system.system_sounds {
                if let Some(ref sfx)=achievement_sound{sfx.play();}
            }
            os.notify(&format!("ACHIEVEMENT: {}",Tracker::label(&id)));
            game.inform(&format!("Achievement unlocked: {}",Tracker::label(&id)));
        }
        } // end normal screen input (console captures input when open)
        if game.delivery_notice {
            os.notify("AmazonCrime: package waiting at the front door.");
            game.delivery_notice=false;
            let _=game.save_campaign();
        }
        if game.finished_day {
            os.notify("Day complete! New emails have arrived.");
            game.finished_day=false;
            let _=game.save_campaign();
        }
        // Rendering scales the unchanged Figma artboard rather than cropping it.
        if last_preset!=os.system.display_preset {
            let (width,height)=if os.system.display_preset==0 {(1440,900)}else{(1280,720)};
            rl.set_window_size(width,height);
            last_preset=os.system.display_preset;
        }
        if let Some(ref a)=audio {
            let volume=if os.system.speakers {os.system.master_volume as f32/100.0}else{0.0};
            a.set_master_volume(volume);
        }
        let now_alarm=game.alarm>=0.0||game.active_wave>=0.0;
        if now_alarm && !previous_alarm {
            if let Some(ref sfx)=alert_sound{sfx.play();}
        }
        previous_alarm=now_alarm;
        if game.screen==Screen::Office && selected_model_idx!=Some(game.arsenal.selected) {
            let target=game.arsenal.selected;
            if pending_visual_idx.is_some(){
                // If player cycles twice while a holster is finishing, keep the latest selection.
                pending_visual_idx=Some(target);
            }else if selected_visual.as_mut().is_some_and(|v|v.trigger(WeaponMotion::Holster)){
                pending_visual_idx=Some(target);
            }else{
                if selected_model_idx.is_some(){game.audio_events.push(Cue::WeaponDraw);}
                selected_model_idx=Some(target);
                selected_visual=WeaponVisual::load(&mut rl,&thread,target);
            }
        }
        // Gameplay animation triggers are independent from sound/mute settings.
        let frame_cues=game.audio_events.take();
        if let Some(ref mut visual)=selected_visual {
            for cue in &frame_cues {
                match *cue {
                    Cue::WeaponFire(index) if index as usize==game.arsenal.selected => {
                        visual.trigger(WeaponMotion::Fire);
                    },
                    _=>{},
                }
            }
            if manual_reload_requested{visual.trigger(WeaponMotion::Reload);}
            visual.update(&mut rl,&thread,dt);
        }
        if let Some(target)=pending_visual_idx {
            if selected_visual.as_ref().is_none_or(|v|v.status().0=="idle"){
                game.audio_events.push(Cue::WeaponDraw);
                selected_model_idx=Some(target);
                selected_visual=WeaponVisual::load(&mut rl,&thread,target);
                pending_visual_idx=None;
            }
        }
        if os.system.speakers {
            for cue in frame_cues {
                // The system-sounds switch silences interface feedback but not gameplay sounds.
                let is_ui=matches!(cue,Cue::ComputerOpen|Cue::Keyboard|Cue::MailHandled|Cue::ShopPurchase|Cue::ShopDenied);
                if is_ui && !os.system.system_sounds {continue;}
                match cue {
                    Cue::ComputerOpen=>{if let Some(ref s)=open_sound{s.play();}},
                    Cue::Keyboard=>{if let Some(ref s)=keyboard_sound{s.play();}},
                    Cue::MailHandled|Cue::Delivery|Cue::WaveEnd=>{if let Some(ref s)=notification_sound{s.play();}},
                    Cue::ShopPurchase=>{if let Some(ref s)=purchase_sound{s.play();}},
                    Cue::ShopDenied=>{if let Some(ref s)=denied_sound{s.play();}},
                    Cue::Footstep=>{if let Some(ref s)=walk_sound{s.play();}},
                    Cue::DoorPound|Cue::WaveStart=>{if let Some(ref s)=knock_sound{s.play();}},
                    Cue::WeaponFire(index)=>{
                        // Distinctive source recordings only where available. Other props remain silent.
                        match index {
                            4=>{if let Some(ref s)=m4_sound{s.play();}},
                            3|5|13=>{if let Some(ref s)=machine_sound{s.play();}},
                            2|10|11|14|15=>{if let Some(ref s)=gunshot_sound{s.play();}},
                            7|8|9=>{if let Some(ref s)=explosion_sound{s.play();}},
                            // Megaphone, cardboard, knife and flamethrower need suitable custom clips.
                            _=>{},
                        }
                    },
                    Cue::Pickup|Cue::SugarPickup=>{if let Some(ref s)=pickup_sound{s.play();}},
                    Cue::PlayerCaught=>{if let Some(ref s)=hurt_sound{s.play();}},
                    Cue::Detonation=>{if let Some(ref s)=explosion_sound{s.play();}},
                    Cue::WeaponDraw=>{if let Some(ref s)=weapon_draw_sound{s.play();}},
                    Cue::WeaponEmpty|Cue::Airhorn|Cue::Confetti|Cue::Drone|Cue::SugarWarning=>{},
                }
            }
        }
        let should_show_cursor=game.screen==Screen::Computer;
        if should_show_cursor!=cursor_on{
            if should_show_cursor{rl.enable_cursor();}else{rl.disable_cursor();}
            cursor_on=should_show_cursor;
        }

        let pos=Vector3::new(game.x,EYE_HEIGHT,game.z);
        let direction=Vector3::new(game.yaw.sin()*game.pitch.cos(),game.pitch.sin(),game.yaw.cos()*game.pitch.cos());
        let camera=Camera3D::perspective(pos,
            Vector3::new(pos.x+direction.x,pos.y+direction.y,pos.z+direction.z),
            Vector3::new(0.0,1.0,0.0),75.0);
        let display_w=rl.get_screen_width() as f32;
        let display_h=rl.get_screen_height() as f32;
        let mut d=rl.begin_drawing(&thread);
        if game.screen==Screen::Computer{
            let mail=EMAILS[game.inbox_index.min(EMAILS.len()-1)];
            let context=OsContext{
                cash:game.money,exposure:game.exposure,day:game.day,
                scans:game.scans,pis:game.raspberry_pis,alarm:game.alarm,
                active_wave:game.active_wave,
                orders_pending:game.pending_deliveries() as u32,
                deliveries_ready:game.ready_deliveries() as u32,
                next_delivery_eta:game.next_delivery_eta(),
                nearest_kid:game.kids.iter().map(|k|(k.x-game.x).hypot(k.z-game.z))
                    .min_by(|a,b|a.total_cmp(b)).unwrap_or(-1.0),
                front_visitors:game.kids.iter().filter(|k|k.z>1.20).count(),
                mail:MailView{pending:game.inbox_index<EMAILS.len(),index:game.inbox_index,sender:mail.sender,
                    subject:mail.subject,body_a:mail.first_line,body_b:mail.second_line,clue:mail.clue},
                inspected:game.inspected,message:&game.message,kids:game.kids.len(),
            };
            {
                let mut ui=d.begin_texture_mode(&thread,&mut ui_target);
                ui.clear_background(Color::BLACK);
                os.draw(&mut ui,&assets,&context);
                // Screen color temperature changes, not the Figma source files.
                if os.system.night_comfort {
                    ui.draw_rectangle(0,0,carteros::W,carteros::H,Color::new(244,158,52,40));
                }
                if os.system.dark_appearance {
                    ui.draw_rectangle(0,0,carteros::W,carteros::H,Color::new(17,22,37,24));
                }
            }
            d.clear_background(Color::new(14,16,28,255));
            let zoom=if os.system.interface_scale==125 {1.25}else{1.0};
            let fit=(display_w/carteros::W as f32).min(display_h/carteros::H as f32)*zoom;
            let dw=carteros::W as f32*fit;
            let dh=carteros::H as f32*fit;
            d.draw_texture_pro(ui_target.texture(),
                Rectangle::new(0.0,0.0,carteros::W as f32,-(carteros::H as f32)),
                Rectangle::new((display_w-dw)/2.0+ui_pan_x,(display_h-dh)/2.0+ui_pan_y,dw,dh),
                Vector2::zero(),0.0,Color::WHITE);
        }else{
            d.clear_background(Color::new(23,27,35,255));
            {
                let mut world=d.begin_mode3D(camera);
                if let Some(ref model)=office{world.draw_model(model,Vector3::zero(),1.0,Color::WHITE);}
                else{
                    world.draw_cube(Vector3::new(0.0,-0.06,0.0),2.8,0.1,5.5,Color::DARKGRAY);
                    world.draw_cube(Vector3::new(0.85,0.35,-0.7),0.75,0.7,1.7,Color::BROWN);
                }
                if game.ready_deliveries()>0 {
                    if let Some(ref crate_m)=crate_model{
                        world.draw_model(crate_m,Vector3::new(0.0,0.15,2.15),0.5,Color::WHITE);
                    } else {world.draw_cube(Vector3::new(0.0,0.18,2.15),0.42,0.36,0.35,Color::GOLD);}
                    world.draw_cube_wires(Vector3::new(0.0,0.18,2.15),0.42,0.36,0.35,Color::BROWN);
                }
                if let Some(pi)=game.decoy{world.draw_cube(Vector3::new(pi.x,0.12,pi.z),0.25,0.12,0.19,Color::GREEN);}
                if game.sugar_visible {
                    world.draw_cube(Vector3::new(0.10,0.13,-1.65),0.23,0.20,0.23,Color::RAYWHITE);
                    world.draw_cube_wires(Vector3::new(0.10,0.13,-1.65),0.23,0.20,0.23,Color::GOLD);
                }
                if show_weapon_preview {
                    if let Some(ref visual)=selected_visual {
                        // User GLBs vary widely in native scale; these remain provisional
                        // until the fully asset-loaded visual QA captures each weapon.
                        const SCALES:[f32;arsenal::COUNT]=[0.006,0.008,0.003,0.003,0.003,0.005,0.006,0.006,0.008,0.6,
                            0.005,0.005,0.005,0.005,0.005,0.005];
                        // Camera-relative position and yaw: no longer a fixed weapon left on
                        // the desk. The original authored animation transforms remain intact.
                        let right=Vector3::new(game.yaw.cos(),0.0,-game.yaw.sin());
                        let view_pos=Vector3::new(
                            pos.x+direction.x*0.68+right.x*0.18,
                            pos.y+direction.y*0.68-0.26,
                            pos.z+direction.z*0.68+right.z*0.18);
                        let scale=SCALES[game.arsenal.selected];
                        world.draw_model_ex(&visual.model,view_pos,
                            Vector3::new(0.0,1.0,0.0),game.yaw.to_degrees(),
                            Vector3::new(scale,scale,scale),Color::WHITE);
                    }
                }
                for kid in &game.kids{
                    if let Some(ref fan)=fan_model {
                        world.draw_model(fan,Vector3::new(kid.x,0.0,kid.z),0.83,Color::WHITE);
                        continue;
                    }
                    let shirt=match kid.variant{0=>Color::BLUE,1=>Color::RED,_=>Color::PURPLE};
                    world.draw_cube(Vector3::new(kid.x,0.70,kid.z),0.33,0.73,0.25,shirt);
                    world.draw_cube(Vector3::new(kid.x,1.19,kid.z),0.27,0.27,0.26,Color::BEIGE);
                    world.draw_cube(Vector3::new(kid.x-0.10,0.19,kid.z),0.11,0.37,0.18,Color::DARKBLUE);
                    world.draw_cube(Vector3::new(kid.x+0.10,0.19,kid.z),0.11,0.37,0.18,Color::DARKBLUE);
                }
            }
            draw_overlay(&mut d,&game);
        }
        if console.open {
            let dw=d.get_screen_width();let dh=d.get_screen_height();
            let height=260;d.draw_rectangle(12,12,dw-24,height,Color::new(17,21,34,246));
            d.draw_rectangle_lines(12,12,dw-24,height,Color::new(65,154,214,255));
            d.draw_text("CarterOS / DEBUG INSPECTOR   F12 to close",25,22,19,Color::SKYBLUE);
            for (i,line) in console.history.iter().rev().take(7).enumerate(){
                let clipped:String=line.chars().take(113).collect();
                d.draw_text(&clipped,25,60+i as i32*24,15,Color::RAYWHITE);
            }
            d.draw_text(&format!("> {}_",console.buffer),25,dh.min(245),18,Color::GREEN);
        }
    }
    if let Some(expected)=smoke_frames {
        if rendered_frames!=expected {
            eprintln!("RUN_CARTER_SMOKE_FAIL frames={rendered_frames} expected={expected}");
            std::process::exit(1);
        }
        println!("RUN_CARTER_SMOKE_OK frames={rendered_frames} screen={}",if smoke_desktop{"desktop"}else{"office"});
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;
    #[test]
    fn sponsorship_adds_money() {
        let mut g=Game::new();g.inbox_index=1;
        g.open_email(true);
        assert_eq!(g.money,380);
        assert_eq!(g.inbox_index,2);
    }
    #[test]
    fn deleting_malicious_email_avoids_location_leak() {
        let mut g=Game::new();g.inbox_index=2;
        g.open_email(false);
        assert_eq!(g.exposure,12.0);
        assert!(g.alarm<0.0);
    }
    #[test]
    fn location_leak_starts_countdown() {
        let mut g=Game::new();g.exposure=30.0;g.inbox_index=2;
        g.open_email(true);
        assert_eq!(g.exposure,100.0);
        assert_eq!(g.alarm,7.0);
    }
    #[test]
    fn phone_protection_cannot_be_spammed() {
        let mut g=Game::new();g.exposure=80.0;
        g.block_unknown_callers();assert_eq!(g.exposure,73.0);
        g.block_unknown_callers();assert_eq!(g.exposure,73.0);
        g.tick(45.0);g.block_unknown_callers();assert_eq!(g.exposure,66.0);
    }
    #[test]
    fn omni_protection_is_once_per_day() {
        let mut g=Game::new();g.exposure=80.0;
        g.optimize_with_omni();assert_eq!(g.exposure,60.0);
        g.optimize_with_omni();assert_eq!(g.exposure,60.0);
        g.day+=1;g.optimize_with_omni();assert_eq!(g.exposure,40.0);
    }
    #[test]
    fn shop_and_decoy_consume_in_game_resources() {
        let mut g=Game::new();g.buy(1);assert_eq!(g.money,95);
        assert_eq!(g.pending_deliveries(),1);
        assert_eq!(g.raspberry_pis,1);
        g.drop_decoy();assert_eq!(g.raspberry_pis,0);
        assert!(g.decoy.is_some());
    }
    #[test]
    fn physical_orders_arrive_and_require_pickup() {
        let mut g=Game::new();g.buy(1);
        assert_eq!(g.money,95);assert_eq!(g.raspberry_pis,1);
        assert_eq!(g.pending_deliveries(),1);
        g.tick(21.0);assert_eq!(g.ready_deliveries(),1);
        g.collect_deliveries();assert_eq!(g.raspberry_pis,1);
        g.x=0.0;g.z=2.0;g.collect_deliveries();
        assert_eq!(g.raspberry_pis,2);assert_eq!(g.ready_deliveries(),0);
    }
    #[test]
    fn cleared_mailbox_ends_shift_without_replaying_sponsorship() {
        let mut g=Game::new();g.inbox_index=EMAILS.len();
        let money=g.money;g.open_email(true);
        assert_eq!(g.money,money+45);assert_eq!(g.inbox_index,0);
        assert_eq!(g.day,2);
    }
    #[test]
    fn cleared_inbox_allows_safe_end_of_shift() {
        let mut g=Game::new();g.inbox_index=EMAILS.len();
        g.open_email(true);
        assert_eq!(g.day,2);assert_eq!(g.inbox_index,0);
        assert_eq!(g.money,225);assert!(g.finished_day);
    }
    #[test]
    fn cheat_money_and_first_use_are_bounded(){
        let mut g=Game::new();g.cheat_state=cheats::CheatState::default();
        assert!(g.use_cheat(CheatKind::Money));
        assert_eq!(g.money,100180);assert!(!g.use_cheat(CheatKind::Money));
        assert_eq!(g.cheat_state.uses,2);assert!(g.cheat_state.pending_steam);
    }
    #[test]
    fn godmode_prevents_contact_and_noclip_keeps_save_in_bounds(){
        let mut g=Game::new();g.cheat_state=cheats::CheatState::default();
        g.use_cheat(CheatKind::GodMode);g.use_cheat(CheatKind::Noclip);
        move_player(&mut g,12.0,0.0);
        assert!(g.snapshot().validate());
        g.active_wave=35.0;g.kids.push(Kid{x:g.x,z:g.z,speed:0.0,variant:0});
        g.tick(0.016);assert_ne!(g.screen,Screen::Caught);
    }
    #[test]
    fn reset_aggro_cancels_existing_wave(){
        let mut g=Game::new();g.cheat_state=cheats::CheatState::default();
        g.alarm=3.0;g.exposure=100.0;g.use_cheat(CheatKind::ClearAggro);
        assert!(g.alarm<0.0);assert_eq!(g.exposure,0.0);
    }
    #[test]
    fn fictional_sugar_addiction_countdown_and_last_second_save(){
        let mut g=Game::new();g.achievements=Tracker::default();
        g.x=0.10;g.z=-1.65;
        for _ in 0..3{g.sugar_visible=true;g.take_sugar();}
        assert!(g.achievements.has("ADDICT"));
        assert!(g.sugar_timer>0.0);
        g.tick(72.7);
        assert!(g.sugar_timer<3.0 && g.sugar_timer>0.0);
        g.sugar_visible=true;g.take_sugar();
        assert!(g.achievements.has("EDGE_OF_DETONATION"));
        assert!(g.sugar_timer>70.0);
    }
    #[test]
    fn cartoon_detonation_and_clean_run_reset(){
        let mut g=Game::new();g.achievements=Tracker::default();
        g.sugar_timer=0.25;g.tick(0.5);
        assert_eq!(g.screen,Screen::Caught);
        assert!(g.achievements.run.detonated);
        g.achievements.restart_clean_run();
        assert!(!g.achievements.run.detonated);
    }
    #[test]
    fn six_new_shop_items_keep_old_purchase_ids(){
        let mut g=Game::new();g.achievements=Tracker::default();g.money=15_000;
        for index in 10..arsenal::COUNT {g.buy(arsenal::FIRST_SHOP_ID+index as i32);}
        assert!(g.arsenal.owned[15]);
        assert_eq!(arsenal::CATALOG[0].name,"Cardboard Blaster");
        assert_eq!(arsenal::CATALOG[9].name,"Dynamite Pinata (prop)");
        assert!(g.arsenal.equip(15));
        assert_eq!(g.arsenal.selected,15);
    }
    #[test]
    fn all_arsenal_gadgets_are_purchasable(){
        let mut g=Game::new();g.achievements=Tracker::default();g.money=20_000;
        for item in [1,5,6,7]{g.buy(item);}
        assert_eq!((g.airhorns,g.confetti_cannons,g.drones),(1,1,1));
        // Utility gadgets alone are NOT the full ten-item armory.
        assert!(!g.achievements.has("FULL_ARSENAL_ZERO_WASTE"));
        for index in 1..arsenal::COUNT {g.buy(arsenal::FIRST_SHOP_ID+index as i32);}
        assert!(g.arsenal.all_owned());
        assert!(g.money>0);
        assert!(g.achievements.has("FULL_ARSENAL_ZERO_WASTE"));
        g.use_airhorn();g.use_confetti();g.deploy_drone();
        assert_eq!((g.airhorns,g.confetti_cannons,g.drones),(0,0,0));
    }
    #[test]
    fn emergency_bailout_marks_run_and_must_be_repaid(){
        let mut g=Game::new();g.achievements=Tracker::default();g.money=1;
        g.emergency_bailout();
        assert_eq!(g.money,401);
        assert_eq!(g.bailout_debt,450);
        assert!(g.achievements.run.bailed_out);
        assert_eq!(g.repay_debt(80),0);
        assert_eq!(g.bailout_debt,370);
    }
    #[test]
    fn sound_events_follow_real_actions_not_fake_reloads() {
        let mut g=Game::new();
        assert!(g.audio_events.take().is_empty());
        g.fire_equipped();
        assert!(g.audio_events.take().contains(&Cue::WeaponFire(0)));
        g.fire_equipped();
        assert!(g.audio_events.take().contains(&Cue::WeaponEmpty));
        g.money=1;
        g.buy(5);
        assert!(g.audio_events.take().contains(&Cue::ShopDenied));
    }
    #[test]
    fn delivery_notice_only_fires_once_and_message_expires() {
        let mut g=Game::new();
        g.buy(1);
        g.audio_events.take(); // discard purchase SFX
        g.orders[0].eta=0.04;
        g.tick(0.05); // model a normal frame instead of skipping 21 seconds in one frame
        assert!(g.ready_deliveries()>0);
        assert!(!g.delivery_notice);
        assert_eq!(g.audio_events.take(),vec![Cue::Delivery]);
        assert!(g.message.contains("package delivered"));
        let first=g.message_seconds;
        g.tick(0.25);
        assert!(g.message_seconds<first,"delivery message must not reset every frame");
        assert!(g.audio_events.take().is_empty());
    }
    #[test]
    fn snapshot_captures_campaign_orders() {
        let mut g=Game::new();g.buy(3);
        let snap=g.snapshot();assert!(snap.validate());
        assert_eq!(snap.orders.len(),1);assert_eq!(snap.orders[0].item,3);
    }

}
