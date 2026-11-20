//! Playback for *existing* GLB clips; no synthetic skeletal keyframes.
//! 60 samples/sec matches raylib 6's glTF animation importer.
//! Named clips have their exact authored glTF time ranges. Single `allanims`
//! clips contain unlabelled sequences, so they are played only as full tracks
//! on use; assigning fake draw/reload boundaries would misrepresent the files.
use raylib::prelude::*;
use raylib::core::models::{ModelAnimations,RaylibModel};
use crate::arsenal;

pub const SAMPLE_HZ:f32=60.0;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Motion{Idle,Draw,Fire,Reload,Holster}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Clip{pub index:usize,pub start:f32,pub end:f32,pub looping:bool}
const fn c(index:usize,start:f32,end:f32)->Clip{Clip{index,start,end,looping:false}}
const fn lp(index:usize,start:f32,end:f32)->Clip{Clip{index,start,end,looping:true}}

#[derive(Debug,Clone,Copy)]
pub struct Profile{
    pub idle:Option<Clip>,pub draw:Option<Clip>,pub fire:Option<Clip>,
    pub reload:Option<Clip>,pub holster:Option<Clip>,pub merged:bool,
}
impl Profile{
    pub fn clip(self,kind:Motion)->Option<Clip>{match kind{
        Motion::Idle=>self.idle,Motion::Draw=>self.draw,Motion::Fire=>self.fire,
        Motion::Reload=>self.reload,Motion::Holster=>self.holster,
    }}
}
const fn merged(end:f32)->Profile{Profile{idle:Some(c(0,0.0,0.0)),draw:None,
    fire:Some(c(0,0.0,end)),reload:None,holster:None,merged:true}}
const STATIC:Profile=Profile{idle:None,draw:None,fire:None,reload:None,holster:None,merged:false};

/// Index/trim ranges are from the animation metadata inside the user's v0.15 GLBs.
/// These ranges are **seconds on the original glTF timeline**, not made-up keyframes.
pub const fn profile(slot:usize)->Profile{match slot{
    0=>Profile{idle:Some(c(0,0.042,0.042)),draw:None,fire:Some(c(0,0.042,3.75)),reload:None,holster:None,merged:true},
    1=>merged(4.833),
    2=>Profile{idle:None,draw:Some(c(3,4.50,5.333)),fire:Some(c(1,3.767,4.10)),
        reload:Some(c(0,0.033,3.767)),holster:Some(c(2,4.10,4.467)),merged:false},
    3=>Profile{idle:None,draw:Some(c(0,4.167,4.667)),fire:Some(c(1,3.367,3.567)),
        reload:Some(c(2,0.033,3.333)),holster:Some(c(3,3.60,4.133)),merged:false},
    4=>Profile{idle:None,draw:Some(c(0,8.967,10.0)),fire:Some(c(2,7.533,8.333)),
        reload:Some(c(1,0.033,3.667)),holster:Some(c(4,8.367,8.933)),merged:false},
    5=>merged(6.0),6=>merged(6.5),
    7|8|9=>STATIC,
    10=>merged(5.967),11=>merged(6.5),12=>merged(7.733),13=>merged(15.567),14=>merged(6.0),
    15=>Profile{idle:Some(lp(3,0.0,1.333)),draw:Some(c(6,0.0,1.033)),
        fire:Some(c(1,0.0,0.417)),reload:Some(c(4,0.0,1.133)),
        holster:Some(c(2,0.0,0.333)),merged:false},
    _=>STATIC,
}}

pub struct Player {slot:usize,motion:Motion,elapsed:f32}
impl Player {
    pub fn new(slot:usize)->Self{
        let action=if profile(slot).draw.is_some(){Motion::Draw}else{Motion::Idle};
        Self{slot,motion:action,elapsed:0.0}
    }
    pub fn state(&self)->Motion{self.motion}
    pub fn trigger(&mut self,motion:Motion)->bool{
        let p=profile(self.slot);
        if p.clip(motion).is_none(){return false;}
        // A long combined track (e.g. minigun's `allanims`) should not restart
        // every 90 ms during automatic fire.
        if motion==Motion::Fire && p.merged && self.motion==Motion::Fire{return true;}
        self.motion=motion;self.elapsed=0.0;true
    }
    pub fn step(&mut self,dt:f32)->Option<(usize,f32)> {
        let clip=profile(self.slot).clip(self.motion)?;
        let seconds=(clip.end-clip.start).max(0.0);
        if dt.is_finite()&&dt>0.0{self.elapsed+=dt.min(20.0);}
        let t=if seconds<=0.0{0.0}else if clip.looping{self.elapsed.rem_euclid(seconds)}else{self.elapsed.min(seconds)};
        let frame=(clip.start+t)*SAMPLE_HZ;
        if !clip.looping && self.elapsed>=seconds && self.motion!=Motion::Idle {
            self.motion=Motion::Idle;self.elapsed=0.0;
        }
        Some((clip.index,frame))
    }
}

