//! RUN CARTER v0.7: exact Figma CarterOS exported assets integrated in 3D office.
//! E at the desk opens CarterOS; source layers are untouched Figma SVG exports.
//! Office geometry/collision remains a prototype; gameplay and desktop run in native Rust.

mod carteros;
mod system_state;
mod storage;
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
mod inbox;
mod business;
mod local_ai;
mod court_voice;
mod presentation;
mod creator;
mod district;
mod new_office;
mod finanzamt;
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
use presentation::Menu;

const EYE_HEIGHT: f32 = 1.58;
const WALK_SPEED: f32 = 1.9;
const SPRINT_SPEED: f32 = 3.2;
const INVENTORY_CAP: u32 = 100_000;
const STARTING_CASH: i32 = 2_400;
const SHIFT_PAY: i32 = 900;
const PRIVACY_BONUS: i32 = 450;
const PC_COST: i32 = 5_000;
const CHUTE_X: f32 = -0.33;
const CHUTE_Z: f32 = -2.39;
const SUGAR_DETONATION_SECONDS: f32 = 75.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Title,
    Hearing,
    Boss,
    Office,
    Computer,
    Caught,
}

#[derive(Clone, Copy)]
struct Kid {
    x: f32,
    z: f32,
    speed: f32,
    variant: usize,
    heading: f32,
    shield: bool,
}

#[derive(Clone, Copy)]
struct Decoy {
    x: f32,
    z: f32,
    remaining: f32,
}

#[derive(Clone, Copy)]
struct PartyProp {
    origin: Vector3,
    destination: Vector3,
    remaining: f32,
    duration: f32,
    radius: f32,
    capacity: usize,
}

impl PartyProp {
    fn position(self) -> Vector3 {
        let t = ((self.duration - self.remaining) / self.duration).clamp(0.0, 1.0);
        Vector3::new(
            self.origin.x + (self.destination.x - self.origin.x) * t,
            self.origin.y + (self.destination.y - self.origin.y) * t + 0.23 * (PI * t).sin(),
            self.origin.z + (self.destination.z - self.origin.z) * t,
        )
    }
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
    chase_flow: nav::Pursuit,
    decoy_flow: nav::Pursuit,
    decoy: Option<Decoy>,
    orders: Vec<Delivery>,
    delivery_notice: bool,
    repelled_this_wave:u32,
    finished_day: bool,
    outside: bool,
    pickup_claimed: bool,
    message: String,
    message_seconds: f32,
    achievements: Tracker,
    arsenal: arsenal::Arsenal,
    airhorns:u32, confetti_cannons:u32, drones:u32,
    sugar_visible:bool, sugar_hits:u32, sugar_timer:f32, sugar_boost:f32,
    rng_state: u64,
    inbox_seed: u64,
    inbox_text: Vec<String>,
    next_mail: Option<(u32, Vec<String>)>,
    business: business::Business,
    side: creator::Campaign,
    arena: Option<finanzamt::Arena>,
    hearing: Option<business::Hearing>,
    shift_closing: bool,
    defended_today: bool,
    reinforcement_at: f32,
    fans_queued: usize,
    pc_handovers: u32,
    studio_day: u32,
    hit_flash: f32,
    fire_flash: f32,
    thrown: Vec<PartyProp>,
    confetti_burst: Option<(Vector3, f32, f32)>,
    bailout_debt:i32,bailout_last_day:u32,
    audio_events:audio_events::Queue,
}

