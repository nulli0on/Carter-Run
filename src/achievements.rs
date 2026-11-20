//! RUN CARTER achievements: persistent lifetime record + clean per-run qualification.
//! No external tracking, no real file inspection beyond the game's own save files.
use std::{collections::BTreeSet,fs,path::PathBuf};

pub const ALL: &[(&str,&str)] = &[
    ("FULL_ARSENAL_ZERO_WASTE","Full Arsenal, Zero Waste"),
    ("AGAINST_ALL_ODDS","Against All Odds (Bullshit Survivor)"),
    ("EDGE_OF_DETONATION","Edge of Detonation"),
    ("IRON_WILL","Iron Will"),
    ("PURIST_MASTER","Purist Master"),
    ("SO_CLOSE","SO CLOSE"),
    ("UNLUCKY","Unlucky"),
    ("ADDICT","Addict"),
    ("CROWD_CONTROL","1,000 Beggars / Crowd Control"),
    ("SUGAR_FIEND","Sugar Fiend"),
    ("AI_VICTIM","AI Victim"),
    ("UNTOUCHABLE_LEGEND","Untouchable Legend"),
    ("ZERO_SHAME_TOTAL_SWEEP","Zero Shame, Total Sweep"),
    ("SHAME_SHAME_SHAME","Shame, Shame, Shame!"),
    ("SHAME_SHAME_SHAME_SHAME","Shame, Shame, Shame, Shame!"),
    ("SHAME_TIME_TRAVELER","Shame, Shame, Shame: Time Traveler"),
    ("SHAME_MEMORY_GREMLIN","Shame, Shame, Shame: Memory Gremlin"),
    ("SHAME_OUTSIDE_CONTRACTOR","Shame, Shame, Shame: Outside Contractor"),
    ("TOTALLY_LEGIT","Totally Legit (Trust Me Bro)"),
];

// All non-cheat, non-meta trophies must be earned during clean runs.
const CLEAN_REQUIREMENTS: &[&str] = &[
    "FULL_ARSENAL_ZERO_WASTE","AGAINST_ALL_ODDS","EDGE_OF_DETONATION",
    "IRON_WILL","PURIST_MASTER","SO_CLOSE","UNLUCKY","ADDICT",
    "CROWD_CONTROL","SUGAR_FIEND","AI_VICTIM","UNTOUCHABLE_LEGEND"
];
const PREFIX:&str="CARTER_ACHIEVEMENTS_V1";

#[derive(Debug,Clone,Default)]
pub struct RunProgress {
    pub cheated:bool, pub detonated:bool, pub bailed_out:bool,
    pub third_hit_seen:bool, pub bad_ai_dare:bool,
    pub hit_count:u32, pub sugar_hits:u32, pub bullshit_streak:u32,
    pub arsenal_mask:u16, pub wave_wins:u32,
}