pub struct WeaponVisual{pub model:Model,animations:Option<ModelAnimations>,player:Player}
impl WeaponVisual{
    pub fn load(rl:&mut RaylibHandle,thread:&RaylibThread,slot:usize)->Option<Self>{
        let weapon=arsenal::CATALOG.get(slot)?;
        let path=format!("assets/arsenal/{}",weapon.asset);
        let model=rl.load_model(thread,&path).ok()?;
        let animations=rl.load_model_animations(thread,&path).ok().filter(|set|{
            let relevant=[profile(slot).idle,profile(slot).draw,profile(slot).fire,
                profile(slot).reload,profile(slot).holster];
            relevant.iter().flatten().all(|clip|{
                let Some(anim)=set.as_slice().get(clip.index) else{return false;};
                model.is_model_animation_valid(anim)
            })
        });
        if animations.is_none() && profile(slot).fire.is_some(){
            eprintln!("GLB animation tracks unavailable/incompatible for {path}; static mesh fallback");
        }
        Some(Self{model,animations,player:Player::new(slot)})
    }
    pub fn trigger(&mut self,motion:Motion)->bool{self.animations.is_some()&&self.player.trigger(motion)}
    pub fn update(&mut self,rl:&mut RaylibHandle,thread:&RaylibThread,dt:f32){
        let Some(ref anims)=self.animations else{return;};
        let Some((index,frame))=self.player.step(dt) else{return;};
        let Some(anim)=anims.as_slice().get(index) else{return;};
        // raylib stores pose frames at 60 Hz even when source glTF keyframes
        // are sparse. Never submit an out-of-bounds interpolation index.
        let count=anim.as_ref().keyframeCount;
        if count>0{rl.update_model_animation(thread,&self.model,anim,frame.clamp(0.0,(count-1) as f32));}
    }
    pub fn status(&self)->(&'static str,usize){
        let mode=match self.player.state(){Motion::Idle=>"idle",Motion::Draw=>"draw",Motion::Fire=>"fire",
            Motion::Reload=>"reload",Motion::Holster=>"holster"};
        (mode,self.animations.as_ref().map_or(0,ModelAnimations::len))
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn named_clips_use_actual_source_indices(){
        assert_eq!(profile(4).draw.unwrap().index,0);
        assert_eq!(profile(4).fire.unwrap().index,2);
        assert_eq!(profile(4).reload.unwrap().index,1);
        assert_eq!(profile(3).fire.unwrap().index,1);
        assert_eq!(profile(15).idle.unwrap().index,3);
    }
    #[test]fn three_uploaded_models_have_no_embedded_clips(){
        for slot in [7,8,9]{let p=profile(slot);assert!(p.fire.is_none()&&p.reload.is_none());}
    }
    #[test]fn metadata_has_no_invalid_time_ranges(){
        for slot in 0..arsenal::COUNT{
            let p=profile(slot);
            for c in [p.idle,p.draw,p.fire,p.reload,p.holster].into_iter().flatten(){
                assert!(c.start>=0.0&&c.end>=c.start);
            }
        }
    }
    #[test]fn draw_fire_reload_progress_then_hold(){
        let mut p=Player::new(4);
        assert_eq!(p.state(),Motion::Draw);
        assert!(p.step(0.5).unwrap().1>8.967*SAMPLE_HZ);
        p.step(0.7);assert_eq!(p.state(),Motion::Idle);
        assert!(p.trigger(Motion::Fire));assert_eq!(p.step(0.05).unwrap().0,2);
        assert!(p.trigger(Motion::Reload));assert_eq!(p.step(0.04).unwrap().0,1);
    }
    #[test]fn automatic_fire_does_not_restart_single_long_track(){
        let mut p=Player::new(5);assert!(p.trigger(Motion::Fire));
        let a=p.step(0.4).unwrap().1;assert!(p.trigger(Motion::Fire));
        let b=p.step(0.1).unwrap().1;assert!(b>a);
    }
    #[test]fn pistol_idle_loops(){
        let mut p=Player::new(15);p.step(2.0);
        assert!(p.trigger(Motion::Idle));
        assert_eq!(p.step(1.34).unwrap().0,3);
        assert_eq!(p.state(),Motion::Idle);
    }
    #[test]fn static_weapon_does_not_claim_it_plays_animation(){
        let mut p=Player::new(8);assert!(!p.trigger(Motion::Fire));assert!(p.step(1.0).is_none());
    }
}