impl Game {
    fn new() -> Self {
        let achievements=Tracker::load();
        let mut cheat_state=cheats::CheatState::read();
        // A clean new run must not inherit cheat powers from an older campaign.
        if !achievements.run.cheated{cheat_state.god_mode=false;cheat_state.noclip=false;}
        let seed = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|time| time.as_nanos() as u64).unwrap_or(0xC4A7E2);
        Self {
            screen: Screen::Office,
            cheat_state,
            x: 0.00,
            z: -0.80,
            yaw: PI / 2.0,
            pitch: -0.18,
            money: STARTING_CASH,
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
            chase_flow: nav::Pursuit::default(),
            decoy_flow: nav::Pursuit::default(),
            decoy: None,
            orders: Vec::new(),
            delivery_notice: false,
            repelled_this_wave:0,
            finished_day: false,
            outside: false,
            pickup_claimed: false,
            message: String::from("Welcome to RUN CARTER. Answer emails, make money, survive."),
            message_seconds: 8.0,
            achievements,
            arsenal:arsenal::Arsenal::new(),
            airhorns:0,confetti_cannons:0,drones:0,
            sugar_visible:false,sugar_hits:0,sugar_timer:-1.0,sugar_boost:0.0,
            bailout_debt:0,bailout_last_day:0,
            audio_events:audio_events::Queue::default(),
            rng_state: seed, inbox_seed: seed, inbox_text: inbox::compose(1, seed), next_mail: None,
            business: business::Business::default(), side: creator::Campaign::default(), arena:None, hearing: None, shift_closing: false,
            defended_today: false, reinforcement_at: -1.0, fans_queued:0, pc_handovers:0, studio_day: 0, hit_flash: 0.0, fire_flash: 0.0, thrown: Vec::new(), confetti_burst: None,
        }
    }

    fn from_save() -> Option<Self> {
        let (s, recovered) = Snapshot::load(&Snapshot::path())?;
        let mut game = Self::new();
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
        game.inbox_seed = s.inbox_seed;
        game.inbox_text = s.inbox_text;
        game.inspected = s.inspected;
        game.business = s.business;
        game.side = s.side;
        if game.relocated() && !new_office::walkable(game.x,game.z) {
            (game.x,game.z)=new_office::ARRIVAL;
            game.yaw=PI;
            game.pitch=0.0;
        }
        game.alarm = s.alarm;
        game.shift_closing = s.shift_closing;
        game.reconcile_verified_cash();
        if let Some(run) = s.run_progress {
            let cheated = game.achievements.run.cheated || run.cheated;
            game.achievements.run = run;
            game.achievements.run.cheated = cheated;
        }
        if !game.achievements.run.cheated {
            game.cheat_state.god_mode = false;
            game.cheat_state.noclip = false;
        }
        game.defended_today = s.defended_today;
        game.studio_day = s.studio_day;
        game.arsenal.cooldown = s.weapon_cooldown;
        game.decoy = s.decoy.map(|(x, z, remaining)| Decoy { x, z, remaining });
        if game.business.verdict >= 2 { game.enter_arena(); }
        else if game.business.summoned() {
            game.hearing = Some(business::Hearing::new(game.inbox_seed, &game.business));
            game.screen = Screen::Hearing;
        }
        game.bailout_debt=s.bailout_debt;game.bailout_last_day=s.bailout_last_day;
        if recovered {
            match game.save_campaign() {
                Ok(()) => game.inform("Recovered the previous checkpoint."),
                Err(error) => game.inform(&format!("Recovered the previous checkpoint. {error}")),
            }
        } else {
            game.inform("Campaign restored. CarterOS uses a separate save for its settings.");
        }
        Some(game)
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
            sugar_boost:self.sugar_boost,
            inbox_seed: self.inbox_seed, inbox_text: self.inbox_text.clone(), inspected: self.inspected,
            business: self.business.clone(), side:self.side.clone(), alarm:self.alarm, shift_closing:self.shift_closing, defended_today: self.defended_today, studio_day: self.studio_day,
            weapon_cooldown: self.arsenal.cooldown, decoy: self.decoy.map(|d| (d.x, d.z, d.remaining)),
            run_progress: Some(self.achievements.run.clone()),
            weapon_owned:self.arsenal.ownership_mask(), weapon_charges:self.arsenal.charges,
            weapon_selected:self.arsenal.selected as u8,
            bailout_debt:self.bailout_debt,bailout_last_day:self.bailout_last_day}
    }
    fn save_campaign(&self)->Result<(),String>{
        if matches!(self.screen, Screen::Title | Screen::Caught) || self.outside || self.active_wave >= 0.0 {
            return Err("No safe checkpoint here. Your last save is still available.".to_owned());
        }
        self.snapshot().save(&Snapshot::path()).map_err(|e|format!("Save failed: {e}"))?;
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
            self.inform("A mysterious packet of COLOMBIAN POWDER appeared by the desk. R to collect.");
        }
    }
    fn take_sugar(&mut self) {
        if !self.sugar_visible || !self.at_powder() {
            self.inform("No Colombian Powder packet nearby.");return;
        }
        self.sugar_visible=false;
        self.audio_events.push(Cue::SugarPickup);
        let remaining=self.sugar_timer;
        self.achievements.sugar();
        self.sugar_hits=(self.sugar_hits + 1).min(INVENTORY_CAP);
        self.sugar_boost=25.0; // short-lived comic speed boost; NOT a real-world effect.
        if self.sugar_hits>=3 {
            if remaining>0.0 {self.achievements.emergency_save(remaining);}
            self.sugar_timer=SUGAR_DETONATION_SECONDS;
            self.audio_events.push(Cue::SugarWarning);
            self.inform(if remaining>0.0 && remaining<=3.0 {
                "EMERGENCY POWDER SAVE! You were milliseconds from cartoon detonation."
            }else{"Third powder pickup: absurd cartoon countdown started. Find more Colombian Powder!"});
        }else{self.inform("Fictional powder rush! Temporary speed boost.");}
    }
    fn fire_equipped(&mut self) {
        if self.arsenal.cooldown > 0.0 { return; }
        let Some(weapon)=self.arsenal.fire() else {
            self.audio_events.push(Cue::WeaponEmpty);
            self.inform("Out of cartridges. Buy a refill in AmazonCrime.");return;
        };
        self.audio_events.push(Cue::WeaponFire(self.arsenal.selected as u8));
        self.fire_flash = 0.09;
        let origin = Vector3::new(self.x, EYE_HEIGHT, self.z);
        let direction = look_direction(self.yaw, self.pitch);
        let selected = self.arsenal.selected;
        let cone = match selected { 1 => 0.64, 6 | 10 | 12 => 0.92, _ => 0.99 };
        if matches!(selected, 7..=9 | 14) {
            let destination = aim_impact(origin, direction, weapon.range, self);
            let duration = if matches!(selected, 8 | 9) { 0.48 } else { 0.27 };
            self.thrown.push(PartyProp { origin, destination, remaining: duration, duration,
                radius: if selected == 7 { 1.37 } else { 1.08 }, capacity: weapon.burst });
            return;
        }
        let candidates = self.kids.iter().enumerate().filter_map(|(i, fan)| {
            let target = Vector3::new(fan.x, 1.27, fan.z);
            let dx = target.x - origin.x;
            let dy = target.y - origin.y;
            let dz = target.z - origin.z;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt().max(0.001);
            let facing = (dx * direction.x + dy * direction.y + dz * direction.z) / distance;
            let tolerance = (0.22 / distance).min(0.45);
            (distance <= weapon.range && facing >= cone - 0.5 * tolerance * tolerance
                && clear_line(origin, target, self)).then_some((i, distance))
        }).collect::<Vec<_>>();
        self.distract(candidates, weapon.burst, false);
    }

    fn distract(&mut self, mut candidates: Vec<(usize, f32)>, capacity: usize, area: bool) {
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut hit = candidates.into_iter().take(capacity).map(|(i, _)| i).collect::<Vec<_>>();
        hit.sort_unstable_by(|a, b| b.cmp(a));
        let mut repelled = 0;
        let mut blocked = 0;
        for i in hit {
            if self.kids[i].shield && !area {
                self.kids[i].shield = false;
                blocked += 1;
            } else {
                self.kids.remove(i);
                repelled += 1;
            }
        }
        if repelled + blocked > 0 {
            self.hit_flash = 0.16;
            self.audio_events.push(Cue::Hit);
        }
        self.repelled_this_wave = self.repelled_this_wave.saturating_add(repelled);
        let feedback = if repelled > 0 { format!("{repelled} fan(s) distracted.") }
            else if blocked > 0 { "The fan's cardboard shield broke.".to_owned() }
            else { "No fans in range.".to_owned() };
        self.inform(&feedback);
    }
    fn tick_props(&mut self, dt: f32) {
        if let Some((_, remaining, _)) = &mut self.confetti_burst { *remaining -= dt; }
        if self.confetti_burst.is_some_and(|(_, remaining, _)| remaining <= 0.0) {
            self.confetti_burst = None;
        }
        let mut landed = Vec::new();
        self.thrown.retain_mut(|prop| {
            prop.remaining -= dt;
            if prop.remaining > 0.0 { true } else { landed.push(*prop); false }
        });
        for prop in landed {
            let point = prop.destination;
            self.confetti_burst = Some((point, 0.32, prop.radius));
            self.audio_events.push(Cue::Confetti);
            let candidates = self.kids.iter().enumerate().filter_map(|(i, fan)| {
                let separation = (fan.x - point.x).hypot(fan.z - point.z);
                (separation <= prop.radius && clear_line(point, Vector3::new(fan.x, 1.27, fan.z), self)).then_some((i, separation))
            }).collect();
            self.distract(candidates, prop.capacity, true);
        }
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
    fn income(&mut self, amount: i32) {
        let previous = self.money;
        let proceeds = self.repay_debt(amount.max(0));
        let pc_payment = self.side.pc_debt.min(proceeds);
        self.side.pc_debt -= pc_payment;
        let proceeds=proceeds-pc_payment;
        let overhead_payment=self.side.overhead_due.min(proceeds.max(0));
        self.side.overhead_due-=overhead_payment;
        self.money=self.money.saturating_add(proceeds-overhead_payment).min(5_000_000);
        self.business.verified_cash += self.money - previous;
        self.business.income(amount);
    }

    fn reconcile_verified_cash(&mut self) {
        self.business.verified_cash = self.business.verified_cash.min(self.money).max(0);
    }

    fn pay_tax(&mut self) {
        if !self.business.registered() {
            self.inform("No business filing is due.");
        } else if self.business.due == 0 {
            self.inform("Finanzamt: your balance is clear.");
        } else {
            let paid = self.business.pay(&mut self.money);
            self.reconcile_verified_cash();
            self.inform(&format!("Finanzamt: paid ${paid}. Balance ${}.", self.business.due));
        }
    }

    fn mail_count(&self) -> usize {
        inbox::count(self.day) + usize::from(self.side.morning!=0) + usize::from(self.side.remote_today)
            + usize::from(self.business.has_notice(self.day))
    }

    fn mail(&self) -> inbox::Mail {
        let mut index=self.inbox_index;
        if let Some(notice) = self.business.audit_notice(self.day) {
            if index == 0 { return notice; }
            index -= 1;
        }
        if self.side.morning!=0 {
            if index==0 {
                if let Some(mail)=inbox::follow_up(self.side.morning) {return mail;}
            }
            index=index.saturating_sub(1);
        }
        if self.side.remote_today {
            if index==0 {
                if let Some(mail)=inbox::follow_up(4) {return mail;}
            }
            index=index.saturating_sub(1);
        }
        inbox::generate(self.inbox_seed,self.day,index,&self.inbox_text)
    }

    fn enter_arena(&mut self) {
        let arena = finanzamt::Arena::new(self.arsenal.clone(),
            [self.raspberry_pis,self.airhorns,self.confetti_cannons,self.drones,u32::from(self.sugar_visible)]);
        self.arena = Some(match self.business.verdict { 3 => arena.close(true), 4 => arena.close(false), _ => arena });
        self.hearing = None;
        self.screen = Screen::Boss;
    }

    fn open_secret(&mut self) {
        if self.alarm >= 0.0 || self.active_wave >= 0.0 || !self.business.may_open_secret(self.day) {
            self.inform("Archived docket unavailable. Return when the file is complete and the office quiet.");
            return;
        }
        if self.business.has_notice(self.day) {
            if self.inbox_index == 0 { self.inspected = false; }
            self.inbox_index = self.inbox_index.saturating_sub(1);
        }
        self.business.secret_open = true;
        self.hearing = Some(business::Hearing::new(self.inbox_seed, &self.business));
        self.screen = Screen::Hearing;
        #[cfg(not(test))]
        if let Err(error)=self.save_campaign() {eprintln!("{error}");}
    }

    fn overhead(&self)->i32 {
        if self.day<12 {0} else {350+(self.day-12).min(120) as i32*15}
    }

    fn finish_shift(&mut self) { self.advance_shift(false); }

    fn advance_shift(&mut self, fled: bool) {
        let clean = !fled && self.exposure < 35.0;
        let reward = if fled { 0 } else { SHIFT_PAY + if clean { PRIVACY_BONUS } else { 0 } };
        self.income(reward);
        let overhead=self.overhead();
        let paid=self.money.min(overhead);
        self.money-=paid;
        self.reconcile_verified_cash();
        self.side.overhead_due=self.side.overhead_due.saturating_add(overhead-paid);
        self.business.close_books(self.day);
        self.achievements.record_shift(self.day);
        self.day = self.day.saturating_add(1);
        self.inbox_index = 0;
        self.inbox_text = match self.next_mail.take() {
            Some((day, text)) if day == self.day => text,
            _ => inbox::compose(self.day, self.inbox_seed),
        };
        self.inspected = false;
        self.shift_closing = false;
        self.defended_today = false;
        self.side.studio = 0;
        self.side.studio_project = 0;
        self.side.consult_done = false;
        self.side.remote_today=std::mem::take(&mut self.side.remote_next);
        self.side.incident = 0;
        let lead=std::mem::take(&mut self.side.lead);
        self.side.morning=if lead==3 && (self.exposure>=55.0 || fled) {5} else {lead};
        self.exposure = (self.exposure - 20.0).max(5.0);
        self.finished_day = true;
        self.maybe_spawn_sugar();
        let mut recap=if fled { format!("New Office. Shift pay forfeited. {} PC handover(s).",self.pc_handovers) }
            else { format!("Shift closed. Gross pay ${reward}{}.",if clean {" incl. privacy bonus"} else {""}) };
        if overhead>0 {recap.push_str(&format!(" Operating costs: ${overhead}."));}
        if self.business.may_open_secret(self.day) {
            recap.push_str(" Finanzamt summons in CarterMail.");
        } else if self.business.has_notice(self.day) {
            recap.push_str(" Finanzamt evidence update in CarterMail.");
        }
        self.inform(&recap);
    }

    fn run_debug(&mut self,command:DebugCommand,enabled:&mut bool)->String {
        match command {
            DebugCommand::Help=>"help | status | debug on | cash N | timer S | sugar spawn | ai bullshit | ai normal | kids N | tp desk | clear aggro".into(),
            DebugCommand::Status=>format!("Day {} | ${} | exposure {:.0}% | powder timer {:.1} | cheated {}",
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
            DebugCommand::Cash(n)=>{self.money=self.money.saturating_add(n).clamp(0,5_000_000);self.reconcile_verified_cash();format!("Granted ${n} (fictional).")},
            DebugCommand::Countdown(seconds)=>{self.sugar_timer=seconds;format!("Cartoon explosion timer set to {seconds:.1} seconds.")},
            DebugCommand::SugarSpawn=>{self.sugar_visible=true;"Colombian Powder appeared by desk. R to collect.".into()},
            DebugCommand::AiBullshit=>{self.exposure=(self.exposure+14.0).min(100.0);self.achievements.bullshit();self.check_new_exposure();"Forced 'le bullshit' AI roll (+14 exposure).".into()},
            DebugCommand::AiNormal=>{self.achievements.answer_normal();"Forced normal AI roll.".into()},
            DebugCommand::Kids(n)=>{for i in 0..n {
                self.kids.push(Kid{x:0.7,z:2.25,speed:0.6,variant:i as usize%3,heading:0.0,shield:false});
            }if self.active_wave<0.0{self.active_wave=35.0;}format!("Spawned {n} fictional fans.")},
            DebugCommand::Teleport=>{self.x=0.0;self.z=-0.80;"Teleported to desk.".into()},
            DebugCommand::ClearAggro=>{self.exposure=0.0;self.alarm=-1.0;self.active_wave=-1.0;self.kids.clear();self.fans_queued=0;self.shift_closing=false;"Aggro cleared.".into()},
            DebugCommand::Unknown=>"Unknown or malformed command. Type help.".into(),
        }
    }
    fn emergency_bailout(&mut self) {
        if self.money>=450 {self.inform("Bailouts are only offered below $450 cash.");return;}
        if self.bailout_last_day==self.day {
            self.inform("Only one embarrassing bailout is permitted per day.");return;
        }
        if self.alarm>=0.0||self.active_wave>=0.0 {
            self.inform("Bailouts unavailable during a fan invasion.");return;
        }
        self.bailout_last_day=self.day;
        self.bailout_debt=self.bailout_debt.saturating_add(2_100).min(100_000);
        self.money+=1_800;
        self.exposure=(self.exposure+18.0).min(100.0);
        self.achievements.bailout();
        self.inform("EMERGENCY BAILOUT: +$1800; repay $2100 from future earnings. Zero Waste forfeited.");
        self.check_new_exposure();
    }
    fn repay_debt(&mut self,payout:i32)->i32 {
        let paid=self.bailout_debt.min(payout);
        self.bailout_debt-=paid;
        payout-paid
    }
    fn relocated(&self) -> bool { self.side.relocations>0 }

    fn delivery_point(&self) -> (f32,f32) {
        if self.relocated() {new_office::DELIVERY} else {(0.0,2.15)}
    }

    fn at_delivery_door(&self)->bool {
        if self.relocated() {
            let (x,z)=self.delivery_point();
            (self.x-x).hypot(self.z-z)<0.86
        } else { self.x.abs()<0.5 && self.z>1.5 }
    }

    fn powder_point(&self) -> (f32,f32) {
        if self.relocated() {new_office::PICKUP} else {(0.10,-1.65)}
    }

    fn at_powder(&self) -> bool {
        let (x,z)=self.powder_point();
        !self.outside && (self.x-x).hypot(self.z-z)<=0.85
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
            let added_pis = pis.min(INVENTORY_CAP - self.raspberry_pis);
            let added_scans = scans.min(INVENTORY_CAP - self.scans);
            self.raspberry_pis += added_pis;
            self.scans += added_scans;
            self.audio_events.push(Cue::Pickup);
            self.inform(&format!("Delivery collected: +{added_pis} Raspberry Pi, +{added_scans} scans."));
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
                self.exposure=0.0;self.alarm=-1.0;self.active_wave=-1.0;self.kids.clear();self.fans_queued=0;self.shift_closing=false;
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
                self.kids.clear();self.fans_queued=0;self.shift_closing=false;self.inform("AGGRO RESET. Everybody remembered their homework.");
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
        if self.relocated() {
            let (x,z)=new_office::DESK;
            (self.x-x).hypot(self.z-z)<0.60
        } else { (self.x - 0.08).abs() < 0.75 && (self.z + 0.80).abs() < 1.05 }
    }

    fn open_email(&mut self, accept: bool) { self.open_email_with_multiplier(accept,1.0); }
    fn open_email_with_multiplier(&mut self, accept: bool, multiplier:f32) {
        if self.inbox_index >= self.mail_count(){
            if accept {self.end_shift();}
            else {self.inform("Inbox already empty. Press Enter to finish the shift.");}
            return;
        }
        let mail = self.mail();
        self.audio_events.push(Cue::MailHandled);
        if self.inbox_index == 0 && self.business.has_notice(self.day) {
            if accept && self.business.may_open_secret(self.day) {
                self.open_secret();
            } else {
                self.inbox_index += 1;
                self.inspected = false;
                self.inform("Notice filed. The office is still open.");
            }
            return;
        }
        if accept {
            self.income(mail.payout);
            self.exposure = (self.exposure + if mail.scam {mail.leak * multiplier} else {mail.leak}).min(100.0);
            if mail.scam {
                self.inform("BAD LINK! Your location data was exposed.");
            } else if mail.payout > 0 {
                self.inform("Sponsorship confirmed. Your channel is gaining attention.");
                if self.day>=4 && mail.leak>=23.0 {
                    self.side.incident=if self.orders.iter().any(|order|!order.ready) {1} else {2};
                    if self.exposure>=75.0 && self.alarm<0.0 && self.active_wave<0.0 {
                        self.alarm=self.warning_seconds();
                        self.inform(if self.side.incident==1 {"The sponsor's courier drew a crowd. Delivery and fans are on the way."}
                            else {"The live tour exposed the office. Expect quick reinforcements."});
                    }
                }
            } else {
                self.inform("Email answered. The sender won't stop talking about PCs.");
            }
        } else {
            self.inform("Email deleted. No money earned and no link opened.");
        }
        self.inbox_index += 1;
        self.inspected = false;
        if self.exposure >= 100.0 && self.alarm < 0.0 && self.active_wave < 0.0 {
            self.alarm = self.warning_seconds();
            self.inform("LOCATION LEAKED! The free-PC seekers are on their way!");
        }
    }

    fn end_shift(&mut self) {
        if self.alarm >= 0.0 || self.active_wave >= 0.0 {
            self.inform("The office must be secure before you can close.");
            return;
        }
        if self.screen == Screen::Hearing { return; }
        if self.day >= 3 && !self.defended_today && self.exposure >= self.closing_risk() {
            self.shift_closing = true;
            self.alarm = self.warning_seconds();
            self.screen = Screen::Office;
            self.inform("Closing time. Hold the office to finish the shift.");
        } else {
            self.finish_shift();
        }
    }

    fn closing_risk(&self) -> f32 {
        (70.0 - self.day as f32 * 0.35).max(35.0)
    }

    fn warning_seconds(&self) -> f32 {
        (11.0 - self.day as f32 * 0.07).max(4.0) + if self.door_upgrade { 7.0 } else { 0.0 }
    }

    fn check_new_exposure(&mut self) {
        if self.exposure >= 100.0 && self.alarm < 0.0 && self.active_wave < 0.0 {
            self.alarm=self.warning_seconds();
            self.inform("LOCATION LEAKED! Incoming fans!");
        }
    }
    fn inspect(&mut self) {
        if self.inbox_index>=self.mail_count(){self.inform("No new mail left to inspect today.");return;}
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
            let first = !self.arsenal.owned[index];
            let result=self.arsenal.buy(index,&mut self.money);
            self.reconcile_verified_cash();
            match result {
                Ok(message)=>{
                    if first && matches!(index, 5..=9 | 13 | 14) {
                        self.business.record_restricted_purchase(index);
                    }
                    self.audio_events.push(Cue::ShopPurchase);
                    self.achievements.purchase(item,self.money);
                    self.inform(&format!("{} {}",arsenal::CATALOG[index].name,message));
                },
                Err(e)=>{self.audio_events.push(Cue::ShopDenied);self.inform(e);},
            }
            return;
        }
        let Some(&(_, _, price)) = arsenal::GADGETS.iter().find(|row| row.0 == item) else { return; };
        let label = match item {
            1 => "Raspberry Pi ordered. Collect it at the front door.",
            2 => "AntiDoxx digital patch installed. Exposure reduced.",
            3 => "Scanner kit ordered. Collect it at the front door.",
            4 if !self.door_upgrade => "Reinforced door installed. +7s warning.",
            4 => {self.inform("The door is already reinforced.");return;}
            5 => "Airhorn added. Z: frighten close attackers.",
            6 => "Confetti cannon added. X: distract all active kids.",
            7 => "Decoy drone added. C: lure kids for 24 seconds.",
            8 => "Fictional Colombian Powder pack delivered to the desk. R to take.",
            25 if !self.business.storage_upgrade => "Delivery shelf installed. Hold up to 12 packages.",
            26 if !self.business.studio_upgrade => "Studio encoder installed. All uploads pay 25% more.",
            25 | 26 => { self.inform("That office upgrade is already installed."); return; },
            _ => return,
        };
        if self.money<price {self.audio_events.push(Cue::ShopDenied);self.inform("Insufficient money. Reply to legitimate sponsors.");return;}
        if item == 8 && self.sugar_visible {
            self.inform("A powder packet is already waiting by the desk.");
            return;
        }
        let stock = match item {
            5 => self.airhorns,
            6 => self.confetti_cannons,
            7 => self.drones,
            _ => 0,
        };
        if stock >= INVENTORY_CAP {
            self.inform("You have reached the inventory limit for this item.");
            return;
        }
        if (item==1||item==3) && self.orders.len() >= if self.business.storage_upgrade { 12 } else { 4 } {
            self.inform("Delivery queue full. Collect pending packages first.");return;
        }
        self.money-=price;
        self.reconcile_verified_cash();
        self.audio_events.push(Cue::ShopPurchase);
        match item {
            1=>self.orders.push(Delivery{item:1,eta:20.0,ready:false}),
            2=>self.exposure=(self.exposure-45.0).max(0.0),
            3=>self.orders.push(Delivery{item:3,eta:15.0,ready:false}),
            4=>self.door_upgrade=true,
            5=>self.airhorns=self.airhorns.saturating_add(1),
            6=>self.confetti_cannons=self.confetti_cannons.saturating_add(1),
            7=>self.drones=self.drones.saturating_add(1),
            25 => self.business.storage_upgrade = true,
            26 => self.business.studio_upgrade = true,
            8=>{
                self.sugar_visible = true;
                self.business.sugar_orders = self.business.sugar_orders.saturating_add(1);
            },
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
            let point = aim_impact(Vector3::new(self.x, EYE_HEIGHT, self.z), look_direction(self.yaw, self.pitch), if self.outside {7.0} else {3.7}, self);
            let (x,z) = (1..=20).rev().find_map(|i| {
                let t = i as f32/20.0;
                let (x,z) = (self.x+(point.x-self.x)*t,self.z+(point.z-self.z)*t);
                (self.walkability()(x,z)).then_some((x,z))
            }).unwrap_or((self.x,self.z));
            self.decoy = Some(Decoy { x, z, remaining: 6.0 });
            self.audio_events.push(Cue::Drone);
            self.inform("Pi thrown. Make your move while the fans investigate.");
        }
    }

    fn wave_total(&self) -> usize {
        let days = self.day.saturating_sub(3);
        let late = self.day.saturating_sub(20) / 2;
        (3 + days.min(240) + late.min(240)).min(240) as usize
    }

    fn spawn_fans(&mut self, count: usize) {
        for i in 0..count {
            let variant = if self.day < 6 { 0 } else { (self.fans_queued + i) % 3 };
            let base_speed = if self.outside {5.85} else {3.75};
            let speed = base_speed + self.day.min(90) as f32 * 0.018
                + if variant == 1 {0.42} else {0.0}
                + if self.business.overtime {0.25} else {0.0};
            let (x,z) = if self.outside {
                district::arrival(i)
            } else {
                (0.82 - (i % 4) as f32 * 0.28, 2.42 - (i / 4) as f32 * 0.14)
            };
            self.kids.push(Kid { x,z,speed,variant,heading:0.0,shield:variant==2 });
        }
    }

    fn start_wave(&mut self) {
        self.kids.clear();
        self.repelled_this_wave = 0;
        self.pc_handovers = 0;
        let first = self.wave_total().min(3);
        self.fans_queued = self.wave_total()-first;
        self.spawn_fans(first);
        self.active_wave = 0.0;
        self.reinforcement_at = self.reinforcement_delay();
        if self.side.incident == 1 {
            if let Some(order) = self.orders.iter_mut().find(|order| !order.ready) {
                order.eta = order.eta.min(1.5);
            }
        }
        self.screen = Screen::Office;
        self.audio_events.push(Cue::WaveStart);
        self.alarm = -1.0;
        self.maybe_spawn_sugar();
        self.inform("Door breached! Throw a Pi, drive the crowd off, or reach the chute.");
    }

    fn reinforcement_delay(&self) -> f32 {
        let delay = match self.side.incident { 1 => 2.8, 2 => 0.85, _ => 1.6 };
        if self.business.overtime { delay * 0.75 } else { delay }
    }

    fn chute_point(&self) -> (f32,f32) {
        if self.relocated() {new_office::CHUTE} else {(CHUTE_X,CHUTE_Z)}
    }

    fn at_chute(&self) -> bool {
        let (x,z)=self.chute_point();
        !self.outside && (self.x-x).hypot(self.z-z)<0.68
    }

    fn walkability(&self) -> fn(f32,f32)->bool {
        if self.outside {district::walkable}
        else if self.relocated() {new_office::walkable}
        else {free_position}
    }

    fn escape_chute(&mut self) {
        if self.active_wave < 0.0 || !self.at_chute() { return; }
        self.outside = true;
        self.pickup_claimed = false;
        self.fans_queued += self.kids.len();
        self.kids.clear();
        self.decoy = None;
        self.thrown.clear();
        self.confetti_burst = None;
        (self.x,self.z) = district::START;
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.reinforcement_at = self.active_wave + 1.65;
        self.inform("Reach the New Office entrance. The green beacon marks the building.");
    }

    fn claim_pickup(&mut self) {
        if !self.outside || self.pickup_claimed {return;}
        match district::Pickup::nearby(self.x,self.z) {
            Some(district::Pickup::Spare) => {
                if self.side.spare_pc {
                    self.inform("Already carrying a spare PC. Your pickup is still available at the Pi depot.");
                    return;
                }
                self.side.spare_pc = true;
                self.inform("Spare PC collected. Your next fan gets this build on the house.");
            }
            Some(district::Pickup::Decoys) => {
                self.raspberry_pis = (self.raspberry_pis + 2).min(INVENTORY_CAP);
                self.inform("Two Raspberry Pis collected. Press Q to throw one.");
            }
            None => return,
        }
        self.pickup_claimed = true;
        self.audio_events.push(Cue::Pickup);
    }

    fn enter_new_office(&mut self) {
        if !self.outside || !district::at_entrance(self.x,self.z) { return; }
        self.outside = false;
        self.kids.clear();
        self.fans_queued = 0;
        self.active_wave = -1.0;
        self.alarm = -1.0;
        self.decoy = None;
        self.thrown.clear();
        self.confetti_burst = None;
        (self.x,self.z)=new_office::ARRIVAL;
        self.yaw=PI;
        self.pitch = 0.0;
        self.exposure = 25.0;
        self.side.relocations = self.side.relocations.saturating_add(1);
        self.advance_shift(true);
    }

    fn hand_over_pcs(&mut self, count: u32) {
        self.pc_handovers += count;
        let prepared = self.side.spare_pc && count>0;
        self.side.spare_pc &= !prepared;
        let bill = PC_COST * (count-u32::from(prepared)) as i32;
        let paid = self.money.min(bill);
        self.money -= paid;
        self.reconcile_verified_cash();
        self.side.pc_debt = self.side.pc_debt.saturating_add(bill-paid);
        self.audio_events.push(Cue::PlayerCaught);
        let spare = if prepared { " Spare PC used." } else { "" };
        self.inform(&format!("{count} PC handover(s): -${paid}. PC debt ${}.{spare}",self.side.pc_debt));
    }

    fn tick(&mut self, dt: f32) {
        self.fire_flash = (self.fire_flash - dt).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);
        self.arsenal.tick(dt);
        self.tick_props(dt);
        if self.sugar_boost>0.0 {self.sugar_boost=(self.sugar_boost-dt).max(0.0);}
        if self.sugar_timer>0.0 {
            self.sugar_timer-=dt;
            if self.sugar_timer<=0.0 {
                self.sugar_timer=-1.0;
                self.achievements.detonation();
                self.audio_events.push(Cue::Detonation);
                self.screen=Screen::Caught;
                self.inform("KABOOM! Cartoon powder overload. A spectacularly bad decision.");
                return;
            }
        }
        let mut arrived = false;
        for order in &mut self.orders {
            if !order.ready {
                order.eta=(order.eta-dt).max(0.0);
                if order.eta <= 0.0 { order.ready = true; arrived = true; }
            }
        }
        if arrived {
            self.delivery_notice = true;
            self.audio_events.push(Cue::Delivery);
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
            if !self.outside {self.audio_events.push(Cue::DoorPound);}
            self.active_wave += dt;
            let mut tags = 0;
            let limit = if self.outside { (6 + self.day.min(100) / 5) as usize }
                else { (3 + self.day.min(110) / 10) as usize };
            let pass=self.walkability();
            if self.fans_queued > 0 && self.active_wave >= self.reinforcement_at && self.kids.len() < limit {
                let batch = if self.outside {limit} else if self.side.incident == 1 {3} else {(2+self.day.min(100)/20) as usize};
                let next = (limit-self.kids.len()).min(batch).min(self.fans_queued);
                self.spawn_fans(next);
                self.fans_queued -= next;
                self.reinforcement_at = self.active_wave + self.reinforcement_delay();
                if !self.outside {
                    self.inform("More fans at the front door!");
                    self.audio_events.push(Cue::WaveStart);
                }
            }
            let area = if self.outside { nav::Area::District } else if self.relocated() {
                nav::Area::RepairShop
            } else { nav::Area::Office };
            let chase = self.chase_flow.flow(area, (self.x, self.z), pass);
            let decoy = self.decoy.map(|d| self.decoy_flow.flow(area, (d.x, d.z), pass));
            self.kids.retain_mut(|kid| {
                let flow = if kid.variant == 1 { chase } else { decoy.unwrap_or(chase) };
                let (tx,tz) = self.decoy.filter(|_|kid.variant!=1).map_or((self.x,self.z),|d|(d.x,d.z));
                let direction = if self.outside && (tx - kid.x).hypot(tz - kid.z) < 0.75
                    && nav::clear_segment((kid.x, kid.z), (tx, tz), pass) {
                    Some((tx-kid.x,tz-kid.z))
                } else {flow.direction(kid.x,kid.z)};
                if let Some((dx,dz))=direction {
                    let distance=dx.hypot(dz);
                    if distance>0.0001 {
                        let change=(kid.speed*dt).min(distance);
                        let nx=kid.x+dx/distance*change;
                        let nz=kid.z+dz/distance*change;
                        if pass(nx,nz){kid.x=nx;kid.z=nz;kid.heading=(-dx).atan2(-dz).to_degrees();}
                    }
                }
                if !self.cheat_state.god_mode && (kid.x-self.x).hypot(kid.z-self.z)<0.32 {
                    tags += 1;
                    false
                } else { true }
            });
            if tags > 0 {
                for _ in 0..tags { self.achievements.hit(); }
                self.hand_over_pcs(tags);
            }
            if !self.outside && self.kids.is_empty() && self.fans_queued == 0 {
                self.audio_events.push(Cue::WaveEnd);
                self.achievements.wave_survived(self.repelled_this_wave);
                self.repelled_this_wave=0;
                self.kids.clear();
                self.active_wave = -1.0;
                self.exposure = 25.0;
                self.defended_today = true;
                let reward = if self.pc_handovers == 0 { 1_400 } else { 0 };
                self.income(reward);
                self.scans = self.scans.saturating_add(1).min(100_000);
                self.achievements.endless_milestones(self.day);
                if self.shift_closing { self.finish_shift(); }
                else {
                    if self.pc_handovers > 0 {
                        self.inform(&format!("Office clear after {} PC handover(s). PC debt ${}.",self.pc_handovers,self.side.pc_debt));
                    } else { self.inform("Office clear. You can return to the inbox or close the shift."); }
                    #[cfg(not(test))]
                    if let Err(error)=self.save_campaign(){eprintln!("{error}");}
                }
            }
        }
    }
}