#[derive(Debug,Clone,Default)]
pub struct Tracker {
    pub earned:BTreeSet<String>,
    pub pending:BTreeSet<String>, // Steam submission queue, not a verified Steam unlock.
    pub run:RunProgress,
    pub total_bullshit:u32, pub total_beggars:u32,
    pub total_sugar_hits:u32, pub total_emergency_saves:u32,
    pub new_unlocks:Vec<String>,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Tamper { SaveOrConfig, ClockJump, MemoryJump, ExternalModification }

impl Tracker {
    pub fn path()->PathBuf {PathBuf::from(".run_carter/achievements_v1.cfg")}
    pub fn label(id:&str)->&str {ALL.iter().find(|(key,_)|*key==id).map(|(_,name)|*name).unwrap_or("Unknown achievement")}
    pub fn load()->Self {
        let Ok(contents)=fs::read_to_string(Self::path())else{return Self::default()};
        if contents.len()>16384 || !contents.starts_with(PREFIX){return Self::default();}
        let mut v=Self::default();
        for line in contents.lines().skip(1) {
            if let Some(k)=line.strip_prefix("earned=") {
                if ALL.iter().any(|(id,_)|*id==k) {v.earned.insert(k.to_owned());}
            }else if let Some(k)=line.strip_prefix("pending=") {
                if ALL.iter().any(|(id,_)|*id==k) {v.pending.insert(k.to_owned());}
            }else if let Some((k,x))=line.split_once('=') {
                match k {
                    "total_bullshit"=>v.total_bullshit=x.parse().unwrap_or(0),
                    "total_beggars"=>v.total_beggars=x.parse().unwrap_or(0),
                    "total_sugar_hits"=>v.total_sugar_hits=x.parse().unwrap_or(0),
                    "total_emergency_saves"=>v.total_emergency_saves=x.parse().unwrap_or(0),
                    "run_cheated"=>v.run.cheated=x=="1",
                    "run_detonated"=>v.run.detonated=x=="1",
                    "run_bailed"=>v.run.bailed_out=x=="1",
                    "run_third"=>v.run.third_hit_seen=x=="1",
                    "run_dare"=>v.run.bad_ai_dare=x=="1",
                    "run_hits"=>v.run.hit_count=x.parse().unwrap_or(0),
                    "run_sugar"=>v.run.sugar_hits=x.parse().unwrap_or(0),
                    "run_streak"=>v.run.bullshit_streak=x.parse().unwrap_or(0),
                    "run_arsenal_v2"=>v.run.arsenal_mask=x.parse().unwrap_or(0),
                    "run_waves"=>v.run.wave_wins=x.parse().unwrap_or(0),
                    _=>{},
                }
            }
        }
        v.run.bullshit_streak=v.run.bullshit_streak.min(1_000_000);
        v.run.arsenal_mask &= ((1u32<<crate::arsenal::COUNT)-1) as u16;
        v
    }
    pub fn save(&self)->std::io::Result<()> {
        fs::create_dir_all(".run_carter")?;
        let mut s=format!("{PREFIX}\ntotal_bullshit={}\ntotal_beggars={}\ntotal_sugar_hits={}\ntotal_emergency_saves={}\nrun_cheated={}\nrun_detonated={}\nrun_bailed={}\nrun_third={}\nrun_dare={}\nrun_hits={}\nrun_sugar={}\nrun_streak={}\nrun_arsenal_v2={}\nrun_waves={}\n",
            self.total_bullshit,self.total_beggars,self.total_sugar_hits,self.total_emergency_saves,
            self.run.cheated as u8,self.run.detonated as u8,self.run.bailed_out as u8,
            self.run.third_hit_seen as u8,self.run.bad_ai_dare as u8,self.run.hit_count,
            self.run.sugar_hits,self.run.bullshit_streak,self.run.arsenal_mask,self.run.wave_wins);
        for id in &self.earned{s.push_str(&format!("earned={id}\n"));}
        for id in &self.pending{s.push_str(&format!("pending={id}\n"));}
        let path=Self::path();let tmp=path.with_extension("tmp");
        fs::write(&tmp,s)?;fs::rename(tmp,path)
    }
    pub fn restart_clean_run(&mut self){
        self.run=RunProgress::default();
        // If the final lifetime unlock was a shame/tamper achievement, completion
        // may only be claimed when a fresh clean run begins.
        self.check_completion();
        self.persist();
    }
    fn persist(&self){#[cfg(not(test))]if let Err(e)=self.save(){eprintln!("Achievement save failed: {e}");}}
    fn grant(&mut self,id:&str){
        if self.earned.insert(id.to_owned()) {
            self.pending.insert(id.to_owned());
            self.new_unlocks.push(id.to_owned());
        }
    }
    fn grant_clean(&mut self,id:&str){if !self.run.cheated{self.grant(id);self.check_completion();}else{self.persist();}}
    fn check_completion(&mut self){
        if self.run.cheated{return;}
        if CLEAN_REQUIREMENTS.iter().all(|id|self.earned.contains(*id)){
            self.grant("ZERO_SHAME_TOTAL_SWEEP");
        }
        if ALL.iter().filter(|(id,_)|*id!="TOTALLY_LEGIT").all(|(id,_)|self.earned.contains(*id)) {
            self.grant("TOTALLY_LEGIT");
        }
    }
    pub fn cheat(&mut self){self.run.cheated=true;self.run.bullshit_streak=0;self.grant("SHAME_SHAME_SHAME");self.persist();}
    pub fn tamper(&mut self,reason:Tamper){
        self.run.cheated=true;
        let id=match reason{Tamper::SaveOrConfig=>"SHAME_SHAME_SHAME_SHAME",Tamper::ClockJump=>"SHAME_TIME_TRAVELER",Tamper::MemoryJump=>"SHAME_MEMORY_GREMLIN",Tamper::ExternalModification=>"SHAME_OUTSIDE_CONTRACTOR"};
        self.grant(id);self.persist();
    }
    pub fn bullshit(&mut self){
        if self.run.cheated{return;}
        self.total_bullshit=self.total_bullshit.saturating_add(1);
        self.run.bullshit_streak=self.run.bullshit_streak.saturating_add(1);
        self.run.bad_ai_dare=true;
        if self.total_bullshit>=50{self.grant_clean("AI_VICTIM");}
        if self.run.bullshit_streak>=50{self.grant_clean("UNLUCKY");}
        self.persist();
    }
    pub fn answer_normal(&mut self){
        let streak=self.run.bullshit_streak;
        self.run.bullshit_streak=0;
        if (18..50).contains(&streak){self.grant_clean("SO_CLOSE");}
        self.persist();
    }
    pub fn dead_air(&mut self){self.run.bullshit_streak=0;self.persist();}
    pub fn purchase(&mut self,item:i32,remaining_cash:i32){
        if !(crate::arsenal::FIRST_SHOP_ID..crate::arsenal::FIRST_SHOP_ID+crate::arsenal::COUNT as i32).contains(&item) {return;}
        let bit=1u16<<(item-crate::arsenal::FIRST_SHOP_ID);
        self.run.arsenal_mask|=bit;
        // Starter cardboard blaster is owned even before the first purchase.
        self.run.arsenal_mask|=1;
        if self.run.arsenal_mask==((1u32<<crate::arsenal::COUNT)-1) as u16 && remaining_cash>0 && !self.run.bailed_out {
            self.grant_clean("FULL_ARSENAL_ZERO_WASTE");
        }
        self.persist();
    }
    pub fn bailout(&mut self){self.run.bailed_out=true;self.persist();}
    pub fn hit(&mut self){self.run.hit_count=self.run.hit_count.saturating_add(1);self.persist();}
    pub fn sugar(&mut self){
        self.run.sugar_hits=self.run.sugar_hits.saturating_add(1);
        if !self.run.cheated{self.total_sugar_hits=self.total_sugar_hits.saturating_add(1);}
        if self.run.sugar_hits==3{self.run.third_hit_seen=true;self.grant_clean("ADDICT");}
        if self.total_sugar_hits>=50 {self.grant_clean("SUGAR_FIEND");}
        self.persist();
    }
    pub fn emergency_save(&mut self,seconds_left:f32){
        if !(0.0..=3.0).contains(&seconds_left){return;}
        if !self.run.cheated {self.total_emergency_saves=self.total_emergency_saves.saturating_add(1);}
        if self.run.third_hit_seen{self.grant_clean("EDGE_OF_DETONATION");}
        if self.total_emergency_saves>=25{self.grant_clean("SUGAR_FIEND");}
        self.persist();
    }
    pub fn detonation(&mut self){self.run.detonated=true;self.persist();}
    pub fn wave_survived(&mut self,repelled:u32){
        self.run.wave_wins=self.run.wave_wins.saturating_add(1);
        if !self.run.cheated {self.total_beggars=self.total_beggars.saturating_add(repelled);}
        if self.run.bad_ai_dare {self.grant_clean("AGAINST_ALL_ODDS");}
        self.run.bad_ai_dare=false;
        if self.total_beggars>=1000 {self.grant_clean("CROWD_CONTROL");}
        self.persist();
    }
    pub fn victory(&mut self){
        if !self.run.detonated && self.run.third_hit_seen {self.grant_clean("IRON_WILL");}
        if self.run.wave_wins>=8 && !self.run.detonated && !self.run.bailed_out && self.run.sugar_hits==0 {
            self.grant_clean("PURIST_MASTER");
        }
        if self.run.wave_wins>=8 && self.run.hit_count==0{self.grant_clean("UNTOUCHABLE_LEGEND");}
        self.persist();
    }
    pub fn take_new(&mut self)->Vec<String>{std::mem::take(&mut self.new_unlocks)}
    pub fn has(&self,id:&str)->bool{self.earned.contains(id)}
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn cheating_disables_further_clean_unlocks_but_not_past(){
        let mut t=Tracker::default();t.sugar();t.sugar();t.sugar();assert!(t.has("ADDICT"));
        t.cheat();t.wave_survived(1000);assert!(t.has("ADDICT"));
        assert!(!t.has("CROWD_CONTROL"));assert!(t.has("SHAME_SHAME_SHAME"));
        t.restart_clean_run();t.wave_survived(1000);assert!(t.has("CROWD_CONTROL"));
    }
    #[test]fn streak_and_no_result_rules(){
        let mut t=Tracker::default();for _ in 0..18{t.bullshit();}t.answer_normal();assert!(t.has("SO_CLOSE"));
        for _ in 0..50{t.bullshit();}assert!(t.has("UNLUCKY"));assert!(t.has("AI_VICTIM"));
        t.restart_clean_run();for _ in 0..18{t.bullshit();}t.dead_air();assert_eq!(t.run.bullshit_streak,0);
    }
    #[test]fn arsenal_and_sugar_window(){
        let mut t=Tracker::default();for id in 10..(crate::arsenal::FIRST_SHOP_ID+crate::arsenal::COUNT as i32){t.purchase(id,50);}assert!(t.has("FULL_ARSENAL_ZERO_WASTE"));
        for _ in 0..3{t.sugar();}t.emergency_save(2.5);assert!(t.has("EDGE_OF_DETONATION"));
        t.restart_clean_run();for id in 10..(crate::arsenal::FIRST_SHOP_ID+crate::arsenal::COUNT as i32){t.purchase(id,50);}t.bailout();assert_eq!(t.run.bailed_out,true);
    }
    #[test]fn victory_requires_real_clean_run(){let mut t=Tracker::default();t.victory();assert!(!t.has("PURIST_MASTER"));
        for _ in 0..8{t.wave_survived(4);}t.victory();assert!(t.has("PURIST_MASTER"));assert!(t.has("UNTOUCHABLE_LEGEND"));}
    #[test]fn tamper_does_not_erase_prior_unlocks(){let mut t=Tracker::default();t.wave_survived(1000);t.tamper(Tamper::SaveOrConfig);assert!(t.has("CROWD_CONTROL"));assert!(t.has("SHAME_SHAME_SHAME_SHAME"));}
    #[test]fn completion_is_possible_after_a_clean_restart(){
        let mut t=Tracker::default();
        for (id,_) in ALL.iter().filter(|(id,_)|*id!="TOTALLY_LEGIT"){
            t.earned.insert((*id).to_owned());
        }
        t.run.cheated=true;
        t.restart_clean_run();
        assert!(t.has("TOTALLY_LEGIT"));
    }
}