fn look_direction(yaw: f32, pitch: f32) -> Vector3 {
    Vector3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos())
}

fn shot_passable(x: f32, y: f32, z: f32, game: &Game) -> bool {
    if game.outside { return district::passable(x,y,z,0.0); }
    if game.relocated() {return new_office::passable(x,y,z,0.0);}
    if !(-1.31..=1.09).contains(&x) || !(-2.56..=2.57).contains(&z) || y < 0.0 || y > 2.7 {
        return false;
    }
    if x > 0.33 && (-2.19..=1.16).contains(&z) && y <= 0.95 { return false; }
    !(x < -0.69 && (-2.35..=2.53).contains(&z) && y < 2.1)
}

fn clear_line(from: Vector3, to: Vector3, game: &Game) -> bool {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    let steps = ((dx * dx + dy * dy + dz * dz).sqrt() / 0.08).ceil() as u32;
    (1..steps).all(|i| {
        let t = i as f32 / steps as f32;
        shot_passable(from.x + dx * t, from.y + dy * t, from.z + dz * t, game)
    })
}

fn aim_impact(origin: Vector3, direction: Vector3, range: f32, game: &Game) -> Vector3 {
    let mut point = origin;
    for i in 1..=((range / 0.08).ceil() as u32) {
        let distance = (i as f32 * 0.08).min(range);
        let next = Vector3::new(origin.x + direction.x * distance, origin.y + direction.y * distance, origin.z + direction.z * distance);
        if !shot_passable(next.x, next.y, next.z, game) { break; }
        point = next;
    }
    point
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
    if g.outside {
        let pass = if g.cheat_state.noclip {district::bounded} else {district::walkable};
        if pass(x,g.z) {g.x=x;}
        if pass(g.x,z) {g.z=z;}
        return;
    }
    let pass=g.walkability();
    if (g.cheat_state.noclip && (-1.31..=1.09).contains(&x)) || pass(x,g.z) { g.x = x; }
    if (g.cheat_state.noclip && (-2.56..=2.57).contains(&z)) || pass(g.x,z) { g.z = z; }
}


fn draw_world<D: RaylibDraw3D>(world: &mut D, game: &Game, office: Option<&Model>,
    fan_model: Option<&Model>, crate_model: Option<&Model>) {
    if game.outside { district::draw(world, game.pickup_claimed); }
    else {
    if game.relocated() {new_office::draw(world);}
    else if let Some(model)=office{world.draw_model(model,Vector3::zero(),1.0,Color::WHITE);}
    else{
        world.draw_cube(Vector3::new(0.0,-0.06,0.0),2.8,0.1,5.5,Color::DARKGRAY);
        world.draw_cube(Vector3::new(0.85,0.35,-0.7),0.75,0.7,1.7,Color::BROWN);
    }
    let (chute_x,_)=game.chute_point();
    world.draw_cube(Vector3::new(chute_x,0.57,-2.54),0.55,1.00,0.16,Color::new(37,49,57,255));
    world.draw_cube(Vector3::new(chute_x,0.78,-2.44),0.46,0.38,0.06,Color::BLACK);
    world.draw_cube(Vector3::new(chute_x,1.10,-2.46),0.45,0.07,0.05,
        if game.active_wave>=0.0 {Color::LIME} else {Color::ORANGE});
    if game.door_upgrade {
        world.draw_cube(Vector3::new(0.0, 1.05, 2.53), 1.02, 0.11, 0.08, Color::new(72, 210, 177, 255));
        world.draw_cube(Vector3::new(-0.49, 1.05, 2.53), 0.06, 1.06, 0.08, Color::DARKGRAY);
        world.draw_cube(Vector3::new(0.49, 1.05, 2.53), 0.06, 1.06, 0.08, Color::DARKGRAY);
    }
    if game.business.storage_upgrade {
        let position=if game.relocated() {Vector3::new(0.88,0.35,0.88)} else {Vector3::new(-0.55,0.18,2.12)};
        world.draw_cube(position,0.23,0.33,0.34,Color::new(91,121,126,255));
    }
    if game.business.studio_upgrade {
        let position=if game.relocated() {Vector3::new(-0.95,0.98,-0.31)} else {Vector3::new(0.51,0.95,-1.27)};
        world.draw_cube(position,0.11,0.16,0.23,Color::new(118,228,195,255));
    }
    if game.ready_deliveries()>0 {
        let (x,z)=game.delivery_point();
        if let Some(crate_m)=crate_model{
            world.draw_model(crate_m,Vector3::new(x,0.15,z),0.5,Color::WHITE);
        } else {world.draw_cube(Vector3::new(x,0.18,z),0.42,0.36,0.35,Color::GOLD);}
        world.draw_cube_wires(Vector3::new(x,0.18,z),0.42,0.36,0.35,Color::BROWN);
    }
    if game.sugar_visible {
        let (x,z)=game.powder_point();
        world.draw_cube(Vector3::new(x,0.13,z),0.23,0.20,0.23,Color::RAYWHITE);
        world.draw_cube_wires(Vector3::new(x,0.13,z),0.23,0.20,0.23,Color::GOLD);
    }
    }
    if let Some(pi)=game.decoy{world.draw_cube(Vector3::new(pi.x,0.12,pi.z),0.25,0.12,0.19,Color::GREEN);}
    for prop in &game.thrown {
        world.draw_sphere(prop.position(), 0.075, Color::GOLD);
    }
    if let Some((point, time, radius)) = game.confetti_burst {
        let progress = (1.0 - time / 0.32).clamp(0.0, 1.0);
        for i in 0..20 {
            let angle = i as f32 * 2.39;
            let radius = progress * radius * (0.5 + (i % 4) as f32 * 0.14);
            let pos = Vector3::new(point.x + angle.cos() * radius, point.y + (angle * 2.0).sin() * radius * 0.42, point.z + angle.sin() * radius);
            let tint = [Color::GOLD, Color::PINK, Color::SKYBLUE, Color::GREEN][i % 4];
            world.draw_cube(pos, 0.045, 0.045, 0.045, tint);
        }
    }
    for kid in &game.kids{
        if let Some(fan)=fan_model {
            let tint = match kid.variant {
                1 => Color::new(255, 188, 142, 255),
                2 if kid.shield => Color::new(150, 240, 255, 255),
                _ => Color::WHITE,
            };
            world.draw_model_ex(fan, Vector3::new(kid.x, 0.0, kid.z), Vector3::new(0.0, 1.0, 0.0),
                kid.heading, Vector3::new(0.83, 0.83, 0.83), tint);
            if kid.shield { world.draw_cube_wires(Vector3::new(kid.x, 0.83, kid.z), 0.54, 0.72, 0.54, Color::SKYBLUE); }
            continue;
        }
        let shirt=match kid.variant{0=>Color::BLUE,1=>Color::RED,_=>Color::PURPLE};
        world.draw_cube(Vector3::new(kid.x,0.70,kid.z),0.33,0.73,0.25,shirt);
        world.draw_cube(Vector3::new(kid.x,1.19,kid.z),0.27,0.27,0.26,Color::BEIGE);
        world.draw_cube(Vector3::new(kid.x-0.10,0.19,kid.z),0.11,0.37,0.18,Color::DARKBLUE);
        world.draw_cube(Vector3::new(kid.x+0.10,0.19,kid.z),0.11,0.37,0.18,Color::DARKBLUE);
    }
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
        let rows = [
            ("YOU WERE CAUGHT".to_owned(), h / 2 - 100, 42, red),
            ("The powder countdown reached zero.".to_owned(), h / 2 - 13, 21, white),
            (format!("Day {}  |  Best shift {}  |  {} waves held", g.day, g.achievements.best_shift, g.achievements.run.wave_wins), h / 2 + 36, 23, teal),
            (format!("Business revenue ${}  |  Tax paid ${}", g.business.gross, g.business.paid), h / 2 + 74, 20, white),
            ("Enter retry checkpoint   R restart   Esc title".to_owned(), h / 2 + 126, 19, muted),
        ];
        for (text, y, size, color) in rows {
            let x = (w - d.measure_text(&text, size)) / 2;
            d.draw_text(&text, x, y, size, color);
        }
        return;
    }
    d.draw_rectangle(0, 0, w, 67, navy);
    let location = if g.relocated() && !g.outside { "NEW OFFICE" } else if g.outside { "SERVICE DISTRICT" } else { "RUN CARTER" };
    d.draw_text(location, 20, 9, 24, white);
    let label_end = 20 + d.measure_text(location, 24);
    if g.business.overtime { d.draw_text("OVERTIME", label_end + 14, 14, 14, teal); }
    let debt = g.bailout_debt.saturating_add(g.side.pc_debt).saturating_add(g.side.overhead_due);
    d.draw_text(&format!("Day {}  |  Cash ${}  |  Debt ${debt}", g.day, g.money), 20, 37, 14, muted);
    d.draw_text(&format!("Pi x{}  |  Scans {}", g.raspberry_pis, g.scans), 20, 52, 13, muted);
    if g.sugar_timer>0.0 {
        d.draw_rectangle(w/2-190,156,380,46,Color::new(130,14,41,243));
        d.draw_text(&format!("CARTOON DETONATION: {:.1}s",g.sugar_timer),w/2-171,167,20,Color::WHITE);
    }else if g.sugar_visible && !g.outside {d.draw_text("COLOMBIAN POWDER: near desk (R)",w/2-195,165,20,teal);}
    if g.achievements.run.cheated {d.draw_text("ACHIEVEMENTS DISABLED THIS RUN",w/2-178,218,19,red);}
    if !g.outside && g.business.registered() {
        d.draw_text(&format!("TAX ${}  |  F7: file payment", g.business.due), 20, 76, 18, teal);
    }
    if !g.outside && g.business.has_notice(g.day) {
        d.draw_text("FINANZAMT  |  New case mail at the desk", 20, 102, 18, teal);
    }
    if g.cheat_state.god_mode {d.draw_text("GOD",w/2-30,11,21,teal);}
    if g.cheat_state.noclip {d.draw_text("NOCLIP",w/2+35,12,17,teal);}
    d.draw_text("LOCATION EXPOSURE",w-239,11,16,white);
    d.draw_rectangle(w-239,37,185,12,Color::DARKGRAY);
    d.draw_rectangle(w-239,37,(185.0*g.exposure/100.0) as i32,12,if g.exposure>70.0{red}else{teal});
    d.draw_text(&format!("{}%",g.exposure as i32),w-48,35,13,white);
    if g.outside {
        let distance=(g.x-district::ENTRANCE.0).hypot(g.z-district::ENTRANCE.1);
        let width = (w - 216).min(452);
        let x = ((w - width) / 2).min(w - 196 - width);
        d.draw_rectangle(x, 79, width, 69, navy);
        d.draw_text(&format!("NEW OFFICE  /  {:.0} m", distance), x + 18, 86, 25, teal);
        d.draw_text(&format!("{} nearby  /  {} following", g.kids.len(), g.fans_queued), x + 18, 121, 16, white);
        district::minimap(d,(g.x,g.z),g.kids.iter().map(|kid|(kid.x,kid.z)),g.yaw,g.pickup_claimed,w);
    } else if g.alarm >= 0.0 {
        d.draw_rectangle(w/2-226,79,452,69,Color::new(117,18,28,246));
        d.draw_text(match g.side.incident {1=>"COURIER AT THE DOOR",2=>"LIVE CLIP LEAKED",_=>"FANS ON THE WAY"},w/2-138,87,24,white);
        d.draw_text(&format!("KIDS ARRIVE IN {:.1}s",g.alarm),w/2-130,120,20,white);
    } else if g.active_wave >= 0.0 {
        d.draw_rectangle(w/2-226,79,452,69,Color::new(117,18,28,246));
        d.draw_text("RUN CARTER!",w/2-107,86,25,white);
        d.draw_text(&format!("{} HERE   /   {} INCOMING   /   {} ",g.kids.len(),g.fans_queued,if g.relocated() {"CHUTE AT BACK WALL"} else {"CHUTE BY WINDOW"}),w/2-207,121,16,white);
    }
    if g.message_seconds > 0.0 {
        let max_width = (w - if g.outside { 216 } else { 50 }).min(820);
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in g.message.split_whitespace() {
            let candidate = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
            if !line.is_empty() && d.measure_text(&candidate, 17) > max_width {
                lines.push(std::mem::replace(&mut line, word.to_owned()));
            } else {
                line = candidate;
            }
        }
        lines.push(line);
        let height = lines.len() as i32 * 21 + 12;
        let y = h - 128 - height;
        d.draw_rectangle(14, y, max_width + 22, height, Color::new(14,30,45,237));
        for (i, line) in lines.iter().enumerate() {
            d.draw_text(line, 25, y + 6 + i as i32 * 21, 17, white);
        }
    }
    d.draw_rectangle(0,h-47,w,47,navy);
    let prompt=if g.outside {
        if district::at_entrance(g.x,g.z) {"E: ENTER NEW OFFICE"} else if !g.pickup_claimed {
            match district::Pickup::nearby(g.x,g.z) {
                Some(district::Pickup::Spare) => "E: CLAIM SPARE PC",
                Some(district::Pickup::Decoys) => "E: CLAIM TWO PIs",
                None => "GREEN: OFFICE  /  GOLD: PICK ONE",
            }
        } else {"FOLLOW THE GREEN BEACON"}
    } else if g.active_wave >= 0.0 && g.at_chute() {"E: DROP THROUGH CHUTE"}
        else if g.alarm >= 0.0 || g.active_wave >= 0.0 {if g.relocated() {"CHUTE AT BACK WALL"} else {"CHUTE BY WINDOW"}}
        else if g.at_delivery_door() && g.ready_deliveries()>0 {"F: COLLECT DELIVERY"}
        else if g.at_computer(){"E: USE COMPUTER"}else{"APPROACH THE DESK"};
    d.draw_text(prompt, 20, h - 42, 17, teal);
    d.draw_text("WASD move   Shift sprint   Q Pi   Z horn   X confetti   C drone   Esc pause", 20, h - 21, 14, white);
    d.draw_rectangle(15,h-119,560,62,Color::new(16,22,37,218));
    let wpn=&arsenal::CATALOG[g.arsenal.selected];
    d.draw_text(&format!("[{} / {}] {}  |  {} charges",g.arsenal.selected+1,arsenal::COUNT,wpn.name,g.arsenal.charges[g.arsenal.selected]),25,h-111,17,white);
    d.draw_text("1-0 / wheel to equip  |  Click to use  |  R inspect or pick up powder  |  B hide",25,h-87,14,muted);
    if g.arsenal.cooldown > 0.0 {
        let progress = (1.0 - g.arsenal.cooldown / wpn.cooldown).clamp(0.0, 1.0);
        d.draw_rectangle(25, h - 65, 126, 3, muted);
        d.draw_rectangle(25, h - 65, (126.0 * progress) as i32, 3, teal);
    }
    let crosshair = if g.hit_flash > 0.0 { teal } else { Color::WHITE };
    let size = if g.fire_flash > 0.0 { 10 } else { 7 };
    d.draw_line(w/2-size,h/2,w/2+size,h/2,crosshair);
    d.draw_line(w/2,h/2-size,w/2,h/2+size,crosshair);
}

fn handle_os_action(a:OsAction,g:&mut Game,os:&CarterOs){
    match a {
        OsAction::Exit=>g.screen=Screen::Office,
        OsAction::EndShift=>g.end_shift(),
        OsAction::AcceptMail=>{
            if os.network_online(){g.open_email_with_multiplier(true,os.phishing_multiplier());}
            else{g.inform("CarterMail: offline. Enable Wi-Fi in Settings.");}
        },
        OsAction::DeleteMail=>g.open_email(false),
        OsAction::InspectMail=>g.inspect(),
        OsAction::BuyItem(i)=>{
            if os.network_online(){g.buy(i);}else{g.inform("AmazonCrime: cannot order while offline.");}
        },
        OsAction::Spend(n)=>{g.money=(g.money-n).max(0);g.reconcile_verified_cash();},
        OsAction::SecurePhone=>g.block_unknown_callers(),
        OsAction::OmniProtect=>g.optimize_with_omni(),
        OsAction::PayWage(n)=>{g.income(n);},
        OsAction::SecretAudit=>g.open_secret(),
        OsAction::VideoUpload(kind)=>{
            if !os.network_online() { g.inform("CarterStudio is offline."); }
            else if g.studio_day == g.day { g.inform("Today's upload slot is already used."); }
            else if let Some((succeeds, pay, exposure)) = creator::upload(g.inbox_seed,g.day,kind as usize) {
                g.studio_day = g.day;
                g.side.studio_project = kind + 1;
                g.side.studio = if succeeds {2} else {1};
                let pay = if g.business.studio_upgrade { pay + pay / 4 } else { pay };
                g.income(pay);
                g.exposure = (g.exposure + exposure).min(100.0);
                g.inform(&format!("Upload {}. Earned ${pay} before debt payments.",if succeeds {"took off"} else {"flopped"}));
                if kind==2 && succeeds && g.day>=3 && g.exposure>=78.0 && g.alarm<0.0 && g.active_wave<0.0 {
                    g.side.incident=2;
                    g.alarm=g.warning_seconds();
                    g.inform("Live setup clip leaked. Reinforcements will come fast.");
                }
                g.check_new_exposure();
            }
        },
        OsAction::StudioComment(choice)=>{
            if !os.network_online() || g.studio_day!=g.day || !matches!(g.side.studio,1|2) {return;}
            let scenario=creator::comment(g.inbox_seed,g.day,g.side.studio==2,g.side.studio_project);
            let Some(&(pay,exposure,lead))=scenario.choices.get(choice as usize) else {return;};
            g.side.studio=3;
            if lead != 0 { g.side.lead = lead; }
            g.income(pay);
            g.exposure=(g.exposure+exposure).clamp(0.0,100.0);
            if lead==3 {g.side.incident=2;}
            if lead == 3 {
                g.inform(&format!("Trending on FanCord. Earned ${pay} gross. Watch for a follow-up."));
            } else if lead != 0 {
                g.inform("Comment handled. Check your inbox next shift.");
            } else {
                g.inform(&format!("Comment handled. Gross bonus ${pay}."));
            }
            g.check_new_exposure();
        },
        OsAction::RemoteConsult=>{
            if !os.network_online() || g.side.consult_done {return;}
            g.side.consult_done=true;
            g.income(creator::CONSULT_PAY);
            g.inform(&format!("Remote build consult complete. ${} gross, no location exposure.", creator::CONSULT_PAY));
        },
        OsAction::BuildSpare=>{
            if g.side.spare_pc {g.inform("A spare PC is already ready.");return;}
            if g.money<creator::SPARE_COST {g.inform(&format!("A spare PC costs ${} in parts.", creator::SPARE_COST));return;}
            g.money-=creator::SPARE_COST;
            g.reconcile_verified_cash();
            g.side.spare_pc=true;
            g.inform("Spare PC built. The next fan tag uses it instead of billing you.");
        },
        OsAction::FanReply(choice)=>{
            if !os.network_online() {g.inform("FanCord is offline.");return;}
            let offer=g.side.fan_today(g.inbox_seed,g.day);
            if !matches!(offer,1|2){return;}
            g.side.fan_day=g.day;
            g.side.fan_offer=3;
            if offer==1 && choice!=1 {g.side.archive_tip=true;}
            match (choice,offer) {
                (1,_)=>g.inform("FanCord DM ignored."),
                (2,2)=>{
                    g.side.remote_next=true;
                    g.inform("Partner verified. Their $4,800 remote offer will arrive via CarterMail.");
                },
                (2,_)=>g.inform("The real partnership desk couldn't confirm that DM. No data shared."),
                (_,2)=>{
                    g.income(7_800);
                    g.exposure=(g.exposure+25.0).min(100.0);
                    g.inform("FanCord collaboration paid $7,800 gross. Exposure +25.");
                    g.check_new_exposure();
                },
                _=>match creator::fan_persona(g.inbox_seed,g.day) {
                    0=>{
                        g.exposure=100.0;
                        g.side.incident=2;
                        g.check_new_exposure();
                        g.inform("Address leaked. The DM mentioned finanzamt://archiv.");
                    },
                    1=>{
                        g.exposure=(g.exposure+68.0).min(100.0);
                        g.side.lead=3;
                        g.inform("They posted your unreleased footage. It is spreading fast.");
                        g.check_new_exposure();
                    },
                    _=>{
                        let fee=g.money.min(2_000);
                        g.money-=fee;
                        g.reconcile_verified_cash();
                        g.exposure=(g.exposure+46.0).min(100.0);
                        g.inform(&format!("Fake partnership fee: -${fee}. The DM mentioned finanzamt://archiv."));
                        g.check_new_exposure();
                    },
                },
            }
        },
        OsAction::FakeSpoof=>{
            g.exposure=(g.exposure-35.0).max(0.0);
            if g.alarm>=0.0 && !g.shift_closing {g.alarm=-1.0;}
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
    #[cfg(not(test))]
    let _=g.save_campaign();
}

// First-person size correction derived from real Xvfb captures of the 16
// original weapons. Values affect only rendering, never gameplay or GLB data.
const VM_PROFILE_MULTIPLIERS:[f32;arsenal::COUNT]=[
    0.83,1.00,1.65,1.60,2.20,1.35,1.30,1.60,
    0.85,1.35,1.35,1.75,1.10,1.60,1.12,1.00,
];
fn main() {
    if let Err(error) = storage::set_install_directory() {
        eprintln!("Could not open the game installation directory: {error}");
        std::process::exit(1);
    }
    storage::init();
    // CI-only bounded visual startup check: actual raylib window, no fake render path.
    // Normal gameplay stays unchanged unless this explicit argument is supplied.
    let smoke_frames=std::env::args().filter_map(|a|{
        a.strip_prefix("--smoke-frames=").and_then(|s|s.parse::<u32>().ok())
    }).find(|n|(1..=1200).contains(n));
    let smoke_desktop=smoke_frames.is_some() && std::env::args().any(|a|a=="--smoke-desktop");
    // QA-only in-memory equip override; not accessible during ordinary gameplay.
    let qa_weapon_slot=if smoke_frames.is_some(){
        std::env::args().filter_map(|a|a.strip_prefix("--qa-weapon-slot=")
            .and_then(|s|s.parse::<usize>().ok())).find(|n|*n<arsenal::COUNT)
    }else{None};
    // Gameplay is unaffected by these bounded smoke-only calibration parameters.
    let qa_vm_value=|name:&str,default:f32,low:f32,high:f32|{
        if smoke_frames.is_none(){return default;}
        std::env::args().find_map(|arg|arg.strip_prefix(name)
            .and_then(|s|s.parse::<f32>().ok()))
            .filter(|n|n.is_finite() && *n>=low && *n<=high)
            .unwrap_or(default)
    };
    let vm_distance=qa_vm_value("--qa-vm-distance=",0.59,0.30,1.50);
    let vm_right=qa_vm_value("--qa-vm-right=",0.25,-0.60,0.70);
    let vm_down=qa_vm_value("--qa-vm-down=",-0.16,-0.90,0.30);
    let vm_scale_multiplier=qa_vm_value("--qa-vm-scale=",1.0,0.1,8.0);
    let vm_yaw_offset=qa_vm_value("--qa-vm-yaw=",0.0,-180.0,180.0);
    let mut rendered_frames=0u32;
    let (mut rl, thread)=raylib::init()
        .size(960, 600)
        .resizable()
        .title("RUN CARTER | Endless Office")
        .build();
    rl.set_target_fps(60);
    rl.set_window_min_size(640, 400);
    // Esc closes the in-game desktop instead of terminating the program.
    rl.set_exit_key(None);
    if smoke_frames.is_some() { rl.disable_cursor(); }
    // Full-asset diagnostic; loads real meshes and samples every weapon action.
    // Exit code 2 indicates missing meshes or incompatible skeletal animations.
    if std::env::args().any(|a|a=="--audit-weapon-animations"){
        let mut failures=0usize;
        for slot in 0..arsenal::COUNT{
            let asset=arsenal::CATALOG[slot].asset;
            match WeaponVisual::load(&mut rl,&thread,slot){
                None=>{println!("ANIM_AUDIT slot={slot} file={asset} MISSING_MODEL");failures+=1;},
                Some(mut v)=>{
                    let expected=weapon_animation::profile(slot).fire.is_some();
                    let loaded=v.animation_count();
                    for action in [WeaponMotion::Draw,WeaponMotion::Fire,WeaponMotion::Reload,WeaponMotion::Holster]{
                        v.trigger(action);
                        for _ in 0..4{v.update(&mut rl,&thread,1.0/60.0);}
                    }
                    let status=if expected && loaded==0{failures+=1;"MISSING_ANIMATION"}
                        else if loaded>0{"SKELETAL_OK"}else{"STATIC_PROCEDURAL_OK"};
                    println!("ANIM_AUDIT slot={slot} file={asset} clips={loaded} status={status} scale={:.6}",v.auto_scale());
                }
            }
        }
        println!("ANIM_AUDIT_RESULT failures={failures} models={}",arsenal::COUNT);
        if failures>0{std::process::exit(2);}
        return;
    }
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
    let mut ui_target = match rl.load_render_texture(&thread, carteros::W as u32, carteros::H as u32) {
        Ok(target) => target,
        Err(error) => {
            eprintln!("CarterOS display could not be initialized: {error}");
            std::process::exit(1);
        }
    };
    ui_target.texture().set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut security_target = rl.load_render_texture(&thread, 640, 214).ok();
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
    let gadget_sounds = ["empty", "airhorn", "confetti", "drone", "sugar_warning", "hit"]
        .map(|name| audio.as_ref().and_then(|a| a.new_sound(&format!("assets/sfx/endless/{name}.wav")).ok()));
    let notification_sound=audio.as_ref().and_then(|a|a.new_sound("assets/sfx/imported/universfield-welcome-notification-131917.mp3").ok());
    let mut previous_alarm=false;
    let mut last_preset = None;
    let mut game = Game::new();
    let mut saved = Snapshot::load(&Snapshot::path()).is_some();
    if smoke_frames.is_none() { game.screen = Screen::Title; }
    if smoke_desktop { game.screen = Screen::Computer; }
    let mut menu = (smoke_frames.is_none()).then_some(Menu::Title);
    let mut menu_selection = 0usize;
    let mut settings_return = Menu::Title;
    let mut settings_error = None;
    let mut quit_return = None;
    let mut last_volume = None;
    let mut hearing_feedback = None::<(usize, std::sync::mpsc::Receiver<Option<String>>)>;
    let mut voice = court_voice::CourtVoice::new(audio.as_ref());
    let mut mail_job = None::<((u64,u32), std::sync::mpsc::Receiver<Option<Vec<String>>>)>;
    let mut mailed_target = None;
    if let Some(slot)=qa_weapon_slot{
        game.arsenal.owned[slot]=true;
        game.arsenal.charges[slot]=99;
        game.arsenal.selected=slot;
    }
    let steam=steam_bridge::Bridge::new();
    let mut steam_retry=5.0f32;
    let mut cursor_on = smoke_frames.is_none();
    let mut console=DebugConsole::default();
    let mut debug_enabled=false;
    let mut show_weapon_preview=true; // Animated first-person weapon model (B toggles).
    let mut animation_inspector=false; // F9: pose/clip diagnostics.
    let mut last_money=game.money;
    let mut last_wall=SystemTime::now();
    let mut last_mono=Instant::now();
    let mut integrity_elapsed=0.0f32;
    let (mut ui_pan_x,mut ui_pan_y)=(0.0f32,0.0f32);
    loop {
        if smoke_frames.is_some_and(|n| rendered_frames >= n) { break; }
        let close_requested = rl.window_should_close();
        if close_requested && menu != Some(Menu::QuitUnsaved) {
            if smoke_frames.is_some() || matches!(game.screen, Screen::Title | Screen::Caught | Screen::Hearing | Screen::Boss) {
                break;
            }
            match game.save_campaign() {
                Ok(()) => break,
                Err(error) => {
                    game.inform(&error);
                    quit_return = menu;
                    menu = Some(Menu::QuitUnsaved);
                    menu_selection = 0;
                }
            }
        }
        let volume = if os.system.speakers { os.system.master_volume } else { 0 };
        if last_volume != Some(volume) {
            if let Some(device) = &audio { device.set_master_volume(f32::from(volume) / 100.0); }
            last_volume = Some(volume);
        }
        steam.tick();
        rendered_frames=rendered_frames.saturating_add(1);
        if smoke_frames.is_none() && rl.is_window_focused() && rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_borderless_windowed();
        }
        if last_preset != Some(os.system.display_preset) && !rl.get_window_state().window_undecorated() {
            let monitor = raylib::core::window::get_current_monitor();
            let origin = raylib::core::window::get_monitor_position(monitor);
            let (monitor_w, monitor_h) = (raylib::core::window::get_monitor_width(monitor),
                raylib::core::window::get_monitor_height(monitor));
            let (width, height) = if os.system.display_preset == 0 { (1440, 900) } else { (1280, 720) };
            let fit = ((monitor_w - 32) as f32 / width as f32)
                .min((monitor_h - 80) as f32 / height as f32).clamp(0.45, 1.0);
            let (width, height) = ((width as f32 * fit) as i32, (height as f32 * fit) as i32);
            rl.set_window_size(width, height);
            rl.set_window_position(origin.x as i32 + (monitor_w - width) / 2,
                origin.y as i32 + (monitor_h - height) / 2);
            last_preset = Some(os.system.display_preset);
        }
        let viewport = presentation::fit_canvas(rl.get_screen_width(), rl.get_screen_height());
        let dt=rl.get_frame_time().min(0.05);
        let mut manual_reload_requested=false;
        if smoke_frames.is_none() && !matches!(game.screen, Screen::Caught | Screen::Hearing | Screen::Boss) {
            let target = if game.screen == Screen::Title && !saved { 1 } else { game.day.saturating_add(1) };
            let key=(game.inbox_seed,target);
            if let Some(((seed,day), receiver)) = &mail_job {
                match inbox::poll(receiver) {
                    Ok(lines) => {
                        if let Some(lines) = lines.filter(|_|*seed==game.inbox_seed) {
                            if game.screen == Screen::Title && !saved && *day == 1 {
                                game.inbox_text = lines;
                            } else if *day==game.day && game.inbox_index==0 && !game.inspected && game.screen!=Screen::Computer {
                                game.inbox_text=lines;
                            } else if *day > game.day { game.next_mail = Some((*day, lines)); }
                        }
                        mail_job = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => mail_job = None,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                }
            }
            if mail_job.is_none() && mailed_target != Some(key) {
                mail_job = inbox::request_next(target,game.inbox_seed).map(|rx| (key, rx));
                mailed_target = Some(key);
            }
        }
        let focused = rl.is_window_focused();
        let pressed = |key| focused && rl.is_key_pressed(key);
        let can_pause = matches!(game.screen, Screen::Office | Screen::Computer | Screen::Boss);
        let opening_pause = smoke_frames.is_none() && menu.is_none() && can_pause
            && (!focused || (game.screen == Screen::Office && !console.open && pressed(KeyboardKey::KEY_ESCAPE)));
        if opening_pause {
            menu = Some(Menu::Pause);
            menu_selection = 0;
        }
        if let Some(current) = menu {
            let rows = current.rows(saved);
            if pressed(KeyboardKey::KEY_UP) { menu_selection = (menu_selection + rows.len() - 1) % rows.len(); }
            if pressed(KeyboardKey::KEY_DOWN) { menu_selection = (menu_selection + 1) % rows.len(); }
            let mouse = presentation::canvas_point(rl.get_mouse_position(), viewport);
            let x = (carteros::W - 660) / 2;
            let click = focused && !opening_pause && !close_requested && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
            let clicked = click && mouse.x >= x as f32 && mouse.x <= (x + 660) as f32
                && (0..rows.len()).any(|i| {
                    let y = presentation::menu_row(i);
                    if (y as f32..(y + 48) as f32).contains(&mouse.y) { menu_selection = i; true } else { false }
                });
            let activate = clicked || (!opening_pause && !close_requested && pressed(KeyboardKey::KEY_ENTER));
            if current == Menu::Settings {
                let left = pressed(KeyboardKey::KEY_LEFT);
                let right = pressed(KeyboardKey::KEY_RIGHT);
                if (left || right || activate) && menu_selection < 3 {
                    match menu_selection {
                        0 => {
                            let value = &mut os.system.mouse_sensitivity;
                            *value = if left { value.saturating_sub(10).max(30) }
                                else if activate && *value == 200 { 30 }
                                else { (*value + 10).min(200) };
                        }
                        1 => os.system.invert_mouse = !os.system.invert_mouse,
                        _ => {
                            let value = &mut os.system.master_volume;
                            *value = if left { value.saturating_sub(10) }
                                else if activate && *value == 100 { 0 }
                                else { (*value + 10).min(100) };
                        }
                    }
                    settings_error = os.system.save(&system_state::SystemState::path()).err()
                        .map(|error| format!("Settings could not be saved. {error}"));
                }
                if (activate && menu_selection == 3) || pressed(KeyboardKey::KEY_ESCAPE) {
                    menu = Some(settings_return);
                    menu_selection = 0;
                }
            } else if !opening_pause && !close_requested && pressed(KeyboardKey::KEY_ESCAPE) && current != Menu::Title {
                menu = match current {
                    Menu::ConfirmNew | Menu::Difficulty => Some(Menu::Title),
                    Menu::QuitUnsaved => quit_return.take(),
                    _ => None,
                };
                menu_selection = 0;
            } else if activate {
                match rows[menu_selection] {
                    "Continue" => {
                        if let Some(restored) = Game::from_save() {
                            game = restored;
                            menu = None;
                        } else {
                            saved = false;
                            menu_selection = 0;
                        }
                    }
                    "New run" if saved || Snapshot::path().exists() => { menu = Some(Menu::ConfirmNew); menu_selection = 0; }
                    "New run" | "Replace with new run" => { menu = Some(Menu::Difficulty); menu_selection = 0; }
                    "Standard" | "Overtime" => {
                        let overtime = rows[menu_selection] == "Overtime";
                        let prepared = (!saved).then(|| (game.inbox_seed,std::mem::take(&mut game.inbox_text)));
                        let mut tracker = std::mem::take(&mut game.achievements);
                        tracker.restart_clean_run();
                        game = Game::new();
                        if let Some((seed,text)) = prepared { game.inbox_seed=seed; game.inbox_text=text; }
                        game.business.overtime = overtime;
                        game.achievements = tracker;
                        game.cheat_state.god_mode = false;
                        game.cheat_state.noclip = false;
                        let _ = game.cheat_state.write();
                        os = CarterOs::new();
                        debug_enabled = false;
                        console = DebugConsole::default();
                        hearing_feedback = None;
                        voice.reset();
                        if let Err(error) = game.save_campaign() { game.inform(&error); }
                        saved = Snapshot::load(&Snapshot::path()).is_some();
                        menu = None;
                    }
                    "Keep current run" | "Back to title" => { menu = Some(Menu::Title); menu_selection = 0; }
                    "Settings" => { settings_return = current; settings_error = None; menu = Some(Menu::Settings); menu_selection = 0; }
                    "Resume" => menu = None,
                    "Quit" | "Quit to last checkpoint" => break,
                    "Save and quit" => {
                        match game.save_campaign() {
                            Ok(()) => return,
                            Err(error) => {
                                game.inform(&error);
                                quit_return = Some(current);
                                menu = Some(Menu::QuitUnsaved);
                                menu_selection = 0;
                            }
                        }
                    }
                    "Keep playing" => menu = quit_return.take(),
                    _ => {}
                }
                last_money = game.money;
            }
            if !cursor_on { rl.enable_cursor(); cursor_on = true; }
            {
                let mut d = rl.begin_drawing(&thread);
                {
                    let mut ui = d.begin_texture_mode(&thread, &mut ui_target);
                    presentation::draw_menu(&mut ui, menu.unwrap_or(current), menu_selection, saved, &game,
                        &os.system, settings_error.as_deref());
                }
                presentation::present_canvas(&mut d, ui_target.texture(), viewport);
            }
            // Ending the frame polls input before control returns to gameplay.
            if menu.is_none() && game.screen == Screen::Office && !console.open {
                rl.disable_cursor();
                cursor_on = false;
            }
            continue;
        }
        if game.screen == Screen::Boss {
            // Arena uses its own input scope so cursor changes precede immutable reads.
            if !cursor_on { rl.enable_cursor(); cursor_on = true; }
            let pressed = |key| rl.is_key_pressed(key);
            if let Some(arena) = &mut game.arena {
                let held = |key| rl.is_key_down(key);
                let axis = |positive, negative| f32::from(held(positive)) - f32::from(held(negative));
                let mut mv = Vector2::new(axis(KeyboardKey::KEY_D,KeyboardKey::KEY_A),axis(KeyboardKey::KEY_S,KeyboardKey::KEY_W));
                let mag = mv.x.hypot(mv.y);
                if mag > 1.0 { mv.x /= mag; mv.y /= mag; }
                let select = [KeyboardKey::KEY_ONE,KeyboardKey::KEY_TWO,KeyboardKey::KEY_THREE,KeyboardKey::KEY_FOUR,
                    KeyboardKey::KEY_FIVE,KeyboardKey::KEY_SIX,KeyboardKey::KEY_SEVEN,KeyboardKey::KEY_EIGHT,
                    KeyboardKey::KEY_NINE,KeyboardKey::KEY_ZERO].into_iter().position(pressed);
                let gadget = [KeyboardKey::KEY_Q,KeyboardKey::KEY_Z,KeyboardKey::KEY_X,KeyboardKey::KEY_C].into_iter().position(pressed);
                let aim = presentation::canvas_point(rl.get_mouse_position(), viewport);
                let wheel = rl.get_mouse_wheel_move();
                arena.tick(dt,finanzamt::Input {
                    movement:mv, aim:Vector2::new(aim.x/carteros::W as f32,aim.y/carteros::H as f32),
                    fire:rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
                    held:rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
                    select, gadget, cycle: if wheel < 0.0 || pressed(KeyboardKey::KEY_PAGE_UP) {1}
                        else if wheel > 0.0 || pressed(KeyboardKey::KEY_PAGE_DOWN) {-1} else {0},
                    sprint:held(KeyboardKey::KEY_LEFT_SHIFT) || held(KeyboardKey::KEY_RIGHT_SHIFT),boost:pressed(KeyboardKey::KEY_R),
                });
            }
            let result = game.arena.as_ref().and_then(|a|a.result);
            if let Some(won)=result {
                if game.business.verdict==2 {
                    game.business.verdict=if won {3} else {4};
                    if let Err(error)=game.save_campaign(){eprintln!("{error}");}
                }
            }
            if result == Some(false) && pressed(KeyboardKey::KEY_R) {
                game.business.verdict = 2;
                game.enter_arena();
                if let Err(error) = game.save_campaign() { eprintln!("{error}"); }
                continue;
            }
            if result==Some(true) && pressed(KeyboardKey::KEY_ENTER) && game.arena.as_ref().is_some_and(|arena|!arena.ending) {
                if let Some(arena)=&mut game.arena {arena.ending=true;}
            } else if pressed(KeyboardKey::KEY_ESCAPE) || (result.is_some() && pressed(KeyboardKey::KEY_ENTER)) {
                game.screen=Screen::Title;
                menu=Some(Menu::Title);
                menu_selection=0;
                saved = Snapshot::load(&Snapshot::path()).is_some();
                continue;
            }
            let mut d=rl.begin_drawing(&thread);
            if let Some(arena) = &game.arena {
                {
                    let mut ui = d.begin_texture_mode(&thread, &mut ui_target);
                    arena.draw(&mut ui);
                }
                presentation::present_canvas(&mut d, ui_target.texture(), viewport);
            }
            continue;
        }
        if game.screen == Screen::Hearing {
            if pressed(KeyboardKey::KEY_F2) { voice.toggle_narration(); }
            let mut retained = false;
            if pressed(KeyboardKey::KEY_F4) && game.hearing.as_ref().is_some_and(|h| h.result.is_none() && !h.showing_docket) {
                voice.status = match game.business.debit_counsel(&mut game.money) {
                    Ok(()) => { retained = true; "Counsel retained. Advice appears below each question." }
                    Err(message) => message,
                };
            }
            if retained {
                if let Err(error) = game.save_campaign() { eprintln!("{error}"); }
            }
            if let Some(hearing) = &mut game.hearing {
                if let Some((round, receiver)) = &hearing_feedback {
                    match receiver.try_recv() {
                        Ok(response) => {
                            if let Some(response) = response {
                                if hearing.round == *round && hearing.result.is_none() {
                                    let aside: String = response.lines().next().unwrap_or("").chars().take(78).collect();
                                    if !aside.is_empty() {
                                        hearing.response.push_str("\n");
                                        hearing.response.push_str(&aside);
                                    }
                                }
                            }
                            hearing_feedback = None;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => hearing_feedback = None,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }
                }
                let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                if let Some(won) = hearing.result {
                    voice.narrate(3 + usize::from(won), os.system.speakers, os.system.master_volume);
                    if enter {
                        voice.reset();
                        hearing_feedback = None;
                        if won {
                            game.screen = Screen::Office;
                            game.inform("Office reopened. The Finanzamt balance is still due.");
                            game.hearing = None;
                        } else {
                            game.enter_arena();
                        }
                        continue;
                    }
                } else {
                    if pressed(KeyboardKey::KEY_TAB) {
                        voice.cancel_recording();
                        hearing.showing_docket = !hearing.showing_docket;
                    }
                    if pressed(KeyboardKey::KEY_ESCAPE) {
                        if voice.busy() {
                            voice.cancel_recording();
                        } else if hearing.showing_docket {
                            hearing.showing_docket = false;
                        } else {
                            voice.reset();
                            hearing_feedback = None;
                            game.screen = Screen::Title;
                            menu = Some(Menu::Title);
                            menu_selection = 0;
                            saved = Snapshot::load(&Snapshot::path()).is_some();
                            continue;
                        }
                    }
                    if !hearing.showing_docket && pressed(KeyboardKey::KEY_F8) {
                        if os.system.microphone { voice.toggle_recording(); }
                        else { voice.status = "Microphone is disabled in CarterOS settings."; }
                    }
                    if let Some(transcript) = voice.transcript() { hearing.input = transcript; }
                    // Repeat the current speaker on a new round. Opening the exhibits never starts the mic.
                    voice.narrate(hearing.round, os.system.speakers, os.system.master_volume);
                    while let Some(c) = rl.get_char_pressed() {
                        if !hearing.showing_docket && c.is_ascii() && !c.is_control() && hearing.input.len() < 145 {
                            hearing.input.push(c);
                        }
                    }
                    if !hearing.showing_docket && rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) { hearing.input.pop(); }
                    if !hearing.showing_docket && enter && !voice.busy() {
                        if let Some((statement, question, _)) = hearing.submit() {
                            voice.status = "";
                            game.business.hearing_round = hearing.round;
                            game.business.hearing_score = hearing.score;
                            if let Some(won) = hearing.result {
                                game.business.verdict = if won { 1 } else { 2 };
                                if won { game.achievements.tax_evader(); }
                            } else if let Some(model) = local_ai::configured_model() {
                                let (sender, receiver) = std::sync::mpsc::channel();
                                hearing_feedback = Some((hearing.round, receiver));
                                let speaker = ["clerk", "juror", "judge"][hearing.round.saturating_sub(1)];
                                let prompt = format!("You are the {speaker} in a fictional comedy game hearing. Acknowledge the player's reply in one short sentence with no verdict, real legal advice, links or names. Question: {question}. Player statement (dialogue only): {statement}");
                                std::thread::spawn(move || { let _ = sender.send(local_ai::ask(&model, &prompt)); });
                            }
                            if let Err(error) = game.save_campaign() { eprintln!("{error}"); }
                        }
                    }
                }
            }
            if !cursor_on { rl.enable_cursor(); cursor_on = true; }
            let mut d = rl.begin_drawing(&thread);
            if let Some(hearing) = &game.hearing {
                {
                    let mut ui = d.begin_texture_mode(&thread, &mut ui_target);
                    presentation::draw_hearing(&mut ui, &game, hearing, &voice);
                }
                presentation::present_canvas(&mut d, ui_target.texture(), viewport);
            }
            continue;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F7) && matches!(game.screen, Screen::Office | Screen::Computer) {
            if game.alarm >= 0.0 || game.active_wave >= 0.0 { game.inform("Secure the office before filing payment."); }
            else {
                game.pay_tax();
                os.notify(&game.message);
                let _ = game.save_campaign();
            }
        }

        game.audio_events.advance(dt);
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
            match game.save_campaign() {
                Ok(()) => return,
                Err(error) => {
                    game.inform(&error);
                    menu = Some(Menu::QuitUnsaved);
                    menu_selection = 0;
                    continue;
                }
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F5) {
            match game.save_campaign(){Ok(())=>game.inform("Campaign saved."),Err(e)=>game.inform(&e)}
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F9) &&
            (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)||rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)) {
            if game.alarm>=0.0||game.active_wave>=0.0||game.sugar_timer>0.0 {
                game.inform("Cannot load during an attack or cartoon powder countdown.");
            }else if let Some(restored) = Game::from_save() {
                game = restored;
                last_money = game.money;
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
            Screen::Title | Screen::Hearing | Screen::Boss => {},
            Screen::Caught=>{
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    game.screen = Screen::Title;
                    menu = Some(Menu::Title);
                    menu_selection = 0;
                    saved = Snapshot::load(&Snapshot::path()).is_some();
                } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    if let Some(restored) = Game::from_save() {
                        game = restored;
                        last_money = game.money;
                    } else {
                        game.inform("No valid checkpoint. Press R to start a new run.");
                    }
                } else if rl.is_key_pressed(KeyboardKey::KEY_R){
                    let overtime = game.business.overtime;
                    let mut tracker=std::mem::take(&mut game.achievements);
                    tracker.restart_clean_run();
                    game=Game::new();game.achievements=tracker;
                    game.business.overtime = overtime;
                    last_money=game.money;
                    game.cheat_state.god_mode=false;game.cheat_state.noclip=false;
                    let _=game.cheat_state.write();
                    os=CarterOs::new();debug_enabled=false;
                    let _=game.save_campaign();
                }
            },
            Screen::Office=>{
                if pressed(KeyboardKey::KEY_E) && game.outside && district::at_entrance(game.x,game.z) {
                    game.enter_new_office();
                } else if pressed(KeyboardKey::KEY_E) && game.outside {
                    game.claim_pickup();
                } else if pressed(KeyboardKey::KEY_E) && game.active_wave>=0.0 && game.at_chute() {
                    game.escape_chute();
                } else if pressed(KeyboardKey::KEY_E) && game.at_computer() && game.active_wave<0.0 && game.alarm<0.0 {
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
                    if rl.is_key_pressed(KeyboardKey::KEY_B){show_weapon_preview=!show_weapon_preview;}
                    if rl.is_key_pressed(KeyboardKey::KEY_F9)
                        && !rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                        && !rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL){animation_inspector=!animation_inspector;}
                    if rl.is_key_pressed(KeyboardKey::KEY_Q){game.drop_decoy();}
                    if rl.is_key_pressed(KeyboardKey::KEY_Z){game.use_airhorn();}
                    if rl.is_key_pressed(KeyboardKey::KEY_X){game.use_confetti();}
                    if rl.is_key_pressed(KeyboardKey::KEY_C){game.deploy_drone();}
                    if rl.is_key_pressed(KeyboardKey::KEY_R){
                        if game.sugar_visible && game.at_powder() {
                            game.take_sugar();
                        }else{
                            // Current mechanics use purchased charges, not magazines.
                            // R is a visual reload/inspection; it does NOT grant free ammo.
                            manual_reload_requested=true;
                        }
                    }
                    if !game.outside && rl.is_key_pressed(KeyboardKey::KEY_F){
                        game.collect_deliveries();
                        if game.active_wave < 0.0 {
                            if let Err(error) = game.save_campaign() { game.inform(&error); }
                        }
                    }
                    let md=rl.get_mouse_delta();
                    let sensitivity = 0.0032 * os.system.mouse_sensitivity as f32 / 100.0;
                    let vertical = if os.system.invert_mouse { -1.0 } else { 1.0 };
                    game.yaw=(game.yaw-md.x*sensitivity).rem_euclid(2.0*PI);
                    game.pitch=(game.pitch-md.y*sensitivity*vertical).clamp(-1.30,1.30);
                    let sprint = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                    let speed=(if game.outside {if sprint {5.3} else {3.4}} else if sprint {SPRINT_SPEED}else{WALK_SPEED})
                        *(if game.sugar_boost>0.0{1.50}else{1.0});
                    let mouse_left = MouseButton::MOUSE_BUTTON_LEFT;
                    let held_auto = matches!(game.arsenal.selected, 3..=6 | 13)
                        && game.arsenal.cooldown <= 0.0 && game.arsenal.charges[game.arsenal.selected] > 0
                        && rl.is_mouse_button_down(mouse_left);
                    if held_auto || rl.is_mouse_button_pressed(mouse_left) { game.fire_equipped(); }
                    let forward=(game.yaw.sin(),game.yaw.cos());
                    let right=(-game.yaw.cos(),game.yaw.sin());
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
                os.set_creator_state(game.studio_day==game.day,game.side.studio,game.side.fan_today(game.inbox_seed,game.day));
                for a in os.handle(&input,game.money){
                    if a==OsAction::DebugEnable {
                        let response=game.run_debug(DebugCommand::DebugOn,&mut debug_enabled);
                        os.notify(&response);
                        continue;
                    }
                    let first_cheat=!game.achievements.run.cheated;
                    handle_os_action(a,&mut game,&os);
                    if matches!(a,OsAction::VideoUpload(_)|OsAction::StudioComment(_)|OsAction::RemoteConsult|OsAction::BuildSpare|
                        OsAction::FanReply(_)|OsAction::AcceptMail|OsAction::DeleteMail|OsAction::InspectMail|OsAction::BuyItem(_)|OsAction::Spend(_)) {
                        if game.message_seconds>0.0 {os.show_action_result(&game.message);}
                        if game.active_wave<0.0 && !game.finished_day {
                            if let Err(error)=game.save_campaign(){os.show_action_result(&error);}
                        }
                    }
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
            if matches!(game.screen, Screen::Office | Screen::Computer) && game.active_wave < 0.0 && !game.finished_day {
                if let Err(error) = game.save_campaign() { game.inform(&error); }
            }
        }
        if game.finished_day {
            os.notify(&game.message);
            game.finished_day=false;
            if let Err(error) = game.save_campaign() {
                os.notify(&error);
                game.inform(&error);
            }
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
                    Cue::WeaponFire(index) if Some(index as usize) == selected_model_idx && pending_visual_idx.is_none() => {
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
                let target = if target == game.arsenal.selected { target } else { game.arsenal.selected };
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
                            2|10|11|15=>{if let Some(ref s)=gunshot_sound{s.play();}},
                            7..=9 | 14 => { if let Some(sound) = &gadget_sounds[3] { sound.play(); } },
                            _ => { if let Some(sound) = &gadget_sounds[2] { sound.play(); } },
                        }
                    },
                    Cue::Pickup|Cue::SugarPickup=>{if let Some(ref s)=pickup_sound{s.play();}},
                    Cue::PlayerCaught=>{if let Some(ref s)=hurt_sound{s.play();}},
                    Cue::Detonation=>{if let Some(ref s)=explosion_sound{s.play();}},
                    Cue::WeaponDraw=>{if let Some(ref s)=weapon_draw_sound{s.play();}},
                    Cue::WeaponEmpty | Cue::Airhorn | Cue::Confetti | Cue::Drone | Cue::SugarWarning | Cue::Hit => {
                        let slot = match cue {
                            Cue::WeaponEmpty => 0, Cue::Airhorn => 1, Cue::Confetti => 2,
                            Cue::Drone => 3, Cue::SugarWarning => 4, _ => 5,
                        };
                        if let Some(sound) = &gadget_sounds[slot] { sound.play(); }
                    },
                }
            }
        }
        let should_show_cursor=game.screen==Screen::Computer;
        if should_show_cursor!=cursor_on{
            if should_show_cursor{rl.enable_cursor();}else{rl.disable_cursor();}
            cursor_on=should_show_cursor;
        }

        let pos=Vector3::new(game.x,EYE_HEIGHT,game.z);
        let direction = look_direction(game.yaw, game.pitch);
        let camera=Camera3D::perspective(pos,
            Vector3::new(pos.x+direction.x,pos.y+direction.y,pos.z+direction.z),
            Vector3::new(0.0,1.0,0.0),75.0);
        let display_w=rl.get_screen_width() as f32;
        let display_h=rl.get_screen_height() as f32;
        let mut d=rl.begin_drawing(&thread);
        if game.screen==Screen::Computer{
            let mail = game.mail();
            let context=OsContext{
                cash:game.money,exposure:game.exposure,day:game.day,
                overhead:game.overhead(),archive_tip:game.side.archive_tip,
                risky_close:game.day>=3 && !game.defended_today && game.exposure>=game.closing_risk(),
                scans:game.scans,pis:game.raspberry_pis,alarm:game.alarm,
                active_wave:game.active_wave,
                orders_pending:game.pending_deliveries() as u32,
                deliveries_ready:game.ready_deliveries() as u32,
                next_delivery_eta:game.next_delivery_eta(),
                nearest_kid:game.kids.iter().map(|k|(k.x-game.x).hypot(k.z-game.z))
                    .min_by(|a,b|a.total_cmp(b)).unwrap_or(-1.0),
                front_visitors:game.kids.iter().filter(|k|k.z>1.20).count(),
                mail:MailView{pending:game.inbox_index<game.mail_count(),index:game.inbox_index,total:game.mail_count(),sender:&mail.sender,
                    subject:&mail.subject,body_a:&mail.first_line,body_b:&mail.second_line,extra:mail.extra_line.as_deref(),clue:mail.clue},
                studio_uploaded: game.studio_day == game.day, studio_result:game.side.studio,
                studio_comment:creator::comment(game.inbox_seed,game.day,game.side.studio==2,game.side.studio_project),
                consult_done:game.side.consult_done,spare_pc:game.side.spare_pc,
                fan_copy:creator::fan_copy(game.side.fan_today(game.inbox_seed,game.day),creator::fan_persona(game.inbox_seed,game.day)) ,
                fan_offer:game.side.fan_today(game.inbox_seed,game.day), tax_due: game.business.due,
                registered: game.business.registered(), sugar_timer: game.sugar_timer,
                studio_upgrade: game.business.studio_upgrade, storage_upgrade: game.business.storage_upgrade,
                armory: &game.arsenal, door_upgrade: game.door_upgrade,
                inspected:game.inspected,message:if game.message_seconds > 0.0 { &game.message } else { "" },kids:game.kids.len(),
            };
            let camera_rect = os.camera_rect();
            if camera_rect.is_some() {
                if let Some(target) = &mut security_target {
                    let mut monitor = d.begin_texture_mode(&thread, target);
                    monitor.clear_background(Color::new(22, 29, 38, 255));
                    let (cam_x,cam_z)=if game.relocated() {new_office::CAMERA} else {(-0.58,2.24)};
                    let security_camera = Camera3D::perspective(
                        Vector3::new(cam_x,2.23,cam_z), Vector3::new(0.05, 0.75, -0.20),
                        Vector3::new(0.0, 1.0, 0.0), 66.0);
                    let mut world = monitor.begin_mode3D(security_camera);
                    draw_world(&mut world, &game, office.as_ref(), fan_model.as_ref(), crate_model.as_ref());
                    world.draw_sphere(Vector3::new(game.x, 1.33, game.z), 0.12, Color::SKYBLUE);
                }
            }
            {
                let mut ui=d.begin_texture_mode(&thread,&mut ui_target);
                ui.clear_background(Color::BLACK);
                os.draw(&mut ui,&assets,&context);
                if let (Some(rect), Some(target)) = (camera_rect, &security_target) {
                    ui.draw_texture_pro(target.texture(), Rectangle::new(0.0, 0.0, 640.0, -214.0),
                        rect, Vector2::zero(), 0.0, Color::new(161, 201, 185, 255));
                    for y in (0..209).step_by(5) {
                        ui.draw_line(rect.x as i32, rect.y as i32 + y, (rect.x + rect.width) as i32,
                            rect.y as i32 + y, Color::new(18, 38, 33, 88));
                    }
                    ui.draw_rectangle(rect.x as i32 + 12, rect.y as i32 + 12, 164, 26, Color::new(13, 21, 25, 240));
                    ui.draw_text("CAM 01  /  LIVE", rect.x as i32 + 22, rect.y as i32 + 16, 17, Color::WHITE);
                }
                os.draw_run_status(&mut ui, &context);
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
            d.clear_background(if game.outside {Color::new(103,133,155,255)} else {Color::new(23,27,35,255)});
            {
                let mut world=d.begin_mode3D(camera);
                draw_world(&mut world, &game, office.as_ref(), fan_model.as_ref(), crate_model.as_ref());
            }
            // Foreground weapon: draw after world using a temporary depth
            // override. Do not cover the screen with an RGBA texture.
            if show_weapon_preview {
                if let Some(ref visual)=selected_visual {
                    {
                        let mut view=d.begin_mode3D(camera);
                        // World pixels are retained; only its depth buffer is
                        // ignored while the foreground model is drawn.
                        unsafe {
                            raylib::ffi::rlDrawRenderBatchActive();
                            raylib::ffi::rlDisableDepthTest();
                        }
                            // Camera forward is +Z at yaw=0; screen-right is -X.
                        let right=Vector3::new(-game.yaw.cos(),0.0,game.yaw.sin());
                            let forward=Vector3::new(game.yaw.sin(),0.0,game.yaw.cos());
                            let (lift,recoil,_roll)=visual.view_offsets();
                            let scale=visual.auto_scale()*VM_PROFILE_MULTIPLIERS[selected_model_idx.unwrap_or(game.arsenal.selected)]*vm_scale_multiplier;
                            let c=visual.center();
                            let (sin,cos)=(game.yaw.sin(),game.yaw.cos());
                            let pivot=Vector3::new(c.x*cos+c.z*sin,c.y,-c.x*sin+c.z*cos);
                            let view_pos=Vector3::new(
                                pos.x+direction.x*vm_distance+right.x*vm_right-forward.x*recoil-pivot.x*scale,
                                pos.y+direction.y*vm_distance+vm_down+lift-pivot.y*scale,
                                pos.z+direction.z*vm_distance+right.z*vm_right-forward.z*recoil-pivot.z*scale);
                        view.draw_model_ex(&visual.model,view_pos,
                            Vector3::new(0.0,1.0,0.0),game.yaw.to_degrees()+vm_yaw_offset,
                            Vector3::new(scale,scale,scale),Color::WHITE);
                        unsafe {
                            raylib::ffi::rlDrawRenderBatchActive();
                            raylib::ffi::rlEnableDepthTest();
                        }
                    }
                }
            }
            draw_overlay(&mut d,&game);
            if animation_inspector {
                let text=if let Some(ref visual)=selected_visual {
                    let (state,count)=visual.status();
                    format!("ANIMATION INSPECTOR [F9] | slot {} | {} | {} clip(s) | scale {:.5} | {}",
                        game.arsenal.selected+1,arsenal::CATALOG[game.arsenal.selected].asset,
                        count,visual.auto_scale()*VM_PROFILE_MULTIPLIERS[game.arsenal.selected]*vm_scale_multiplier,state)
                }else{format!("ANIMATION INSPECTOR [F9] | slot {} | MODEL NOT LOADED",game.arsenal.selected+1)};
                d.draw_rectangle(10,72,1120,36,Color::new(12,18,26,238));
                d.draw_text(&text,20,80,17,Color::YELLOW);
            }
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
    fn fresh_run_builds_an_investigation_and_accepts_the_summons() {
        let mut game = Game::new();
        game.buy(8);
        game.buy(arsenal::FIRST_SHOP_ID + 5);
        assert_eq!((game.business.sugar_orders, game.business.restricted_orders), (1, 1));
        for _ in 1..5 { game.finish_shift(); }
        assert!(game.mail().extra_line.unwrap().contains("powder receipt"));
        game.finish_shift();
        assert!(game.mail().extra_line.unwrap().contains("Minigun"));
        game.finish_shift();
        assert_eq!(game.day, 7);
        assert_eq!(game.mail().subject, "Summons to hearing 16-C");
        game.open_email(false);
        assert_eq!(game.inbox_index, 1);
        game.open_secret();
        assert!(game.hearing.is_some());
        assert_eq!(game.inbox_index, 0);
        assert!(game.business.summoned());
        assert!(game.snapshot().validate());
    }

    #[test]
    fn sponsor_income_repays_all_debt_and_books_gross_revenue() {
        let mut game = Game::new();
        game.bailout_debt = 600;
        game.side.pc_debt = 120;
        game.side.overhead_due = 45;
        let starting_cash = game.money;
        let gross = game.mail().payout;
        game.open_email(true);
        assert_eq!((game.bailout_debt, game.side.pc_debt, game.side.overhead_due), (0, 0, 0));
        assert_eq!(game.business.gross, gross);
        assert_eq!(game.money, starting_cash + gross - 765);
        assert_eq!(game.business.verified_cash, gross - 765);
    }

    #[test]
    fn grenade_launcher_profile_is_not_oversized(){
        // The original 1.65 factor clipped the entire right sleeve.
        assert!(VM_PROFILE_MULTIPLIERS[14]>=0.75 && VM_PROFILE_MULTIPLIERS[14]<=1.20);
    }
    #[test]
    fn every_weapon_has_finite_positive_viewmodel_scale(){
        assert_eq!(VM_PROFILE_MULTIPLIERS.len(),arsenal::COUNT);
        assert!(VM_PROFILE_MULTIPLIERS.iter().all(|v|v.is_finite()&&*v>0.1&&*v<5.0));
    }
    #[test]
    fn sponsorship_adds_money() {
        let mut g=Game::new();
        let pay = g.mail().payout;
        assert!(pay > 0);
        g.open_email(true);
        assert_eq!(g.money,STARTING_CASH + pay);
        assert_eq!(g.inbox_index,1);
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
        let mut g=Game::new();g.exposure=78.0;g.inbox_index=2;
        g.open_email(true);
        assert_eq!(g.exposure,100.0);
        assert_eq!(g.alarm,g.warning_seconds());
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
        let mut g=Game::new();g.buy(1);assert_eq!(g.money,STARTING_CASH-arsenal::GADGETS[0].2);
        assert_eq!(g.pending_deliveries(),1);
        assert_eq!(g.raspberry_pis,1);
        g.drop_decoy();assert_eq!(g.raspberry_pis,0);
        assert!(g.decoy.is_some());
    }
    #[test]
    fn physical_orders_arrive_and_require_pickup() {
        let mut g=Game::new();g.buy(1);
        assert_eq!(g.money,STARTING_CASH-arsenal::GADGETS[0].2);assert_eq!(g.raspberry_pis,1);
        assert_eq!(g.pending_deliveries(),1);
        g.tick(21.0);assert_eq!(g.ready_deliveries(),1);
        g.collect_deliveries();assert_eq!(g.raspberry_pis,1);
        g.x=0.0;g.z=2.0;g.collect_deliveries();
        assert_eq!(g.raspberry_pis,2);assert_eq!(g.ready_deliveries(),0);
    }
    #[test]
    fn cleared_mailbox_ends_shift_without_replaying_sponsorship() {
        let mut g=Game::new();g.inbox_index=inbox::count(g.day);
        let money=g.money;g.open_email(true);
        assert_eq!(g.money,money+SHIFT_PAY+PRIVACY_BONUS);assert_eq!(g.inbox_index,0);
        assert_eq!(g.day,2);
    }
    #[test]
    fn cleared_inbox_allows_safe_end_of_shift() {
        let mut g=Game::new();g.inbox_index=inbox::count(g.day);
        g.open_email(true);
        assert_eq!(g.day,2);assert_eq!(g.inbox_index,0);
        assert_eq!(g.money,STARTING_CASH+SHIFT_PAY+PRIVACY_BONUS);assert!(g.finished_day);
    }
    #[test]
    fn quiet_shifts_close_and_the_chute_preserves_the_active_shift() {
        let mut g = Game::new();
        g.end_shift();
        assert_eq!(g.day, 2);
        g.day = 100;
        g.end_shift();
        assert_eq!(g.day,101);
        g.exposure=85.0;
        g.end_shift();
        assert!(g.shift_closing);
        g.start_wave();
        assert!(g.fans_queued > 130);
        g.cheat_state.god_mode = true;
        for _ in 0..1600 { g.tick(0.05); }
        assert_eq!(g.day,101);
        g.x=CHUTE_X;g.z=CHUTE_Z;
        assert!(g.at_chute());
        g.escape_chute();
        assert_eq!(g.day,101);
        assert!(g.outside);
        g.enter_new_office();
        assert_eq!(g.day,101);
        (g.x,g.z)=district::ENTRANCE;
        g.enter_new_office();
        assert_eq!(g.day,102);
        assert!(!g.outside);
        assert!(g.kids.is_empty());
    }

    #[test]
    fn district_pickup_is_one_choice_per_escape_and_existing_spare_is_protected() {
        let mut g=Game::new();
        g.outside=true;
        (g.x,g.z)=(-10.5,-4.5);
        g.side.spare_pc=true;
        g.claim_pickup();
        assert!(!g.pickup_claimed);
        g.side.spare_pc=false;
        g.claim_pickup();
        assert!(g.side.spare_pc && g.pickup_claimed);
        let cash=g.money;
        g.hand_over_pcs(1);
        assert_eq!(g.money,cash);
        (g.x,g.z)=(11.5,-2.0);
        let pis=g.raspberry_pis;
        g.claim_pickup();
        assert_eq!(g.raspberry_pis,pis);
        g.pickup_claimed=false;
        g.claim_pickup();
        g.claim_pickup();
        assert_eq!(g.raspberry_pis,pis+2);
    }

    #[test]
    fn outdoor_reinforcements_spawn_on_open_ground_and_gain_on_a_sprinter() {
        let mut g=Game::new();
        g.outside=true;
        g.spawn_fans(26);
        for fan in &g.kids {
            assert!(district::walkable(fan.x,fan.z));
            assert!(fan.speed>5.3);
        }
    }

    #[test]
    fn late_operating_costs_make_idle_shifts_unprofitable_and_allow_recovery() {
        let mut game=Game::new();
        assert_eq!(game.overhead(),0);
        game.day=100;
        game.money=0;
        game.exposure=0.0;
        assert!(game.overhead()>SHIFT_PAY+PRIVACY_BONUS);
        game.finish_shift();
        assert_eq!(game.money,0);
        let owed=game.side.overhead_due;
        assert!(owed>0);
        game.income(creator::CONSULT_PAY);
        assert_eq!(game.side.overhead_due,0);
        assert_eq!(game.money,creator::CONSULT_PAY-owed);
    }

    #[test]
    fn relocated_office_keeps_the_desk_delivery_and_escape_reachable() {
        let mut g=Game::new();
        g.side.relocations=1;
        for (x,z) in [new_office::ARRIVAL,new_office::DESK,new_office::DELIVERY,new_office::CHUTE,new_office::PICKUP] {
            assert!(g.walkability()(x,z));
        }
        (g.x,g.z)=new_office::ARRIVAL;
        assert!(!g.at_computer());
        (g.x,g.z)=new_office::DESK;
        assert!(g.at_computer());
        (g.x,g.z)=new_office::DELIVERY;
        assert!(g.at_delivery_door());
        (g.x,g.z)=new_office::CHUTE;
        assert!(g.at_chute());
        assert!(g.snapshot().validate());
        let flow=nav::Flow::new(g.x,g.z,g.walkability());
        let mut pos=(0.8,2.4);
        for _ in 0..200 {
            if let Some((dx,dz))=flow.direction(pos.0,pos.1) {
                let distance=dx.hypot(dz);
                if distance>0.0 {
                    let shift=distance.min(0.06)/distance;
                    pos.0+=dx*shift;
                    pos.1+=dz*shift;
                    assert!(g.walkability()(pos.0,pos.1));
                }
            }
        }
        assert!((g.x-pos.0).hypot(g.z-pos.1)<0.28);
    }

    #[test]
    fn verified_fancord_offer_arrives_once_and_spare_pc_covers_one_tag() {
        let mut g=Game::new();
        let mut os=CarterOs::new();
        os.system.wifi=true;
        g.side.fan_day=g.day;
        g.side.fan_offer=2;
        handle_os_action(OsAction::FanReply(2),&mut g,&os);
        g.side.lead=1;
        handle_os_action(OsAction::FanReply(0),&mut g,&os);
        assert_eq!(g.money,STARTING_CASH);
        g.finish_shift();
        assert_eq!(g.mail().payout,1700);
        g.open_email(false);
        assert_eq!(g.mail().payout,4800);
        let cash=g.money;
        g.open_email(true);
        assert_eq!(g.money,cash+4800);
        assert_ne!(g.mail().payout,4800);
        handle_os_action(OsAction::BuildSpare,&mut g,&os);
        let cash=g.money;
        g.hand_over_pcs(1);
        assert_eq!(g.money,cash);
        assert!(!g.side.spare_pc);
        g.hand_over_pcs(1);
        assert_eq!(g.money,(cash-PC_COST).max(0));
    }

    #[test]
    fn starting_breach_has_three_faster_fans_and_tags_are_pc_bills() {
        let mut g=Game::new();
        g.cheat_state.god_mode=false;
        g.start_wave();
        assert_eq!(g.kids.len(),3);
        assert!(g.kids.iter().all(|k|k.speed>SPRINT_SPEED));
        g.kids[0].x=g.x;g.kids[0].z=g.z;
        g.tick(0.01);
        assert_eq!(g.kids.len(),2);
        assert_eq!(g.money,0);
        assert_eq!(g.side.pc_debt, PC_COST-STARTING_CASH);
        assert_eq!(g.snapshot().side.pc_debt,PC_COST-STARTING_CASH);
    }

    #[test]
    fn cheat_money_and_first_use_are_bounded(){
        let mut g=Game::new();g.cheat_state=cheats::CheatState::default();
        assert!(g.use_cheat(CheatKind::Money));
        assert_eq!(g.money,100_000+STARTING_CASH);assert!(!g.use_cheat(CheatKind::Money));
        assert_eq!(g.cheat_state.uses,2);assert!(g.cheat_state.pending_steam);
    }
    #[test]
    fn godmode_prevents_contact_and_noclip_keeps_save_in_bounds(){
        let mut g=Game::new();g.cheat_state=cheats::CheatState::default();
        g.use_cheat(CheatKind::GodMode);g.use_cheat(CheatKind::Noclip);
        move_player(&mut g,12.0,0.0);
        assert!(g.snapshot().validate());
        g.active_wave=35.0;g.kids.push(Kid{x:g.x,z:g.z,speed:0.0,variant:0,heading:0.0,shield:false});
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
        assert_eq!(g.money,1801);
        assert_eq!(g.bailout_debt,2100);
        assert!(g.achievements.run.bailed_out);
        assert_eq!(g.repay_debt(80),0);
        assert_eq!(g.bailout_debt,2020);
    }
    #[test]
    fn sound_events_follow_real_actions_not_fake_reloads() {
        let mut g=Game::new();
        assert!(g.audio_events.take().is_empty());
        g.arsenal.charges[0] = 1;
        g.fire_equipped();
        assert!(g.audio_events.take().contains(&Cue::WeaponFire(0)));
        g.fire_equipped();
        assert!(g.audio_events.take().is_empty(), "recharging equipment should stay quiet");
        g.arsenal.tick(arsenal::CATALOG[0].cooldown);
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
        assert!(std::mem::take(&mut g.delivery_notice));
        assert_eq!(g.audio_events.take(),vec![Cue::Delivery]);
        assert!(g.message.contains("package delivered"));
        let first=g.message_seconds;
        g.tick(0.25);
        assert!(g.message_seconds<first,"delivery message must not reset every frame");
        assert!(!g.delivery_notice);
        assert!(g.audio_events.take().is_empty());
    }
    #[test]
    fn district_fan_goes_around_a_corner_during_close_pursuit() {
        let mut game = Game::new();
        game.outside = true;
        game.active_wave = 0.0;
        (game.x, game.z) = (-1.53, -7.7);
        game.kids.push(Kid {
            x: -1.75, z: -7.0, speed: 5.85, variant: 0, heading: 0.0, shield: false,
        });
        for _ in 0..120 { game.tick(1.0 / 60.0); }
        assert_eq!(game.pc_handovers, 1);
    }

    #[test]
    fn fans_reach_players_near_furniture() {
        for relocated in [false, true] {
            for (x, z) in [(-0.65, -2.3), (0.31, 1.18), (0.77, 1.21)] {
                let mut game = Game::new();
                game.side.relocations = u32::from(relocated);
                if !(game.walkability())(x, z) { continue; }
                (game.x, game.z) = (x, z);
                game.start_wave();
                for _ in 0..360 {
                    game.tick(1.0 / 60.0);
                    if game.active_wave < 0.0 { break; }
                }
                assert_eq!(game.pc_handovers, 3, "fans stuck near ({x}, {z}), relocated={relocated}");
            }
        }
    }

    #[test]
    fn overtime_reinforcements_arrive_before_standard() {
        let mut game = Game::new();
        for incident in 0..=2 {
            game.side.incident = incident;
            game.business.overtime = false;
            let standard = game.reinforcement_delay();
            game.business.overtime = true;
            assert!(game.reinforcement_delay() < standard);
        }
    }

    #[test]
    fn inventory_at_capacity_stays_saveable() {
        let mut game = Game::new();
        game.airhorns = INVENTORY_CAP;
        let cash = game.money;
        game.buy(5);
        assert_eq!(game.money, cash);
        game.raspberry_pis = INVENTORY_CAP;
        game.scans = INVENTORY_CAP - 1;
        (game.x, game.z) = game.delivery_point();
        game.orders = vec![
            Delivery { item: 1, eta: 0.0, ready: true },
            Delivery { item: 3, eta: 0.0, ready: true },
        ];
        game.collect_deliveries();
        assert_eq!(game.raspberry_pis, INVENTORY_CAP);
        assert_eq!(game.scans, INVENTORY_CAP);
        assert!(game.snapshot().validate());
    }

    #[test]
    fn snapshot_captures_campaign_orders() {
        let mut g=Game::new();g.buy(3);
        let snap=g.snapshot();assert!(snap.validate());
        assert_eq!(snap.orders.len(),1);assert_eq!(snap.orders[0].item,3);
    }

}
